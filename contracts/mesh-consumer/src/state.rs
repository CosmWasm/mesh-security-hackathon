use cosmwasm_schema::cw_serde;
use cosmwasm_std::Decimal;
use cw_storage_plus::Item;

use crate::msg::ProviderInfo;

#[cw_serde]
pub struct Config {
    pub provider: ProviderInfo,
    pub remote_to_local_exchange_rate: Decimal,
    pub meta_staking_contract_address: String,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const CHANNEL: Item<String> = Item::new("channel");
