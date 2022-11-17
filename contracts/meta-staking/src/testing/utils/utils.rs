use cosmwasm_std::{testing::mock_env, Addr, Coin, Decimal, Uint128, Validator};
use cw_multi_test::{next_block, App, AppBuilder, BankSudo, StakingInfo, SudoMsg};

use super::super::utils::{NATIVE_DENOM, USER, VALIDATOR};
use super::instantiate::instantiate_meta_staking;

pub fn mock_app() -> App {
    let env = mock_env();
    AppBuilder::new().build(|router, api, storage| {
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

        router
            .bank
            .init_balance(
                storage,
                &Addr::unchecked(USER.addr()),
                vec![Coin {
                    denom: NATIVE_DENOM.to_string(),
                    amount: Uint128::new(1),
                }],
            )
            .unwrap();
    })
}

pub fn instantiate_setup() -> (App, Addr) {
    let mut app = mock_app();

    // Init meta staking contract
    let meta_staking_addr = instantiate_meta_staking(&mut app, None);

    // Gov funds meta-staking contract
    // This is a workaround until we have superfluid staking
    app.sudo(SudoMsg::Bank(BankSudo::Mint {
        to_address: meta_staking_addr.to_string(),
        amount: vec![Coin {
            amount: Uint128::new(100000000),
            denom: NATIVE_DENOM.to_string(),
        }],
    }))
    .unwrap();

    app.update_block(next_block);

    (app, meta_staking_addr)
}
