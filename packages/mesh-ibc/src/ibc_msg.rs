use cosmwasm_schema::cw_serde;

use cosmwasm_std::Uint128;

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
        /// The delegator address used as a unique key to save stake on consumer, and to
        /// properly handle ack and timeout messages
        delegator_addr: String,
    },
    Unstake {
        /// Which validator to unstake from
        validator: String,
        /// How much to unstake from this validator
        amount: Uint128,
        /// The delegator address used as a unique key to save stake on consumer, and to
        /// properly handle ack and timeout messages
        delegator_addr: String,
    },
    WithdrawRewards {
        /// Which validator to withdraw from
        validator: String,
        /// The address to send rewards to on the other chain (IbcMsg::Transfer)
        delegator_addr: String,
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
