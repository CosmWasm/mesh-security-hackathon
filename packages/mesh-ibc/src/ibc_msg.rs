use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::Uint128;

/// These are messages sent from the provider to the consumer
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ProviderMsg {
    /// Returns the current validator set. The first time this is called, it will
    /// initiate a stream of `UpdateValidator` messages to be sent through the channel.
    ListValidators {},
    Stake {
        /// How much to stake to which validator
        validators: Vec<ValidatorAmount>,
        /// A unique key for this request set by the caller, to be used to
        /// properly handle ack and timeout messages
        key: String,
    },
    Unstake {
        /// How much to unstake from which validator
        validators: Vec<ValidatorAmount>,
        /// A unique key for this request set by the caller, to be used to
        /// properly handle ack and timeout messages
        key: String,
    },
}

/// These are messages sent from the consumer to the provider
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
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
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ValidatorAmount {
    pub validator: String,
    pub amount: Uint128,
}

/// List the current validator set.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ListValidatorsResponse {
    pub validators: Vec<String>,
}

/// TODO: any data we want when incrementing stake
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct StakeResponse {}

/// TODO: any data we want when decrementing stake
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UnstakeResponse {}

/// This is an event stream, doesn't ever get a response, but we include it here for clarity
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UpdateValidatorsResponse {}

/// TODO: any data we want after delivering rewards
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct RewardsResponse {}
