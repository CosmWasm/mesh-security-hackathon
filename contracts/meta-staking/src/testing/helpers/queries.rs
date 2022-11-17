use cosmwasm_std::{Addr, Coin, Uint128};
use cw_multi_test::App;

use crate::{msg::QueryMsg, state::ConsumerInfo};

pub fn query_module_rewards(app: &mut App, meta_staking_addr: &Addr, validator: &Addr) -> Coin {
    app.read_module(|router, _api, storage| {
        //let rewards = router.distribution.remove_rewards(api,  storage, &app.block_info(), &CONSUMER.addr(), &VALIDATOR.addr()).unwrap();
        router
            .staking
            .get_rewards(storage, &app.block_info(), &meta_staking_addr, validator)
            .unwrap()
            .unwrap()
    })
}

pub fn query_delegation(
    app: &mut App,
    meta_staking_addr: &Addr,
    consumer: &Addr,
    validator: &Addr,
) -> Uint128 {
    app.wrap()
        .query_wasm_smart(
            meta_staking_addr,
            &QueryMsg::Delegation {
                consumer: consumer.to_string(),
                validator: validator.to_string(),
            },
        )
        .unwrap()
}

pub fn query_consumer(app: &mut App, meta_staking_addr: &Addr, consumer: &Addr) -> ConsumerInfo {
    app.wrap()
        .query_wasm_smart(
            meta_staking_addr,
            &QueryMsg::Consumer {
                address: consumer.to_string(),
            },
        )
        .unwrap()
}
