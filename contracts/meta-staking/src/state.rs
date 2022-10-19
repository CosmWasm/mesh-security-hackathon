use crate::ContractError;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{ensure, Addr, Uint128};
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct Config {
    pub admin: String,
    pub denom: String,
}

pub const CONFIG: Item<Config> = Item::new("config");

pub const CONSUMERS: Map<&Addr, ConsumerInfo> = Map::new("consumers");

#[cw_serde]
pub struct ConsumerInfo {
    // HACK: Funds available for the contract to stake
    // In the future, this could use a generic Superfluid staking module
    // The amount of these funds will limit the voting power
    pub available_funds: Uint128,
    // Total staked funds, cannot stake more than available funds
    pub total_staked: Uint128,
}

impl ConsumerInfo {
    pub fn new(funds: impl Into<Uint128>) -> Self {
        ConsumerInfo {
            available_funds: funds.into(),
            total_staked: Uint128::zero(),
        }
    }

    pub fn increase_stake(&mut self, stake: Uint128) -> Result<(), ContractError> {
        let new_stake = self.total_staked + stake;
        ensure!(
            self.available_funds >= new_stake,
            ContractError::NoFundsToDelegate {}
        );
        self.total_staked = new_stake;
        Ok(())
    }

    pub fn decrease_stake(&mut self, stake: Uint128) -> Result<(), ContractError> {
        self.total_staked = self
            .total_staked
            .checked_sub(stake)
            .map_err(|_| ContractError::InsufficientDelegation {})?;
        Ok(())
    }
}

/// Map<(consumer address, validator address), rewards amount>
pub const VALIDATORS_REWARDS: Map<&str, Uint128> = Map::new("validators_rewards");
pub const REWARDS_DENOM: Item<String> = Item::new("rewards_denom");

/// Map<(consumer address, validator address), Amount>
pub const VALIDATORS_BY_CONSUMER: Map<(&Addr, &str), Uint128> = Map::new("validators_by_consumer");
/// Map<(validator address, consumer address), Amount>
pub const CONSUMERS_BY_VALIDATOR: Map<(&str, &Addr), Uint128> = Map::new("consumers_by_validators");
