use cosmwasm_schema::cw_serde;
use cw_storage_plus::Item;

use crate::msg::ProviderInfo;

#[cw_serde]
pub struct Config {
    pub provider: ProviderInfo,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const CHANNEL: Item<String> = Item::new("channel");
