use cosmwasm_schema::cw_serde;

use cosmwasm_std::{Coin, Uint128};

/// These are messages sent from the provider to the consumer
#[cw_serde]
pub enum ProviderMsg {
    /// Returns the current validator set. The first time this is called, it will
    /// initiate a stream of `UpdateValidator` messages to be sent through the channel.
    ListValidators {},
    Stake {
        /// Which validator to stake to
        validator: String,
        /// How much to stake with this validator
        amount: Uint128,
        /// A unique key for this request set by the caller, to be used to
        /// properly handle ack and timeout messages (not used by consumer)
        key: String,
    },
    Unstake {
        /// Which validator to unstake from
        validator: String,
        /// How much to unstake from this validator
        amount: Uint128,
        /// A unique key for this request set by the caller, to be used to
        /// properly handle ack and timeout messages (not used by consumer)
        key: String,
    },
}

/// These are messages sent from the consumer to the provider
#[cw_serde]
pub enum ConsumerMsg {
    /// Shows a diffs of valid addresses to stake to, based on changes in the active set.
    /// Calling ListValidators and adding every UpdateValidators call will gives you the
    /// full set of current validators that can be staked to.
    UpdateValidators {
        added: Vec<String>,
        removed: Vec<String>,
    },
    Rewards {
        validator: String,
        total_funds: Coin,
    },
}

/// List the current validator set.
#[cw_serde]
pub struct ListValidatorsResponse {
    pub validators: Vec<String>,
}

/// TODO: any data we want when incrementing stake
#[cw_serde]
pub struct StakeResponse {}

/// TODO: any data we want when decrementing stake
#[cw_serde]
pub struct UnstakeResponse {}

/// This is an event stream, doesn't ever get a response, but we include it here for clarity
#[cw_serde]
pub struct UpdateValidatorsResponse {}

/// TODO: any data we want after delivering rewards
#[cw_serde]
pub struct RewardsResponse {}
