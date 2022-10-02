use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
pub struct InstantiateMsg {
    pub provider: ProviderInfo,
}

#[cw_serde]
pub struct ProviderInfo {
    pub port_id: String,
    pub connection_id: String,
}

#[cw_serde]
pub enum ExecuteMsg {}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {}
