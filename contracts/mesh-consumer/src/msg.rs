use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Decimal;

use crate::state::Config;

#[cw_serde]
pub struct InstantiateMsg {
    pub provider: ProviderInfo,
    pub remote_to_local_exchange_rate: Decimal,
    pub meta_staking_contract_address: String,
    pub ics20_channel: String,
    pub packet_lifetime: Option<u64>,
}

#[cw_serde]
pub struct ProviderInfo {
    pub port_id: String,
    pub connection_id: String,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    // Return configuration info
    #[returns(Config)]
    Config {},
}
