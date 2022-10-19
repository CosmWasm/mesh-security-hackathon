use serde::Serialize;

use crate::state::ValStatus;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{to_binary, Binary, Decimal, StdResult, Uint128};

#[cw_serde]
pub struct InstantiateMsg {
    pub consumer: ConsumerInfo,
    /// data to instantiate the slasher
    pub slasher: SlasherInfo,
    /// Address of Lockup contract from which we accept ReceiveClaim
    pub lockup: String,
    /// Unbonding period of the remote chain in seconds
    pub unbonding_period: u64,
    /// IBC denom string - "port_id/channel_id/denom"
    pub rewards_ibc_denom: String,
}

#[cw_serde]
pub struct ConsumerInfo {
    /// We can add port later if we have it, for now, just assert the chain we talk with
    pub connection_id: String,
}

#[cw_serde]
pub struct SlasherInfo {
    pub code_id: u64,
    pub msg: Binary,
}

impl SlasherInfo {
    pub fn new<T: Serialize>(code_id: u64, msg: &T) -> StdResult<Self> {
        Ok(SlasherInfo {
            code_id,
            msg: to_binary(msg)?,
        })
    }
}

#[cw_serde]
pub enum ExecuteMsg {
    Slash {
        /// which validator to slash
        validator: String,
        /// what percentage we should slash all stakers
        percentage: Decimal,
        /// do we forcibly unbond this validator on the provider side,
        /// regardless of the behavior of the consumer?
        force_unbond: bool,
    },
    /// This gives the receiver access to slash part up to this much claim
    ReceiveClaim {
        owner: String,
        amount: Uint128,
        validator: String,
    },
    /// Triggers the unbonding period for your staked tokens
    Unstake {
        amount: Uint128,
        validator: String,
    },
    /// Called after unbonding_period has passed from Unstake. Releases claim on lockup contract
    Unbond {/* ??? */},
    ClaimRewards {
        validator: String
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ConfigResponse)]
    Config {},
    /// how much this account has staked where
    #[returns(AccountResponse)]
    Account { address: String },
    /// Details of one validator
    #[returns(ValidatorResponse)]
    Validator { address: String },
    /// Details of one validator
    #[returns(ListValidatorsResponse)]
    ListValidators {
        start_after: Option<String>,
        limit: Option<u32>,
    },
}

#[cw_serde]
pub struct ConfigResponse {
    pub consumer: ConsumerInfo,
    pub slasher: Option<String>,
}

#[cw_serde]
pub struct AccountResponse {
    pub staked: Vec<StakeInfo>,
}

#[cw_serde]
pub struct StakeInfo {
    pub validator: String,
    pub tokens: Uint128,
    pub slashed: Uint128,
}

#[cw_serde]
pub struct ValidatorResponse {
    pub address: String,
    pub tokens: Uint128,
    pub status: ValStatus,
    pub multiplier: Decimal,
}

#[cw_serde]
pub struct ListValidatorsResponse {
    pub validators: Vec<ValidatorResponse>,
}
