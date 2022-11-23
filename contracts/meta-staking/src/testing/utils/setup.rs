use cosmwasm_std::{coins, testing::mock_env, Addr, Decimal, Validator};
use cw_multi_test::{App, AppBuilder, BankSudo, StakingInfo, SudoMsg};

use mesh_testing::{
    constants::CREATOR_ADDR,
    instantiates::{instantiate_mesh_consumer, instantiate_meta_staking},
};

use super::executes::add_consumer;

const NATIVE_DENOM: &str = "vative_denom";
const VALIDATOR: &str = "validator";

pub fn setup_app() -> App {
    AppBuilder::new().build(|router, api, storage| {
        let env = mock_env();

        // Setup staking module for the correct mock data.
        router
            .staking
            .setup(
                storage,
                StakingInfo {
                    bonded_denom: NATIVE_DENOM.to_string(),
                    unbonding_time: 1,
                    apr: Decimal::percent(10),
                },
            )
            .unwrap();

        // Add mock validator
        router
            .staking
            .add_validator(
                api,
                storage,
                &env.block,
                Validator {
                    address: VALIDATOR.to_string(),
                    commission: Decimal::zero(),
                    max_commission: Decimal::one(),
                    max_change_rate: Decimal::one(),
                },
            )
            .unwrap();
    })
}

pub fn setup_with_contracts() -> (App, Addr, Addr) {
    let mut app = setup_app();

    let meta_staking_addr = instantiate_meta_staking(&mut app, None);
    // Fund meta-staking
    app.sudo(SudoMsg::Bank(BankSudo::Mint {
        to_address: meta_staking_addr.to_string(),
        amount: coins(100000, NATIVE_DENOM),
    }))
    .unwrap();

    let mesh_consumer_addr =
        instantiate_mesh_consumer(&mut app, None, Some(meta_staking_addr.clone()));

    (app, meta_staking_addr, mesh_consumer_addr)
}

pub fn setup_with_consumer() -> (App, Addr, Addr) {
    let (mut app, meta_staking_addr, mesh_consumer_addr) = setup_with_contracts();

    add_consumer(
        &mut app,
        meta_staking_addr.as_str(),
        CREATOR_ADDR,
        mesh_consumer_addr.as_str(),
        10000,
    )
    .unwrap();

    (app, meta_staking_addr, mesh_consumer_addr)
}
