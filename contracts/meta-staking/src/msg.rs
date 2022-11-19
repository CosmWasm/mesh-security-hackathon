use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Coin, Uint128, Addr};

use crate::state::ConsumerInfo;

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {
    /// This is translated to a [MsgDelegate](https://github.com/cosmos/cosmos-sdk/blob/v0.40.0/proto/cosmos/staking/v1beta1/tx.proto#L81-L90).
    /// `delegator_address` is automatically filled with the current contract's address.
    Delegate {
        validator: String,
        amount: Uint128,
    },
    /// This is translated to a [MsgUndelegate](https://github.com/cosmos/cosmos-sdk/blob/v0.40.0/proto/cosmos/staking/v1beta1/tx.proto#L112-L121).
    /// `delegator_address` is automatically filled with the current contract's address.
    Undelegate {
        validator: String,
        amount: Uint128,
    },
    /// This is translated to a [[MsgWithdrawDelegatorReward](https://github.com/cosmos/cosmos-sdk/blob/v0.42.4/proto/cosmos/distribution/v1beta1/tx.proto#L42-L50).
    /// `delegator_address` is automatically filled with the current contract's address.
    WithdrawDelegatorReward {
        /// The `validator_address`
        validator: String,
    },
    WithdrawToCostumer {
        consumer: String,
        validator: String,
    },
    /// Use for now, only admin can call - later we can remove if x/gov calls SudoMsg directly
    Sudo(SudoMsg),
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// AllDelegations will return all delegations by the consumer
    #[returns(Vec<Delegation>)]
    AllDelegations { consumer: String },
    /// Returns an individual consumer
    #[returns(ConsumerInfo)]
    Consumer { address: String },
    /// Returns list of consumers
    #[returns(Vec<Addr>)]
    Consumers {
        start: Option<String>,
        limit: Option<u32>,
    },
    /// Delegation will return more detailed info on a particular
    /// delegation, defined by delegator/validator pair
    #[returns(Uint128)]
    Delegation { consumer: String, validator: String },
    /// Returns all validators the consumer delegates to.
    ///
    /// The query response type is `AllValidatorsResponse`.
    #[returns(Vec<String>)]
    AllValidators {
        consumer: String,
        start: Option<String>,
        limit: Option<u32>,
    },
}

#[cw_serde]
pub enum SudoMsg {
    /// HACK temporary workaround for the proof of concepy.
    /// Governance will fund the meta-staking contract with native tokens.
    /// In production, this would be accomplished by something like
    /// a generic version of the Superfluid staking module which would
    /// be in charge of minting and burning synthetic tokens.
    /// Update list of consumers
    AddConsumer {
        consumer_address: String,
        funds_available_for_staking: Coin,
    },
    RemoveConsumer {
        consumer_address: String,
    },
}

#[cw_serde]
pub struct Delegation {
    pub validator: String,
    pub delegation: Uint128,
}
