use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Uint128, Coin};

#[cw_serde]
pub enum StakingExecuteMsg {
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
    Sudo(StakingSudoMsg),
}

#[cw_serde]
pub enum StakingSudoMsg {
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
