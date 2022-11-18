use cosmwasm_std::{coin, Coin};
use mesh_testing::NATIVE_DENOM;

use super::setup_app::AppWrapperType;

pub fn query_rewards(app_wrapper: &AppWrapperType, addr: String, validator: String) -> Coin {
    app_wrapper
        .module_querier()
        .query_delegation(addr, validator)
        .unwrap()
        .unwrap()
        .accumulated_rewards
        .first()
        .unwrap()
        .clone()
}

pub fn query_rewards_expect_empty(
    app_wrapper: &AppWrapperType,
    addr: String,
    validator: String,
) -> Coin {
    let rewards = app_wrapper
        .module_querier()
        .query_delegation(addr, validator)
        .unwrap()
        .unwrap()
        .accumulated_rewards;

    if rewards.is_empty() {
        return coin(0, NATIVE_DENOM);
    } else {
        return rewards[0].clone();
    }
}
