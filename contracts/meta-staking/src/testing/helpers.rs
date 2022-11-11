use cosmwasm_schema::cw_serde;
use cosmwasm_std::{testing::mock_env, to_binary, Addr, Coin, Decimal, Uint128, Validator};
use cw_multi_test::{next_block, App, AppBuilder, BankSudo, SudoMsg, WasmSudo, StakingInfo};

use super::{instantiate::instantiate_meta_staking, CONSUMER, NATIVE_DENOM, USER, VALIDATOR, CONSUMER2};
use crate::msg::SudoMsg as MetaStakingSudoMsg;

#[cw_serde]
pub struct AddrHelper<'a>(pub &'a str);

impl<'a> AddrHelper<'a> {
    pub const fn new(addr: &'a str) -> Self {
        AddrHelper(addr)
    }

    pub fn addr(&self) -> Addr {
        Addr::unchecked(self.0.clone())
    }

    pub fn to_string(&self) -> String {
        self.0.clone().to_string()
    }
}

pub fn mock_app() -> App {
    let env = mock_env();
    AppBuilder::new().build(|router, api, storage| {
        router.staking.setup(storage, StakingInfo {
            bonded_denom: NATIVE_DENOM.to_string(),
            unbonding_time: 1,
            apr: Decimal::percent(10),
        }).unwrap();

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

    // Gov adds consumer
    app.sudo(SudoMsg::Wasm(WasmSudo {
        contract_addr: meta_staking_addr.clone(),
        msg: to_binary(&MetaStakingSudoMsg::AddConsumer {
            consumer_address: CONSUMER.addr().to_string(),
            funds_available_for_staking: Coin {
                denom: NATIVE_DENOM.to_string(),
                amount: Uint128::new(100000000),
            },
        })
        .unwrap(),
    }))
    .unwrap();

    app.sudo(SudoMsg::Wasm(WasmSudo {
        contract_addr: meta_staking_addr.clone(),
        msg: to_binary(&MetaStakingSudoMsg::AddConsumer {
            consumer_address: CONSUMER2.addr().to_string(),
            funds_available_for_staking: Coin {
                denom: NATIVE_DENOM.to_string(),
                amount: Uint128::new(100000000),
            },
        })
        .unwrap(),
    }))
    .unwrap();

    (app, meta_staking_addr)
}
