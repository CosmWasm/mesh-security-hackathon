use serde::Serialize;

use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{to_binary, Binary, Decimal, StdResult, Uint128};

#[cw_serde]
pub struct InstantiateMsg {
    pub consumer: ConsumerInfo,
    // data for the slasher to instantiate
    pub slasher: SlasherInfo,
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
    // TODO: add some way to slash a claim if a lein was slashed somewhere else?
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ConfigResponse)]
    Config {},
    /// how much this account has staked where
    #[returns(AccountResponse)]
    Account { address: String },
    /// how much power each validator has received
    #[returns(ValidatorPowerResponse)]
    ValidatorPower {},
}

#[cw_serde]
pub struct ConfigResponse {
    pub consumer: ConsumerInfo,
    pub slasher: Option<String>,
}

#[cw_serde]
pub struct AccountResponse {
    // TODO
}

#[cw_serde]
pub struct ValidatorPowerResponse {
    // TODO
}
