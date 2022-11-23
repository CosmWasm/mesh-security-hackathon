use cosmwasm_std::{testing::mock_env, Addr, Decimal, Validator, coins};
use cw_multi_test::{App, AppBuilder, StakingInfo, SudoMsg, BankSudo};

use crate::testing::{NATIVE_DENOM, VALIDATOR};
use mesh_testing::instantiates::{instantiate_mesh_consumer, instantiate_meta_staking};

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

    let mesh_consumer_addr = instantiate_mesh_consumer(&mut app, None, meta_staking_addr.clone());

    (app, meta_staking_addr, mesh_consumer_addr)
}
