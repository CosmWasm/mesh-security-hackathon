use cosmwasm_std::Addr;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cw_storage_plus::Item;

use crate::msg::ConsumerInfo;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub consumer: ConsumerInfo,
    pub slasher: Option<Addr>,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const CHANNEL: Item<String> = Item::new("channel");
