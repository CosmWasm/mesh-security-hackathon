use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
pub struct InstantiateMsg {
    pub consumer: ConsumerInfo,
}

#[cw_serde]
pub struct ConsumerInfo {
    /// We can add port later if we have it, for now, just assert the chain we talk with
    pub connection_id: String,
}

#[cw_serde]
pub enum ExecuteMsg {}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {}
