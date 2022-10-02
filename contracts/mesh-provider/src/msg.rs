use serde::Serialize;

use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{to_binary, Binary, Decimal, StdResult};

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
    Slash { validator: String, amount: Decimal },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ConfigResponse)]
    Config {},
}

#[cw_serde]
pub struct ConfigResponse {
    pub consumer: ConsumerInfo,
    pub slasher: Option<String>,
}
