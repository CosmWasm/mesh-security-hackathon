#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, SudoMsg};
use crate::state::{Config, CONFIG};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:meta-staking";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    // Save config
    CONFIG.save(
        deps.storage,
        &Config {
            local_denom: msg.local_denom,
            provider_denom: msg.provider_denom,
            consumer_provider_exchange_rate: msg.consumer_provider_exchange_rate,
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
    }
}

mod execute {
    use cosmwasm_std::{Coin, DistributionMsg, StakingMsg, SubMsg};

    use crate::state::{ConsumerInfo, ValidatorInfo, CONSUMERS, VALIDATORS_BY_CONSUMER};

    use super::*;

    pub fn delegate(
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        validator: String,
        amount: Coin,
    ) -> Result<Response, ContractError> {
        //// TODO converion rate? What to do if it changes?
        // Check denom
        let config = CONFIG.load(deps.storage)?;
        if amount.denom != config.local_denom {
            return Err(ContractError::IncorrectDenom {});
        }

        // Check this is a consumer calling this, fails if no consumer loads
        let ConsumerInfo {
            address: consumer_addr,
            available_funds,
            total_staked,
        } = CONSUMERS.load(deps.storage, &info.sender)?;

        // Validate validator address
        let validator_addr = deps.api.addr_validate(&validator)?;

        // HACK: We have the budget for each consumer funded by the x/gov module.
        // A much better solution would be a generic liquid staking module.
        // This is intended only for proof of concept
        //
        // Check consumer chain has available budget to delegate
        if available_funds + amount.amount > total_staked {
            return Err(ContractError::NoFundsToDelegate {});
        }

        // HACK temporary work around for proof of concept. Real implementation
        // would use something like a generic Superfluid module to mint or burn
        // synthetic tokens.

        // Update info for the (consumer, validator) map
        // We add the amount delegated to the validator.
        VALIDATORS_BY_CONSUMER.update(
            deps.storage,
            (consumer_addr.clone(), validator_addr.clone()),
            |validator_info| -> Result<ValidatorInfo, ContractError> {
                match validator_info {
                    Some(validator_info) => Ok(ValidatorInfo {
                        address: validator_info.address,
                        consumer: validator_info.consumer,
                        total_delegated: validator_info.total_delegated + amount.amount,
                    }),
                    // No one has delegated to this validator, we save the info
                    // Initial amount is this delegation
                    None => Ok(ValidatorInfo {
                        address: validator_addr,
                        consumer: consumer_addr,
                        total_delegated: amount.amount,
                    }),
                }
            },
        )?;

        // Subtract amount of available funds for that consumer
        CONSUMERS.update(deps.storage, &info.sender, |current| match current {
            Some(current) => Ok(ConsumerInfo {
                address: current.address,
                available_funds: available_funds - amount.amount,
                total_staked: current.total_staked,
            }),
            None => Err(ContractError::Unauthorized {}),
        })?;

        // Create message to delegate the underlying tokens
        let msg = StakingMsg::Delegate { validator, amount };

        Ok(Response::default().add_message(msg))
    }

    pub fn undelegate(
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        validator: String,
        amount: Coin,
    ) -> Result<Response, ContractError> {
        //// TODO converion rate?
        // Check denom
        let config = CONFIG.load(deps.storage)?;
        if amount.denom != config.local_denom {
            return Err(ContractError::IncorrectDenom {});
        }

        // Check this is a consumer calling this, fails if no consumer loads
        let ConsumerInfo {
            address: consumer_addr,
            available_funds,
            total_staked: _,
        } = CONSUMERS.load(deps.storage, &info.sender)?;

        // Validate validator address
        let validator_addr = deps.api.addr_validate(&validator)?;

        // HACK temporary work around for proof of concept. Real implementation
        // would use something like a generic Superfluid module to mint or burn
        // synthetic tokens

        // Update info for the (consumer, validator) map
        // We subtract the amount delegated to the validator.
        VALIDATORS_BY_CONSUMER.update(
            deps.storage,
            (consumer_addr.clone(), validator_addr.clone()),
            |validator_info| -> Result<ValidatorInfo, ContractError> {
                match validator_info {
                    Some(validator_info) => Ok(ValidatorInfo {
                        address: validator_info.address,
                        consumer: validator_info.consumer,
                        total_delegated: validator_info.total_delegated - amount.amount,
                    }),
                    // No one has delegated to this validator, throw error
                    None => Err(ContractError::NoDelegationsForValidator {}),
                }
            },
        )?;
        // Increase the amount of available funds for that consumer
        CONSUMERS.update(deps.storage, &info.sender, |current| match current {
            Some(current) => Ok(ConsumerInfo {
                address: current.address,
                available_funds: available_funds + amount.amount,
                total_staked: current.total_staked,
            }),
            None => Err(ContractError::Unauthorized {}),
        })?;

        // Create message to delegate the underlying tokens
        let msg = StakingMsg::Undelegate { validator, amount };

        Ok(Response::default().add_message(msg))
    }

    pub fn withdraw_delegator_reward(
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        validator: String,
    ) -> Result<Response, ContractError> {
        // Check this is a consumer calling this, fails if no consumer loads
        CONSUMERS.has(deps.storage, &info.sender);

        // TODO make sure can't consumer can't withdraw more than its share of rewards?

        // Withdraw rewards as a submessage
        let sub_msg = SubMsg::new(DistributionMsg::WithdrawDelegatorReward { validator });

        // TODO send funds to the consumer contract? Perhaps send directly to provider contract

        Ok(Response::default().add_submessage(sub_msg))
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::AllDelegations { delegator } => query::all_delegations(deps, delegator),
        QueryMsg::AllValidators {} => query::all_validators(deps),
        QueryMsg::BondedDenom {} => query::bonded_denom(deps),
        QueryMsg::Delegation {
            delegator,
            validator,
        } => query::delegation(deps, delegator, validator),
        QueryMsg::Validator { address } => query::validator(deps, address),
    }
}

mod query {
    use cosmwasm_std::to_binary;

    use super::*;

    pub fn all_delegations(deps: Deps, delegator: String) -> StdResult<Binary> {
        let all_delegations = deps.querier.query_all_delegations(delegator)?;
        to_binary(&all_delegations)
    }

    pub fn all_validators(deps: Deps) -> StdResult<Binary> {
        let all_validators = deps.querier.query_all_validators()?;
        to_binary(&all_validators)
    }

    pub fn bonded_denom(deps: Deps) -> StdResult<Binary> {
        let bonded_denom = deps.querier.query_bonded_denom()?;
        to_binary(&bonded_denom)
    }

    pub fn delegation(deps: Deps, delegator: String, validator: String) -> StdResult<Binary> {
        let delegation = deps.querier.query_delegation(delegator, validator)?;
        to_binary(&delegation)
    }

    pub fn validator(deps: Deps, address: String) -> StdResult<Binary> {
        let validator = deps.querier.query_validator(address)?;
        to_binary(&validator)
    }

    pub fn config(deps: Deps) -> StdResult<Binary> {
        let config = CONFIG.load(deps.storage)?;
        to_binary(&config)
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn sudo(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: SudoMsg,
) -> Result<Response, ContractError> {
    match msg {
        SudoMsg::Fund { consumer } => sudo::fund(deps, env, info, consumer),
        SudoMsg::UpdateConsumers { to_add, to_remove } => {
            sudo::update_consumers(deps, env, info, to_add, to_remove)
        }
    }
}

mod sudo {
    use cosmwasm_std::Uint128;

    use crate::state::{ConsumerInfo, CONSUMERS};

    use super::*;

    pub fn fund(
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        _consumer: String,
    ) -> Result<Response, ContractError> {
        let config = CONFIG.load(deps.storage)?;

        // Check this is a consumer calling this, fails if no consumer loads
        CONSUMERS.has(deps.storage, &info.sender);

        // TODO there has to be a better way to do this but I am tired
        let mut amount = Uint128::zero();
        for coin in info.funds {
            // Check denom matches bonded denom
            if coin.denom == config.local_denom {
                amount = coin.amount;
            }
        }
        if amount == Uint128::zero() {
            return Err(ContractError::IncorrectDenom {});
        }

        // Increase the amount of available funds for that consumer
        CONSUMERS.update(deps.storage, &info.sender, |current| match current {
            Some(current) => Ok(ConsumerInfo {
                address: current.address,
                available_funds: amount,
                total_staked: current.total_staked,
            }),
            None => Err(ContractError::Unauthorized {}),
        })?;

        Ok(Response::default())
    }

    pub fn update_consumers(
        deps: DepsMut,
        _env: Env,
        _info: MessageInfo,
        to_add: Option<Vec<String>>,
        to_remove: Option<Vec<String>>,
    ) -> Result<Response, ContractError> {
        // Remove consumers
        if let Some(to_remove) = to_remove {
            for addr in to_remove {
                let addr = deps.api.addr_validate(&addr)?;
                CONSUMERS.remove(deps.storage, &addr);
            }
        }

        if let Some(to_add) = to_add {
            // Add consumers
            for addr in to_add {
                let address = deps.api.addr_validate(&addr)?;
                CONSUMERS.save(
                    deps.storage,
                    &address,
                    &ConsumerInfo {
                        // The address of the consumer contract
                        address: address.clone(),
                        // Consumers start with zero until they are funded
                        available_funds: Uint128::zero(),
                        // Zero until funds are delegated
                        total_staked: Uint128::zero(),
                    },
                )?;
            }
        }

        Ok(Response::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, Decimal};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {
            local_denom: "ujuno".to_string(),
            provider_denom: "uosmo".to_string(),
            consumer_provider_exchange_rate: Decimal::percent(10),
        };
        let info = mock_info("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());
    }
}
