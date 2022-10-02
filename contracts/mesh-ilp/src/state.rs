use cosmwasm_std::{Addr, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cw_storage_plus::{Item, Map};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub denom: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Balance {
    pub bonded: Uint128,
    pub claims: Vec<LeinAddr>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct LeinAddr {
    pub leinholder: Addr,
    pub amount: Uint128,
}

impl Balance {
    pub fn free(&self) -> Uint128 {
        let claimed = self
            .claims
            .iter()
            .map(|l| l.amount)
            .max()
            .unwrap_or_default();
        self.bonded - claimed
    }
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const BALANCES: Map<&Addr, Balance> = Map::new("balances");
