use cosmwasm_std::{Addr, Coin, Uint128};
use cw_multi_test::{App, AppBuilder};
use mesh_testing::{
    addr,
    constants::{CREATOR_ADDR, NATIVE_DENOM},
    instantiates::instantiate_mesh_provider,
};

pub fn setup_app() -> App {
    AppBuilder::new().build(|router, _, storage| {
        router
            .bank
            .init_balance(
                storage,
                &addr!(CREATOR_ADDR),
                vec![Coin {
                    denom: NATIVE_DENOM.to_string(),
                    amount: Uint128::new(1),
                }],
            )
            .unwrap();
    })
}

pub fn setup_with_contract() -> (App, Addr) {
    let mut app = setup_app();

    let mesh_provider_addr = instantiate_mesh_provider(&mut app, None);

    (app, mesh_provider_addr)
}
