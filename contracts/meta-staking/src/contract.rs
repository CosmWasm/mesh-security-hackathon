#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    ensure_eq, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response, StdResult,
};
use cw2::set_contract_version;
use cw_utils::parse_reply_execute_data;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, SudoMsg};
use crate::state::{Config, CONFIG};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:meta-staking";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const WITHDRAW_REWARDS_REPLY_ID: u64 = 0;

const DEFAULT_LIMIT: u32 = 100;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    // Save config
    CONFIG.save(
        deps.storage,
        &Config {
            // HACK for demo...
            admin: info.sender.to_string(),
            rewards_denom: msg.rewards_denom,
        },
    )?;

    Ok(Response::new().add_attribute("method", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Delegate {
            validator,
            staker,
            amount,
        } => execute::delegate(deps, env, info, validator, staker, amount),
        ExecuteMsg::Undelegate {
            validator,
            staker,
            amount,
        } => execute::undelegate(deps, env, info, validator, staker, amount),
        ExecuteMsg::WithdrawToCostumer {
            validator,
            staker,
            consumer,
        } => execute::withdraw_to_customer(deps, env, info, staker, validator, &consumer),
        ExecuteMsg::Sudo(sudo_msg) => {
            ensure_eq!(
                CONFIG.load(deps.storage)?.admin,
                info.sender,
                ContractError::Unauthorized {}
            );
            sudo(deps, env, sudo_msg)
        }
    }
}

mod execute {
    use std::vec;

    use mesh_apis::CallbackDataResponse;

    use cosmwasm_std::{
        coin, Addr, BankMsg, Coin, CosmosMsg, DistributionMsg, Order, ReplyOn, StakingMsg, SubMsg,
        Uint128,
    };

    use crate::state::{
        ValidatorRewards, CONSUMERS, CONSUMERS_BY_VALIDATOR, VALIDATORS_BY_CONSUMER,
        VALIDATORS_REWARDS,
    };

    use super::*;

    pub fn delegate(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        validator: String,
        staker: String,
        amount: Uint128,
    ) -> Result<Response, ContractError> {
        // TODO Validate validator valoper address

        // We Withdraw rewards from the validator (total rewards of all consumers)
        let (deps, mut response, validator_rewards, mut rewards_denom) =
            withdraw_validator_reward(deps, env, validator.clone())?;

        // We get delegations of this specific consumer to this specific validator.
        let delegations = (VALIDATORS_BY_CONSUMER.may_load(deps.storage, (&info.sender, &validator))?).unwrap_or_default();
        let mut consumer = CONSUMERS.load(deps.storage, &info.sender)?;

        // calculate consumer rewards till now (with old stake and rewards till now)
        consumer.calc_pending_rewards(validator_rewards.rewards_per_token, delegations)?;
        // HACK temporary work around for proof of concept. Real implementation
        // would use something like a generic Superfluid module to mint or burn
        // synthetic tokens.
        consumer.increase_stake(amount)?;

        // Update info for both of the maps
        // We add the amount delegated to the validator.
        let to_update = delegations.checked_add(amount)?;
        VALIDATORS_BY_CONSUMER.save(deps.storage, (&info.sender, &validator), &to_update)?;
        CONSUMERS_BY_VALIDATOR.save(deps.storage, (&validator, &info.sender), &to_update)?;

        // if the denom is empty, means we have no rewards from validator, so get denom from config.
        if rewards_denom.is_empty() {
            rewards_denom = (CONFIG.load(deps.storage)?).rewards_denom;
        }

        // Create message to delegate the underlying tokens
        let denom = deps.querier.query_bonded_denom()?;
        let sub_msg = SubMsg {
            id: 0,
            msg: CosmosMsg::Staking(StakingMsg::Delegate {
                validator: validator.clone(),
                amount: Coin { denom, amount },
            }),
            reply_on: ReplyOn::Never,
            gas_limit: None,
        };

        response = response.add_submessage(sub_msg);

        // Send the rewards to consumer
        let mut send_rewards = Coin::new(0_u128, "".to_string());

        if !consumer.rewards.pending.floor().is_zero() {
            // We have something to send.
            send_rewards = Coin {
                denom: rewards_denom,
                amount: Uint128::from(consumer.pending_to_u128()?),
            };
            let msg = BankMsg::Send {
                to_address: info.sender.to_string(),
                amount: vec![send_rewards.clone()],
            };

            // We add the bankMsg into response.
            response = response.add_message(msg);
            // We reset pending rewards because we send them away.
            consumer.reset_pending_rewards();
        }

        // We save consumer
        CONSUMERS.save(deps.storage, &info.sender, &consumer)?;

        // We set data so we can get this data in reply and update consumer.
        // Even if we don't have rewards to send to consumer, we still need to update the stake there.
        let set_data = to_binary(&CallbackDataResponse {
            validator,
            staker,
            stake_amount: amount,
            rewards: send_rewards,
        })?;

        Ok(response.set_data(set_data))
    }

    pub fn undelegate(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        validator: String,
        staker: String,
        amount: Uint128,
    ) -> Result<Response, ContractError> {
        // TODO Validate validator valoper address

        // We Withdraw rewards from the validator (total rewards of all consumers)
        let (deps, mut response, validator_rewards, mut rewards_denom) =
            withdraw_validator_reward(deps, env, validator.clone())?;

        // We get delegations of this specific consumer to this specific validator.
        let delegations = (VALIDATORS_BY_CONSUMER.may_load(deps.storage, (&info.sender, &validator))?).unwrap_or_default();
        let mut consumer = CONSUMERS.load(deps.storage, &info.sender)?;

        // calculate consumer rewards till now (with old stake and rewards till now)
        consumer.calc_pending_rewards(validator_rewards.rewards_per_token, delegations)?;
        // HACK temporary work around for proof of concept. Real implementation
        // would use something like a generic Superfluid module to mint or burn
        // synthetic tokens.
        consumer.decrease_stake(amount)?;

        // Update info for both of the maps
        // We add the amount delegated to the validator.
        let to_update = delegations
            .checked_sub(amount)
            .map_err(|_| ContractError::InsufficientDelegation {})?;
        VALIDATORS_BY_CONSUMER.save(deps.storage, (&info.sender, &validator), &to_update)?;
        CONSUMERS_BY_VALIDATOR.save(deps.storage, (&validator, &info.sender), &to_update)?;

        // if the denom is empty, means we have no rewards from validator, so get denom from config.
        if rewards_denom.is_empty() {
            rewards_denom = (CONFIG.load(deps.storage)?).rewards_denom;
        }

        // Create message to delegate the underlying tokens
        let denom = deps.querier.query_bonded_denom()?;
        let sub_msg = SubMsg {
            id: 0,
            msg: CosmosMsg::Staking(StakingMsg::Undelegate {
                validator: validator.clone(),
                amount: Coin { denom, amount },
            }),
            reply_on: ReplyOn::Never,
            gas_limit: None,
        };

        response = response.add_submessage(sub_msg);

        // Send the rewards to consumer
        let mut send_rewards = Coin::new(0_u128, "".to_string());

        if !consumer.rewards.pending.floor().is_zero() {
            // We have something to send.
            send_rewards = Coin {
                denom: rewards_denom,
                amount: Uint128::from(consumer.pending_to_u128()?),
            };
            let msg = BankMsg::Send {
                to_address: info.sender.to_string(),
                amount: vec![send_rewards.clone()],
            };

            // We add the bankMsg into response.
            response = response.add_message(msg);
            // We reset pending rewards because we send them away.
            consumer.reset_pending_rewards();
        }

        // We save consumer
        CONSUMERS.save(deps.storage, &info.sender, &consumer)?;

        // We set data so we can get this data in reply and update consumer.
        // Even if we don't have rewards to send to consumer, we still need to update the stake there.
        let set_data = to_binary(&CallbackDataResponse {
            validator,
            staker,
            stake_amount: amount,
            rewards: send_rewards,
        })?;

        Ok(response.set_data(set_data))
    }

    fn withdraw_validator_reward(
        deps: DepsMut,
        env: Env,
        validator: String,
    ) -> Result<(DepsMut, Response, ValidatorRewards, String), ContractError> {
        // We get validator rewards and set default if nothing there.
        let mut validator_rewards = VALIDATORS_REWARDS
            .may_load(deps.storage, &validator)?
            .unwrap_or_default();
        let response = Response::new();

        // Query fullDelegation to get the total rewards amount
        let delegation_query = deps
            .querier
            .query_delegation(env.contract.address, validator.clone())?;

        // Total accumulated rewards we have from this validator
        let total_accumulated_rewards = &match delegation_query {
            Some(delegation) => delegation.accumulated_rewards,
            None => vec![coin(0_u128, "")],
        }[0];

        if total_accumulated_rewards.amount.is_zero() {
            // Nothing to withdraw or calculate, so just return default response.
            Ok((
                deps,
                response,
                validator_rewards,
                total_accumulated_rewards.denom.clone(),
            ))
        } else {
            // We have rewards to send.
            let total_delegations = CONSUMERS_BY_VALIDATOR
                .prefix(&validator)
                .range(deps.storage, None, None, Order::Ascending)
                .map(|res| -> StdResult<Uint128> { Ok(res?.1) })
                .sum::<StdResult<Uint128>>()?;

            // Calculate total rewards we got from validator (total rewards / total stake)
            validator_rewards.calc_rewards(total_accumulated_rewards.amount, total_delegations)?;
            VALIDATORS_REWARDS.save(deps.storage, &validator, &validator_rewards)?;

            // Withdraw the rewards from validator
            let withdraw_msg = SubMsg {
                id: 0,
                msg: CosmosMsg::Distribution(DistributionMsg::WithdrawDelegatorReward {
                    validator: validator.to_string(),
                }),
                reply_on: ReplyOn::Never,
                gas_limit: None,
            };

            Ok((
                deps,                                         //deps cause we need mut here.
                Response::new().add_submessage(withdraw_msg), //response with the withdraw message
                validator_rewards,                            //because we already load it here.
                total_accumulated_rewards.denom.clone(),      // To send the coins to consumer
            ))
        }
    }

    pub fn withdraw_to_customer(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        validator: String,
        staker: String,
        consumer_addr: &Addr,
    ) -> Result<Response, ContractError> {
        let (deps, mut response, validator_rewards, mut rewards_denom) =
            withdraw_validator_reward(deps, env, validator.clone())?;

        let mut consumer = CONSUMERS.load(deps.storage, consumer_addr)?;

        // consumer delegations to this validator
        let delegations = (VALIDATORS_BY_CONSUMER.may_load(deps.storage, (consumer_addr, &validator))?).unwrap_or_default();

        // Do the consumer rewards calculation
        consumer.calc_pending_rewards(validator_rewards.rewards_per_token, delegations)?;

        let mut send_rewards = Coin::new(0_u128, "".to_string());

        if !consumer.rewards.pending.floor().is_zero() {
            // if the denom is empty, means we have no rewards from validator, so get denom from config.
            if rewards_denom.is_empty() {
                rewards_denom = (CONFIG.load(deps.storage)?).rewards_denom;
            }

            // We have something to send.
            send_rewards = Coin {
                denom: rewards_denom,
                amount: Uint128::from(consumer.pending_to_u128()?),
            };
            let msg = BankMsg::Send {
                to_address: info.sender.to_string(),
                amount: vec![send_rewards.clone()],
            };

            // We add the bankMsg into response.
            response = response.add_message(msg);
            // We reset pending rewards because we send them away.
            consumer.reset_pending_rewards();
        }

        // Save consumer
        CONSUMERS.save(deps.storage, consumer_addr, &consumer)?;

        // We set data so we can get this data in reply and update consumer.
        // Even if we don't have rewards to send to consumer, we still need to update the stake there.
        let set_data = to_binary(&CallbackDataResponse {
            validator,
            staker,
            stake_amount: Uint128::from(0_u128),
            rewards: send_rewards,
        })?;

        Ok(response.set_data(set_data))
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::AllDelegations { consumer } => {
            to_binary(&query::all_delegations(deps, consumer)?)
        }
        QueryMsg::AllValidators {
            consumer,
            start,
            limit,
        } => query::all_validators(deps, consumer, start, limit),
        QueryMsg::Consumer { address } => query::consumer(deps, address),
        QueryMsg::Consumers { start, limit } => query::consumers(deps, start, limit),
        QueryMsg::Delegation {
            consumer,
            validator,
        } => query::delegation(deps, consumer, validator),
    }
}

mod query {
    use crate::msg::{AllDelegationsResponse, Delegation};
    use cosmwasm_std::{to_binary, Addr, Order};
    use cw_storage_plus::Bound;
    use cw_utils::maybe_addr;

    use crate::state::{CONSUMERS, VALIDATORS_BY_CONSUMER};

    use super::*;

    pub fn all_delegations(deps: Deps, consumer: String) -> StdResult<AllDelegationsResponse> {
        let consumer = deps.api.addr_validate(&consumer)?;
        let delegations = VALIDATORS_BY_CONSUMER
            .prefix(&consumer)
            .range(deps.storage, None, None, Order::Ascending)
            .map(|r| {
                let (validator, delegation) = r?;
                Ok(Delegation {
                    validator,
                    delegation,
                })
            })
            .collect::<StdResult<Vec<_>>>()?;
        Ok(AllDelegationsResponse { delegations })
    }

    pub fn all_validators(
        deps: Deps,
        consumer: String,
        start: Option<String>,
        limit: Option<u32>,
    ) -> StdResult<Binary> {
        let limit = limit.unwrap_or(DEFAULT_LIMIT) as usize;
        let consumer = deps.api.addr_validate(&consumer)?;
        let start_bound = start.as_deref().map(Bound::exclusive);

        let validators = VALIDATORS_BY_CONSUMER
            .prefix(&consumer)
            .keys(deps.storage, start_bound, None, Order::Ascending)
            .take(limit)
            .collect::<StdResult<Vec<_>>>()?;
        to_binary(&validators)
    }

    pub fn delegation(deps: Deps, consumer: String, validator: String) -> StdResult<Binary> {
        let consumer_addr = deps.api.addr_validate(&consumer)?;
        let delegation = VALIDATORS_BY_CONSUMER
            .may_load(deps.storage, (&consumer_addr, &validator))?
            .unwrap_or_default();
        to_binary(&delegation)
    }

    pub fn consumer(deps: Deps, address: String) -> StdResult<Binary> {
        let addr = deps.api.addr_validate(&address)?;
        let consumer = CONSUMERS.load(deps.storage, &addr)?;
        to_binary(&consumer)
    }

    pub fn consumers(deps: Deps, start: Option<String>, limit: Option<u32>) -> StdResult<Binary> {
        let limit = limit.unwrap_or(DEFAULT_LIMIT) as usize;
        let start_addr = maybe_addr(deps.api, start)?;
        let start = start_addr.as_ref().map(Bound::exclusive);

        let consumers = CONSUMERS
            .keys(deps.storage, start, None, Order::Ascending)
            .take(limit)
            .collect::<StdResult<Vec<Addr>>>()?;

        to_binary(&consumers)
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn sudo(deps: DepsMut, env: Env, msg: SudoMsg) -> Result<Response, ContractError> {
    match msg {
        SudoMsg::AddConsumer {
            consumer_address,
            funds_available_for_staking,
        } => sudo::add_consumer(deps, env, consumer_address, funds_available_for_staking),
        SudoMsg::RemoveConsumer { consumer_address } => {
            sudo::remove_consumer(deps, env, consumer_address)
        }
    }
}

mod sudo {
    use cosmwasm_std::{ensure, Coin};

    use crate::state::{ConsumerInfo, CONSUMERS};

    use super::*;

    pub fn add_consumer(
        deps: DepsMut,
        env: Env,
        consumer_address: String,
        funds_available_for_staking: Coin,
    ) -> Result<Response, ContractError> {
        let _config = CONFIG.load(deps.storage)?;

        // Validate consumer address
        let address = deps.api.addr_validate(&consumer_address)?;

        // Check consumer doesn't already exist
        ensure!(
            !CONSUMERS.has(deps.storage, &address),
            ContractError::ConsumerAlreadyExists {}
        );

        // Check denom
        let denom = deps.querier.query_bonded_denom()?;

        // Check there are enough funds available to fund consumer
        let contract_balance = deps
            .as_ref()
            .querier
            .query_balance(env.contract.address, denom.clone())?;

        ensure!(
            contract_balance.amount >= funds_available_for_staking.amount,
            ContractError::NotEnoughFunds {denom, balance: contract_balance.amount, funds: funds_available_for_staking.amount}
        );

        CONSUMERS.save(
            deps.storage,
            &address,
            &ConsumerInfo::new(funds_available_for_staking.amount),
        )?;

        Ok(Response::new())
    }

    pub fn remove_consumer(
        deps: DepsMut,
        _env: Env,
        consumer_address: String,
    ) -> Result<Response, ContractError> {
        let _config = CONFIG.load(deps.storage)?;

        // Validate consumer address
        let address = deps.api.addr_validate(&consumer_address)?;

        // Check consumer exists
        ensure!(
            CONSUMERS.has(deps.storage, &address),
            ContractError::NoConsumer {}
        );

        // Remove consumer
        CONSUMERS.remove(deps.storage, &address);

        // TODO revisit what other cleanup do we need here?
        // Unbond all assets for all validators?

        Ok(Response::new())
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> Result<Response, ContractError> {
    match msg.id {
        WITHDRAW_REWARDS_REPLY_ID => reply::forward_rewards_to_consumer(deps, env, msg),
        _ => Err(ContractError::UnknownReplyID {}),
    }
}

mod reply {
    use super::*;

    pub fn forward_rewards_to_consumer(
        _deps: DepsMut,
        _env: Env,
        msg: Reply,
    ) -> Result<Response, ContractError> {
        // Send funds to consumer
        // IbcMsg to provider
        let res = parse_reply_execute_data(msg)?;
        println!("{:?}", res);

        Ok(Response::new())
    }
}

#[cfg(test)]
mod tests {
    use core::panic;

    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coin, coins, Addr, Uint128, QueryRequest, StakingQuery, from_binary, BondedDenomResponse, Validator, Decimal, from_slice};
    use mesh_apis::CallbackDataResponse;

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {
            rewards_denom: "wasm".to_string(),
        };
        let info = mock_info("creator", &coins(1000, ""));

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());
    }

    const ADMIN: &str = "admin_addr";
    const DENOM: &str = "utest";
    const REWARDS_DENOM: &str = "ucosm";
    const VALIDATOR_ADDR: &str = "some_validator";
    const STAKER_ADDR: &str = "some_staker";
    const CONSUMER_ADDR: &str = "some_consumer";
    const AMOUNT: Uint128 = Uint128::new(1000_u128);

    #[test]
    fn test_delegate() {
        let validator = Validator {
            address: VALIDATOR_ADDR.to_string(),
            commission: Decimal::one(),
            max_commission: Decimal::one(),
            max_change_rate: Decimal::zero(),
        };
        let mut deps = mock_dependencies();

        deps.querier.update_staking(DENOM, &[validator],&[]);

        // Init the contract
        let info = mock_info(ADMIN, &coins(10000, DENOM));
        let msg = InstantiateMsg {
            rewards_denom: REWARDS_DENOM.to_string(),
        };

        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Add consumer
        let info = mock_info(ADMIN, &coins(1000, DENOM));
        let env = mock_env();

        deps.querier.update_balance(env.contract.address, coins(10000, DENOM));

        let msg = ExecuteMsg::Sudo(SudoMsg::AddConsumer {
            consumer_address: CONSUMER_ADDR.to_string(),
            funds_available_for_staking: coin(1000, DENOM),
        });
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Do delegate from consumer
        let info = mock_info(CONSUMER_ADDR, &coins(10000, DENOM));

        let msg = ExecuteMsg::Delegate {
            validator: VALIDATOR_ADDR.to_string(),
            staker: STAKER_ADDR.to_string(),
            amount: AMOUNT,
        };
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        let binary = res.data.unwrap();
        println!("{:?}", from_slice::<CallbackDataResponse>(&binary));
        panic!();
    }
}
