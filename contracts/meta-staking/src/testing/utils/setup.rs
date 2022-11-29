use cosmwasm_std::{
    coin, coins,
    testing::{mock_dependencies, mock_env, mock_info},
    Addr, Decimal, OwnedDeps, Uint128, Validator,
};
use cw_multi_test::{App, AppBuilder, BankSudo, StakingInfo, SudoMsg};

use mesh_testing::{
    constants::{CREATOR_ADDR, NATIVE_DENOM, VALIDATOR},
    instantiates::{instantiate_mesh_consumer, instantiate_meta_staking},
    macros::addr,
};

use crate::{
    contract::{execute, instantiate, sudo},
    msg::InstantiateMsg,
};
use mesh_apis::{StakingExecuteMsg as ExecuteMsg, StakingSudoMsg};

use super::executes::{add_consumer, delegate};

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

pub fn setup_with_multiple_delegations() -> (App, Addr, Addr, Addr) {
    let (mut app, meta_staking_addr, mesh_consumer_addr_1) = setup_with_consumer();

    // We add another consumer
    let mesh_consumer_addr_2 =
        instantiate_mesh_consumer(&mut app, None, Some(meta_staking_addr.clone()));

    add_consumer(
        &mut app,
        meta_staking_addr.as_str(),
        CREATOR_ADDR,
        mesh_consumer_addr_2.as_str(),
        10000,
    )
    .unwrap();

    // Delegate from both consumers
    delegate(
        &mut app,
        meta_staking_addr.as_str(),
        mesh_consumer_addr_1.as_str(),
        VALIDATOR,
        Uint128::new(2345),
    )
    .unwrap();

    delegate(
        &mut app,
        meta_staking_addr.as_str(),
        mesh_consumer_addr_2.as_str(),
        VALIDATOR,
        Uint128::new(7655),
    )
    .unwrap();

    (
        app,
        meta_staking_addr,
        mesh_consumer_addr_1,
        mesh_consumer_addr_2,
    )
}

// UNIT test setups
pub fn setup_unit_with_contract() -> (
    OwnedDeps<
        cosmwasm_std::MemoryStorage,
        cosmwasm_std::testing::MockApi,
        cosmwasm_std::testing::MockQuerier,
    >,
    Addr,
) {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let init_info = mock_info(CREATOR_ADDR, &[]);

    // init meta-staking
    instantiate(deps.as_mut(), env.clone(), init_info, InstantiateMsg {}).unwrap();

    let staking_addr = env.contract.address;
    deps.querier
        .update_balance(staking_addr.clone(), coins(100000, NATIVE_DENOM));

    // Add module Staking init
    deps.querier.update_staking(
        NATIVE_DENOM,
        &[Validator {
            address: VALIDATOR.to_string(),
            commission: Decimal::zero(),
            max_commission: Decimal::one(),
            max_change_rate: Decimal::one(),
        }],
        &[],
    );

    (deps, staking_addr)
}

pub fn setup_unit_with_delegation() -> (
    OwnedDeps<
        cosmwasm_std::MemoryStorage,
        cosmwasm_std::testing::MockApi,
        cosmwasm_std::testing::MockQuerier,
    >,
    Addr,
    Addr,
) {
    let (mut deps, staking_addr) = setup_unit_with_contract();
    let env = mock_env();
    let consumer_addr = addr!("consumer");

    // add_consumer
    sudo(
        deps.as_mut(),
        env.clone(),
        StakingSudoMsg::AddConsumer {
            consumer_address: consumer_addr.to_string(),
            funds_available_for_staking: coin(10000, NATIVE_DENOM),
        },
    )
    .unwrap();

    // execute delegation on contract
    let info = mock_info(consumer_addr.as_str(), &[]);
    execute(
        deps.as_mut(),
        env,
        info,
        ExecuteMsg::Delegate {
            validator: VALIDATOR.to_string(),
            amount: Uint128::new(10000),
        },
    )
    .unwrap();

    (deps, staking_addr, consumer_addr)
}
