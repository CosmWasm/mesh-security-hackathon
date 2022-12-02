#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    coin, ensure_eq, to_binary, BankMsg, Binary, Decimal, Deps, DepsMut, Env, IbcMsg, MessageInfo,
    Order, Reply, Response, StdResult, SubMsg, SubMsgResponse, Uint128, WasmMsg,
};
use cw2::set_contract_version;
use cw_storage_plus::Bound;
use cw_utils::{parse_instantiate_response_data, Expiration};
use mesh_apis::ClaimProviderMsg;
use mesh_ibc::ProviderMsg;

use crate::error::ContractError;
use crate::ibc::build_timeout;
use crate::msg::{
    AccountResponse, ConfigResponse, ExecuteMsg, InstantiateMsg, ListValidatorsResponse, QueryMsg,
    StakeInfo, ValidatorResponse,
};
use crate::state::{
    Config, ValStatus, Validator, CHANNEL, CLAIMS, CONFIG, PACKET_LIFETIME, STAKED, VALIDATORS,
};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:mesh-provider";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

// for reply callbacks
const INIT_CALLBACK_ID: u64 = 1;

// Default packet life time = 1 hour
const DEFAULT_PACKET_LIFETIME: u64 = 60 * 60;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let state = Config {
        consumer: msg.consumer,
        slasher: None,
        lockup: deps.api.addr_validate(&msg.lockup)?,
        unbonding_period: msg.unbonding_period,
        rewards_ibc_denom: msg.rewards_ibc_denom,
    };
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    CONFIG.save(deps.storage, &state)?;

    // Set packet time from msg or set default
    PACKET_LIFETIME.save(
        deps.storage,
        &msg.packet_lifetime.unwrap_or(DEFAULT_PACKET_LIFETIME),
    )?;

    let label = format!("Slasher for {}", &env.contract.address);
    let msg = WasmMsg::Instantiate {
        admin: Some(env.contract.address.into_string()),
        code_id: msg.slasher.code_id,
        msg: msg.slasher.msg,
        funds: vec![],
        label,
    };
    let msg = SubMsg::reply_on_success(msg, INIT_CALLBACK_ID);

    Ok(Response::new()
        .add_submessage(msg)
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, reply: Reply) -> Result<Response, ContractError> {
    match reply.id {
        INIT_CALLBACK_ID => reply_init_callback(deps, reply.result.unwrap()),
        _ => Err(ContractError::InvalidReplyId(reply.id)),
    }
}

pub fn reply_init_callback(deps: DepsMut, resp: SubMsgResponse) -> Result<Response, ContractError> {
    CONFIG.update::<_, ContractError>(deps.storage, |mut cfg| {
        let init_response = parse_instantiate_response_data(&resp.data.unwrap_or_default())?;
        cfg.slasher = Some(deps.api.addr_validate(&init_response.contract_address)?);
        Ok(cfg)
    })?;
    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::ReceiveClaim {
            owner,
            amount,
            validator,
        } => execute_receive_claim(deps, info, env, owner, amount, validator),
        ExecuteMsg::Slash {
            validator,
            percentage,
            force_unbond,
        } => execute_slash(deps, info, env, validator, percentage, force_unbond),
        ExecuteMsg::Unstake { amount, validator } => {
            execute_unstake(deps, info, env, validator, amount)
        }
        ExecuteMsg::Unbond {} => execute_unbond(deps, info, env),
        ExecuteMsg::ClaimRewards { validator } => execute_claim_rewards(deps, env, info, validator),
    }
}

pub fn execute_receive_claim(
    deps: DepsMut,
    info: MessageInfo,
    env: Env,
    owner: String,
    amount: Uint128,
    validator: String,
) -> Result<Response, ContractError> {
    let cfg = CONFIG.load(deps.storage)?;
    ensure_eq!(cfg.lockup, info.sender, ContractError::Unauthorized);
    let owner = deps.api.addr_validate(&owner)?;

    if amount.is_zero() {
        return Err(ContractError::ZeroAmount);
    }

    let mut val = VALIDATORS
        .may_load(deps.storage, &validator)?
        .ok_or_else(|| ContractError::UnknownValidator(validator.clone()))?;
    let mut stake = STAKED
        .may_load(deps.storage, (&owner, &validator))?
        .unwrap_or_default();

    // First calculate rewards with old stake (or set default if first delegation)
    stake.calc_pending_rewards(
        val.rewards.rewards_per_token,
        val.shares_to_tokens(stake.shares),
    )?;

    stake.stake_validator(&mut val, amount);
    STAKED.save(deps.storage, (&owner, &validator), &stake)?;
    VALIDATORS.save(deps.storage, &validator, &val)?;

    // send out IBC packet for staking change
    let packet = ProviderMsg::Stake {
        validator,
        amount,
        key: owner.into_string(),
    };
    let msg = IbcMsg::SendPacket {
        channel_id: CHANNEL.load(deps.storage)?,
        data: to_binary(&packet)?,
        timeout: build_timeout(deps.as_ref(), &env)?,
    };
    Ok(Response::new().add_message(msg))
}

pub fn execute_slash(
    deps: DepsMut,
    info: MessageInfo,
    _env: Env,
    validator: String,
    percentage: Decimal,
    force_unbond: bool,
) -> Result<Response, ContractError> {
    let cfg = CONFIG.load(deps.storage)?;
    ensure_eq!(cfg.slasher, Some(info.sender), ContractError::Unauthorized);
    if percentage.is_zero() {
        return Err(ContractError::ZeroAmount);
    }

    VALIDATORS.update::<_, ContractError>(deps.storage, &validator, |val| {
        let mut val = val.ok_or_else(|| ContractError::UnknownValidator(validator.clone()))?;
        val.slash(percentage);
        if force_unbond {
            val.status = ValStatus::Tombstoned;
        }
        Ok(val)
    })?;

    Ok(Response::new()
        .add_attribute("action", "slash")
        .add_attribute("validator", validator))
}

pub fn execute_unstake(
    deps: DepsMut,
    info: MessageInfo,
    env: Env,
    validator: String,
    amount: Uint128,
) -> Result<Response, ContractError> {
    if amount.is_zero() {
        return Err(ContractError::ZeroAmount);
    }

    // updates the stake
    let mut val = VALIDATORS
        .may_load(deps.storage, &validator)?
        .ok_or_else(|| ContractError::UnknownValidator(validator.clone()))?;
    if val.status != ValStatus::Active {
        return Err(ContractError::RemovedValidator(validator));
    }
    let mut stake = STAKED.load(deps.storage, (&info.sender, &validator))?;

    // Calculate rewards with old stake
    stake.calc_pending_rewards(
        val.rewards.rewards_per_token,
        val.shares_to_tokens(stake.shares),
    )?;

    stake.unstake_validator(&mut val, amount)?;
    // check if we need to slash
    let slash = stake.take_slash(&val);
    STAKED.save(deps.storage, (&info.sender, &validator), &stake)?;
    VALIDATORS.save(deps.storage, &validator, &val)?;

    // create a future claim on number of shares (so we can adjust for later slashing)
    let cfg = CONFIG.load(deps.storage)?;
    let ready = env.block.time.plus_seconds(cfg.unbonding_period);
    CLAIMS.create_claim(
        deps.storage,
        &info.sender,
        amount,
        Expiration::AtTime(ready),
    )?;

    // send out IBC packet for staking change
    let packet = ProviderMsg::Unstake {
        validator,
        amount,
        key: info.sender.to_string(),
    };
    let msg = IbcMsg::SendPacket {
        channel_id: CHANNEL.load(deps.storage)?,
        data: to_binary(&packet)?,
        timeout: build_timeout(deps.as_ref(), &env)?,
    };
    let mut res = Response::new().add_message(msg);

    if let Some(slash) = slash {
        let msg = WasmMsg::Execute {
            contract_addr: cfg.lockup.into_string(),
            msg: to_binary(&ClaimProviderMsg::SlashClaim {
                owner: info.sender.into_string(),
                amount: slash,
            })?,
            funds: vec![],
        };
        res = res.add_message(msg);
    }

    Ok(res)
}

pub fn execute_unbond(
    deps: DepsMut,
    info: MessageInfo,
    env: Env,
) -> Result<Response, ContractError> {
    // TODO: slash tokens if we lost some during unbonding (requires larger changes to claiming)
    let mature = CLAIMS.claim_tokens(deps.storage, &info.sender, &env.block, None)?;
    if mature.is_zero() {
        return Err(ContractError::NothingToClaim);
    }

    let cfg = CONFIG.load(deps.storage)?;
    let msg = WasmMsg::Execute {
        contract_addr: cfg.lockup.into_string(),
        msg: to_binary(&ClaimProviderMsg::SlashClaim {
            owner: info.sender.into_string(),
            amount: mature,
        })?,
        funds: vec![],
    };
    Ok(Response::new().add_message(msg))
}

// HACK this implementation of claiming rewards is not performant or robust
// It is intended for proof of concept only.
pub fn execute_claim_rewards(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    validator: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    // calculate rewards
    let validator_info = VALIDATORS.load(deps.storage, &validator)?;
    let mut delegator_stake = STAKED.load(deps.storage, (&info.sender, &validator))?;

    // We calculate the rewards
    delegator_stake.calc_pending_rewards(
        validator_info.rewards.rewards_per_token,
        delegator_stake.shares,
    )?;

    if delegator_stake.rewards.pending.floor().is_zero() {
        return Err(ContractError::NoRewardsToClaim {});
    }

    let balance = deps
        .querier
        .query_balance(env.contract.address, config.rewards_ibc_denom.clone())?;

    // Make sure we have something to send, if its false, funds might be stuck in consumer and need admin. (or we messed up badly)
    if delegator_stake.rewards.pending > Decimal::new(balance.amount) {
        return Err(ContractError::WrongBalance {
            balance: balance.amount,
            rewards: delegator_stake.rewards.pending,
        });
    }

    let send_amount = delegator_stake.pending_to_u128()?;

    let msg = BankMsg::Send {
        to_address: info.sender.to_string(),
        amount: vec![coin(send_amount, config.rewards_ibc_denom)],
    };

    // Save new rewards
    delegator_stake.reset_pending();
    STAKED.save(deps.storage, (&info.sender, &validator), &delegator_stake)?;

    Ok(Response::new().add_message(msg))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::Account { address } => to_binary(&query_account(deps, address)?),
        QueryMsg::Validator { address } => to_binary(&query_validator(deps, address)?),
        QueryMsg::ListValidators { start_after, limit } => {
            to_binary(&list_validators(deps, start_after, limit)?)
        }
    }
}

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let cfg = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        consumer: cfg.consumer,
        slasher: cfg.slasher.map(|x| x.into_string()),
    })
}

pub fn query_account(deps: Deps, address: String) -> StdResult<AccountResponse> {
    let account = deps.api.addr_validate(&address)?;
    let staked = STAKED
        .prefix(&account)
        .range(deps.storage, None, None, Order::Ascending)
        .map(|res| {
            let (validator, stake) = res?;
            let val = VALIDATORS.load(deps.storage, &validator)?;
            let tokens = stake.current_value(&val);
            let slashed = stake.locked - tokens;
            Ok(StakeInfo {
                validator,
                tokens,
                slashed,
            })
        })
        .collect::<StdResult<Vec<_>>>()?;

    Ok(AccountResponse { staked })
}

pub fn query_validator(deps: Deps, address: String) -> StdResult<ValidatorResponse> {
    let val = VALIDATORS.load(deps.storage, &address)?;
    Ok(build_response((address, val)))
}

// settings for pagination
const MAX_LIMIT: u32 = 100;
const DEFAULT_LIMIT: u32 = 30;

pub fn list_validators(
    deps: Deps,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<ListValidatorsResponse> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start = start_after.as_ref().map(|x| Bound::exclusive(x.as_str()));

    let validators = VALIDATORS
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|r| Ok(build_response(r?)))
        .collect::<StdResult<Vec<_>>>()?;

    Ok(ListValidatorsResponse { validators })
}

fn build_response((address, val): (String, Validator)) -> ValidatorResponse {
    ValidatorResponse {
        address,
        tokens: val.stake_value(),
        status: val.status,
        multiplier: val.multiplier,
    }
}
