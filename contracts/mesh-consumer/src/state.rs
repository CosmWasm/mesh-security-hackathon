use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cw_storage_plus::Item;

use crate::msg::ProviderInfo;

#[cw_serde]
pub struct Config {
    pub owner: Addr,
    pub provider: ProviderInfo,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const CHANNEL: Item<String> = Item::new("channel");
