#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    ensure_eq, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response, StdResult,
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
            execute::withdraw_delegator_reward(deps, env, info, validator)
        }
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
    use cosmwasm_std::{Coin, DistributionMsg, StakingMsg, SubMsg, Uint128};

    use crate::state::{CONSUMERS, VALIDATORS_BY_CONSUMER};

    use super::*;

    pub fn delegate(
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        validator: String,
        amount: Uint128,
    ) -> Result<Response, ContractError> {
        // TODO Validate validator valoper address

        CONSUMERS.update(
            deps.storage,
            &info.sender,
            |cons| -> Result<_, ContractError> {
                // fail if consumer was never registered
                let mut cons = cons.ok_or(ContractError::Unauthorized {})?;
                // HACK temporary work around for proof of concept. Real implementation
                // would use something like a generic Superfluid module to mint or burn
                // synthetic tokens.
                cons.increase_stake(amount)?;
                Ok(cons)
            },
        )?;

        // Update info for the (consumer, validator) map
        // We add the amount delegated to the validator.
        VALIDATORS_BY_CONSUMER.update(
            deps.storage,
            (&info.sender.to_string(), &validator),
            |validator_info| -> Result<_, ContractError> {
                Ok(validator_info.unwrap_or_default() + amount)
            },
        )?;

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
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        validator: String,
        amount: Uint128,
    ) -> Result<Response, ContractError> {
        // TODO Validate validator valoper address

        // Increase the amount of available funds for that consumer
        CONSUMERS.update(
            deps.storage,
            &info.sender,
            |current| -> Result<_, ContractError> {
                // fail if consumer was never registered
                let mut cur = current.ok_or(ContractError::Unauthorized {})?;
                // HACK temporary work around for proof of concept. Real implementation
                // would use something like a generic Superfluid module to mint or burn
                // synthetic tokens.
                cur.decrease_stake(amount)?;
                Ok(cur)
            },
        )?;

        // HACK temporary work around for proof of concept. Real implementation
        // would use something like a generic Superfluid module to mint or burn
        // synthetic tokens

        // Update info for the (consumer, validator) map
        // We subtract the amount delegated to the validator.
        VALIDATORS_BY_CONSUMER.update(
            deps.storage,
            (&info.sender.to_string(), &validator),
            |validator_info| -> Result<_, ContractError> {
                let val = validator_info.ok_or(ContractError::NoDelegationsForValidator {})?;
                val.checked_sub(amount)
                    .map_err(|_| ContractError::InsufficientDelegation {})
            },
        )?;

        // Get local denom
        let denom = deps.querier.query_bonded_denom()?;

        // Create message to delegate the underlying tokens
        let msg = StakingMsg::Undelegate {
            validator,
            amount: Coin { denom, amount },
        };

        Ok(Response::default().add_message(msg))
    }

    // TODO finish me
    pub fn withdraw_delegator_reward(
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        validator: String,
    ) -> Result<Response, ContractError> {
        // Check this is a consumer calling this, fails if no consumer loads
        CONSUMERS.has(deps.storage, &info.sender);

        // TODO Need to figure out how many rewards we got, so can send them
        // to the consumer contract

        // Withdraw rewards as a submessage
        let withdraw_msg = SubMsg::reply_on_success(
            DistributionMsg::WithdrawDelegatorReward { validator },
            WITHDRAW_REWARDS_REPLY_ID,
        );

        // TODO On reply, send funds to consumer contract

        Ok(Response::default().add_submessage(withdraw_msg))
    }
}

// TODO query this info by consumer...
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::AllDelegations { consumer } => query::all_delegations(deps, consumer),
        QueryMsg::AllValidators { consumer } => query::all_validators(deps, consumer),
        QueryMsg::Consumer { address } => query::consumer(deps, address),
        QueryMsg::Consumers {} => query::consumers(deps),
        QueryMsg::Delegation {
            consumer,
            validator,
        } => query::delegation(deps, consumer, validator),
    }
}

mod query {
    use cosmwasm_std::to_binary;

    use crate::state::CONSUMERS;

    use super::*;

    pub fn all_delegations(_deps: Deps, _consumer: String) -> StdResult<Binary> {
        unimplemented!()
    }

    pub fn all_validators(_deps: Deps, _consumer: String) -> StdResult<Binary> {
        unimplemented!()
    }

    pub fn delegation(_deps: Deps, _consumer: String, _validator: String) -> StdResult<Binary> {
        unimplemented!()
    }

    pub fn consumer(deps: Deps, address: String) -> StdResult<Binary> {
        let addr = deps.api.addr_validate(&address)?;
        let consumer = CONSUMERS.load(deps.storage, &addr)?;
        to_binary(&consumer)
    }

    pub fn consumers(_deps: Deps) -> StdResult<Binary> {
        // let consumers = CONSUMERS.;
        // to_binary(&consumers)
        unimplemented!()
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
            contract_balance.amount <= funds_available_for_staking.amount,
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
        // TODO add explicit method to mesh consumer that will fire off
        // IbcMsg to provider
        let res = parse_reply_execute_data(msg)?;
        println!("{:?}", res);

        Ok(Response::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::coins;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {};
        let info = mock_info("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());
    }
}
