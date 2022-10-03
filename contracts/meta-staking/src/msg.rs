use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{
    AllDelegationsResponse, AllValidatorsResponse, BondedDenomResponse, Coin, Decimal,
    DelegationResponse, ValidatorResponse,
};

use crate::state::Config;

#[cw_serde]
pub struct InstantiateMsg {
    pub local_denom: String,
    pub provider_denom: String,
    pub consumer_provider_exchange_rate: Decimal,
}

#[cw_serde]
pub enum ExecuteMsg {
    /// This is translated to a [MsgDelegate](https://github.com/cosmos/cosmos-sdk/blob/v0.40.0/proto/cosmos/staking/v1beta1/tx.proto#L81-L90).
    /// `delegator_address` is automatically filled with the current contract's address.
    Delegate { validator: String, amount: Coin },
    /// This is translated to a [MsgUndelegate](https://github.com/cosmos/cosmos-sdk/blob/v0.40.0/proto/cosmos/staking/v1beta1/tx.proto#L112-L121).
    /// `delegator_address` is automatically filled with the current contract's address.
    Undelegate { validator: String, amount: Coin },
    /// This is translated to a [[MsgWithdrawDelegatorReward](https://github.com/cosmos/cosmos-sdk/blob/v0.42.4/proto/cosmos/distribution/v1beta1/tx.proto#L42-L50).
    /// `delegator_address` is automatically filled with the current contract's address.
    WithdrawDelegatorReward {
        /// The `validator_address`
        validator: String,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Returns the denomination that can be bonded (if there are multiple native tokens on the chain)
    #[returns(BondedDenomResponse)]
    BondedDenom {},
    /// AllDelegations will return all delegations by the delegator
    #[returns(AllDelegationsResponse)]
    AllDelegations { delegator: String },
    /// Returns meta-staking config
    #[returns(Config)]
    Config {},
    /// Delegation will return more detailed info on a particular
    /// delegation, defined by delegator/validator pair
    #[returns(DelegationResponse)]
    Delegation {
        delegator: String,
        validator: String,
    },
    /// Returns all validators in the currently active validator set.
    ///
    /// The query response type is `AllValidatorsResponse`.
    #[returns(AllValidatorsResponse)]
    AllValidators {},
    /// Returns the validator at the given address. Returns None if the validator is
    /// not part of the currently active validator set.
    ///
    /// The query response type is `ValidatorResponse`.
    #[returns(ValidatorResponse)]
    Validator {
        /// The validator's address (e.g. (e.g. cosmosvaloper1...))
        address: String,
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
