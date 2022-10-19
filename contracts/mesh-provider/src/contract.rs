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
    Config, ValStatus, Validator, CHANNEL, CLAIMS, CONFIG, STAKED, STAKED_BY_VALIDATOR,
    VALIDATORS, VALIDATOR_REWARDS,
};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:mesh-provider";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

// for reply callbacks
const INIT_CALLBACK_ID: u64 = 1;

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
        denom: msg.rewards_ibc_denom,
    };
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    CONFIG.save(deps.storage, &state)?;

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

#[entry_point]
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
        ExecuteMsg::ClaimRewards {validator} => execute_claim_rewards(deps, env, info, validator),
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
    stake.stake_validator(&mut val, amount);
    STAKED.save(deps.storage, (&owner, &validator), &stake)?;
    STAKED_BY_VALIDATOR.save(deps.storage, (&validator, &owner), &stake)?;
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
        timeout: build_timeout(&env),
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
    stake.unstake_validator(&mut val, amount)?;
    // check if we need to slash
    let slash = stake.take_slash(&val);
    STAKED.save(deps.storage, (&info.sender, &validator), &stake)?;
    STAKED_BY_VALIDATOR.save(deps.storage, (&validator, &info.sender), &stake)?;
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
        timeout: build_timeout(&env),
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
    validator: String
) -> Result<Response, ContractError> {
    let delegator = deps.api.addr_validate(info.sender.as_str())?;
    let config = CONFIG.load(deps.storage)?;

    // calculate rewards
    let validator_stake = VALIDATORS.load(deps.storage, &validator)?.stake;
    let delegator_stake = STAKED.load(deps.storage, (&delegator, &validator))?;
    let total_rewards = VALIDATOR_REWARDS.load(deps.storage, &validator)?;

    // Make sure we have something to send
    if total_rewards.is_zero() {
        return Err(ContractError::NoRewardsToClaim {});
    }

    // We calculate the rewards
    let rewards_to_send = delegator_stake.calc_rewards(validator_stake, total_rewards)?;

    if rewards_to_send.is_zero() {
        return Err(ContractError::NoRewardsToClaim {});
    }

    VALIDATOR_REWARDS.update::<_, ContractError>(deps.storage, &validator, |val| -> Result<Uint128, ContractError> {
        match val {
            Some(val) => Ok(val.checked_sub(rewards_to_send)?),
            None => Err(ContractError::NoRewardsToClaim),
        }
    })?;

    let balance = deps
        .querier
        .query_balance(env.contract.address, config.denom.clone())?;

    // Make sure we have something to send, if its false, funds are stuck in consumer and needed admin.
    if rewards_to_send > balance.amount {
        return Err(ContractError::WrongBalance {
            balance: balance.amount,
            rewards: rewards_to_send,
        });
    }

    let msg = BankMsg::Send {
        to_address: delegator.to_string(),
        amount: vec![coin(rewards_to_send.u128(), config.denom)],
    };

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

#[cfg(test)]
mod tests {
    use crate::msg::{ConsumerInfo, SlasherInfo};

    use super::*;
    use cosmwasm_std::coins;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {
            consumer: ConsumerInfo {
                connection_id: "1".to_string(),
            },
            slasher: SlasherInfo {
                code_id: 17,
                msg: b"{}".into(),
            },
            lockup: "lockup_contract".to_string(),
            unbonding_period: 86400 * 14,
            rewards_ibc_denom: "".to_string(),
        };
        let info = mock_info("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(1, res.messages.len());
    }
}
