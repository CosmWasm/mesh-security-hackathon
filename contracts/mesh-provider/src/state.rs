use cosmwasm_std::Addr;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cw_storage_plus::Item;

use crate::msg::ConsumerInfo;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub consumer: ConsumerInfo,
    pub slasher: Option<Addr>,
    /// Address of Lockup contract from which we accept ReceiveClaim
    pub lockup: Addr,
    /// Unbonding period of the remote chain in seconds
    pub unbonding_period: u64,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const CHANNEL: Item<String> = Item::new("channel");
