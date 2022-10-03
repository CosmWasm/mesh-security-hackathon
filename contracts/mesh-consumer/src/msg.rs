use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
pub struct InstantiateMsg {
    pub provider: ProviderInfo,
    // TODO: add remote_to_local_exchange_rate (Decimal)
    // TODO: add mesh-staking contract address
    // (Note: we may need to start updating deploy script in integration.rs)
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
pub enum QueryMsg {
    // TODO: add config info
}
