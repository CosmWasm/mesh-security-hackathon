use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Decimal, Uint128};
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct Config {
    pub local_denom: String,
    pub provider_denom: String,
    pub consumer_provider_exchange_rate: Decimal,
}

pub const CONFIG: Item<Config> = Item::new("config");

#[cw_serde]
pub struct ConsumerInfo {
    // The consumer address
    pub address: Addr,
    // HACK: Funds available for the contract to stake
    // In the future, this could use a generic Superfluid staking module
    // The amount of these funds will limit the voting power
    pub available_funds: Uint128,
    // Total staked funds, cannot stake more than available funds
    pub total_staked: Uint128,
}

pub const CONSUMERS: Map<&Addr, ConsumerInfo> = Map::new("consumers");

#[cw_serde]
pub struct ValidatorInfo {
    // The validator address
    pub address: Addr,
    // The consumer
    pub consumer: Addr,
    // Total delegated by the consumer
    pub total_delegated: Uint128,
}

/// Map<(consumer address, validator address), Validator>
pub const VALIDATORS_BY_CONSUMER: Map<(Addr, Addr), ValidatorInfo> =
    Map::new("validators_by_consumer");
