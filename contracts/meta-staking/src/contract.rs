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
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let denom = deps.querier.query_bonded_denom()?;

    // Save config
    CONFIG.save(
        deps.storage,
        &Config {
            // HACK for demo...
            admin: info.sender.to_string(),
            denom,
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
        ExecuteMsg::Delegate { validator, amount } => {
            execute::delegate(deps, env, info, validator, amount)
        }
        ExecuteMsg::Undelegate { validator, amount } => {
            execute::undelegate(deps, env, info, validator, amount)
        }
        ExecuteMsg::WithdrawDelegatorReward { validator } => {
            execute::withdraw_delegator_reward(deps, env, validator)
        }
        ExecuteMsg::WithdrawToCostumer {
            consumer,
            validator,
        } => execute::withdraw_to_customer(deps, env, consumer, validator),
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

    use mesh_apis::ConsumerExecuteMsg;

    use cosmwasm_std::{
        coin, Coin, CosmosMsg, DistributionMsg, Order, StakingMsg, Uint128, WasmMsg,
    };

    use crate::state::{
        ValidatorRewards, CONSUMERS, CONSUMERS_BY_VALIDATOR, REWARDS_DENOM, VALIDATORS_BY_CONSUMER,
        VALIDATORS_REWARDS,
    };

    use super::*;

    pub fn delegate(
        mut deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        validator: String,
        amount: Uint128,
    ) -> Result<Response, ContractError> {
        // TODO Validate validator valoper address

        // If its a first delegation to a validator, we set validator rewards to 0
        let validator_rewards = VALIDATORS_REWARDS.may_load(deps.storage, &validator)?;

        let validator_rewards = match validator_rewards {
            Some(val_rewards) => val_rewards,
            None => {
                let val = ValidatorRewards::default();

                VALIDATORS_REWARDS.save(deps.storage, &validator, &val)?;
                val
            }
        };

        let delegations = VALIDATORS_BY_CONSUMER
            .load(deps.storage, (&info.sender, &validator))
            .unwrap_or_else(|_| Uint128::zero());

        CONSUMERS.update(
            deps.storage,
            &info.sender,
            |cons| -> Result<_, ContractError> {
                // fail if consumer was never registered
                let mut cons = cons.ok_or(ContractError::Unauthorized {})?;
                // calculate consumer rewards till now (with old stake)
                cons.calc_pending_rewards(validator_rewards.rewards_per_token, delegations)?;
                // HACK temporary work around for proof of concept. Real implementation
                // would use something like a generic Superfluid module to mint or burn
                // synthetic tokens.
                cons.increase_stake(amount)?;
                Ok(cons)
            },
        )?;

        // Update info for the (consumer, validator) map
        // We add the amount delegated to the validator.
        deps = update_delegations(deps, info, &validator, amount, Method::Add)?;

        // Get local denom
        let denom = deps.querier.query_bonded_denom()?;

        // Create message to delegate the underlying tokens
        let msg = StakingMsg::Delegate {
            validator,
            amount: Coin { denom, amount },
        };

        Ok(Response::default().add_message(msg))
    }

    pub fn undelegate(
        mut deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        validator: String,
        amount: Uint128,
    ) -> Result<Response, ContractError> {
        // TODO Validate validator valoper address

        let validator_rewards = VALIDATORS_REWARDS.load(deps.storage, &validator)?;

        // TODO: We check if we have delegation before we check if we have consumer
        // If we don't consumer is not registered, we shouldn't have delegation by default
        // so we return a wrong error to the problem in that case.
        let delegations = VALIDATORS_BY_CONSUMER
            .may_load(deps.storage, (&info.sender, &validator))?
            .ok_or(ContractError::NoDelegationsForValidator {})?;

        // Increase the amount of available funds for that consumer
        CONSUMERS.update(
            deps.storage,
            &info.sender,
            |cons| -> Result<_, ContractError> {
                // fail if consumer was never registered
                let mut cons = cons.ok_or(ContractError::Unauthorized {})?;
                // calculate consumer rewards till now (with old stake)
                cons.calc_pending_rewards(validator_rewards.rewards_per_token, delegations)?;

                // HACK temporary work around for proof of concept. Real implementation
                // would use something like a generic Superfluid module to mint or burn
                // synthetic tokens.
                cons.decrease_stake(amount)?;
                Ok(cons)
            },
        )?;

        // HACK temporary work around for proof of concept. Real implementation
        // would use something like a generic Superfluid module to mint or burn
        // synthetic tokens

        // Update info for the (consumer, validator) map
        // We subtract the amount delegated to the validator.
        deps = update_delegations(deps, info, &validator, amount, Method::Sub)?;

        // Get local denom
        let denom = deps.querier.query_bonded_denom()?;

        // Create message to delegate the underlying tokens
        let msg = StakingMsg::Undelegate {
            validator,
            amount: Coin { denom, amount },
        };

        Ok(Response::default().add_message(msg))
    }

    pub(crate) enum Method {
        Add,
        Sub,
    }

    pub(crate) fn update_delegations<'a>(
        deps: DepsMut<'a>,
        info: MessageInfo,
        validator: &str,
        amount: Uint128,
        method: Method,
    ) -> Result<DepsMut<'a>, ContractError> {
        let action = |validator_info: Option<Uint128>| -> Result<_, ContractError> {
            match method {
                Method::Sub => {
                    let val = validator_info.ok_or(ContractError::NoDelegationsForValidator {})?;
                    val.checked_sub(amount)
                        .map_err(|_| ContractError::InsufficientDelegation {})
                }
                Method::Add => Ok(validator_info.unwrap_or_default() + amount),
            }
        };

        VALIDATORS_BY_CONSUMER.update(deps.storage, (&info.sender, validator), action)?;

        CONSUMERS_BY_VALIDATOR.update(deps.storage, (validator, &info.sender), action)?;

        Ok(deps)
    }

    pub fn withdraw_delegator_reward(
        deps: DepsMut,
        env: Env,
        validator: String,
    ) -> Result<Response, ContractError> {
        // Query fullDelegation to get the total rewards amount
        let delegation_query = deps
            .querier
            .query_delegation(env.contract.address, validator.clone())?;

        // Total rewards we have from this validator
        let total_accumulated_rewards = &match delegation_query {
            Some(delegation) => delegation.accumulated_rewards,
            None => return Err(ContractError::NoDelegationsForValidator {}),
        };

        // Check to make sure there are rewards
        if total_accumulated_rewards.is_empty() || total_accumulated_rewards[0].amount.is_zero() {
            return Err(ContractError::ZeroRewardsToSend {});
        }

        let total_accumulated_rewards = &total_accumulated_rewards[0];

        let total_delegations = CONSUMERS_BY_VALIDATOR
            .prefix(&validator)
            .range(deps.storage, None, None, Order::Ascending)
            .map(|res| -> StdResult<Uint128> { Ok(res?.1) })
            .sum::<StdResult<Uint128>>()?;

        // HACK - better way of saving the denom of rewards?
        REWARDS_DENOM.save(deps.storage, &total_accumulated_rewards.denom)?;

        VALIDATORS_REWARDS.update(
            deps.storage,
            &validator,
            |rewards: Option<ValidatorRewards>| -> Result<_, ContractError> {
                let mut validator_rewards = rewards.unwrap();

                validator_rewards
                    .calc_rewards(total_accumulated_rewards.amount, total_delegations)?;
                Ok(validator_rewards)
            },
        )?;

        // Withdraw rewards from validator
        let withdraw_msg =
            CosmosMsg::Distribution(DistributionMsg::WithdrawDelegatorReward { validator });

        Ok(Response::default().add_message(withdraw_msg))
    }

    pub fn withdraw_to_customer(
        deps: DepsMut,
        _env: Env,
        consumer: String,
        validator: String,
    ) -> Result<Response, ContractError> {
        let consumer_addr = deps.api.addr_validate(&consumer)?;

        if !CONSUMERS.has(deps.storage, &consumer_addr) {
            return Err(ContractError::NoConsumer {});
        };

        let mut consumer = CONSUMERS.load(deps.storage, &consumer_addr)?;
        let validators_rewards = VALIDATORS_REWARDS.load(deps.storage, &validator)?;

        // consumer delegations to this validator
        let delegations =
            VALIDATORS_BY_CONSUMER.load(deps.storage, (&consumer_addr, &validator))?;

        // Do the rewards calculation and update for future calculations
        consumer.calc_pending_rewards(validators_rewards.rewards_per_token, delegations)?;

        if consumer.rewards.pending.floor().is_zero() {
            return Err(ContractError::ZeroRewardsToSend {});
        }

        let rewards_denom = REWARDS_DENOM.load(deps.storage)?;
        let send_amount = consumer.pending_to_u128()?;

        let msg = CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: consumer_addr.to_string(),
            msg: to_binary(&ConsumerExecuteMsg::MeshConsumerRecieveRewardsMsg { validator })?,
            funds: vec![coin(send_amount, rewards_denom)],
        });

        // Save new rewards
        consumer.reset_pending_rewards();
        CONSUMERS.save(deps.storage, &consumer_addr, &consumer)?;

        Ok(Response::default().add_message(msg))
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::AllDelegations { consumer } => {
            query::all_delegations(deps, consumer)
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
    use crate::msg::{Delegation};
    use cosmwasm_std::{to_binary, Addr, Order};
    use cw_storage_plus::Bound;
    use cw_utils::maybe_addr;

    use crate::state::{CONSUMERS, VALIDATORS_BY_CONSUMER};

    use super::*;

    pub fn all_delegations(deps: Deps, consumer: String) -> StdResult<Binary> {
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
            to_binary(&delegations)
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
            .query_balance(env.contract.address, denom)?;

        ensure!(
            contract_balance.amount >= funds_available_for_staking.amount,
            ContractError::NotEnoughFunds {}
        );

        CONSUMERS.save(
            deps.storage,
            &address,
            &ConsumerInfo::new(funds_available_for_staking.amount),
        )?;

        Ok(Response::default())
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

        Ok(Response::default())
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

        Ok(Response::default())
    }
}
