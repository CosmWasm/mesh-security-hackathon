use cosmwasm_schema::cw_serde;

use cosmwasm_std::Uint128;

/// These are messages sent from the provider to the consumer
#[cw_serde]
pub enum ProviderMsg {
    /// Returns the current validator set. The first time this is called, it will
    /// initiate a stream of `UpdateValidator` messages to be sent through the channel.
    ListValidators {},
    Stake {
        /// How much to stake to which validator
        validators: Vec<ValidatorAmount>,
        /// A unique key for this request set by the caller, to be used to
        /// properly handle ack and timeout messages
        key: u64,
    },
    Unstake {
        /// How much to unstake from which validator
        validators: Vec<ValidatorAmount>,
        /// A unique key for this request set by the caller, to be used to
        /// properly handle ack and timeout messages
        key: u64,
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
        // TODO: what info do we sent??
    },
}

/// Simple struct with a validator and an amount used a few places.
/// denom is defined in the channel, doesn't need to be every message.
#[cw_serde]
pub struct ValidatorAmount {
    pub validator: String,
    pub amount: Uint128,
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
