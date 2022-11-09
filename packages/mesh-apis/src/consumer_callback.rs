use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Coin, Uint128};

#[cw_serde]
pub struct CallbackDataResponse {
    pub validator: String,
    pub staker: String,
    pub stake_amount: Uint128,
    pub rewards: Coin,
}
