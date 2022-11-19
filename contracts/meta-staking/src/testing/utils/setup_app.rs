use cosmwasm_std::{coin, testing::mock_env, Addr, Decimal, Empty, Uint128, Validator};
use cw_multi_test::{Contract, ContractWrapper, StakingInfo};

// use mesh_consumer::msg::{InstantiateMsg as ConsumerInstantiateMsg, ProviderInfo};
use mesh_testing::{
    app_wrapper::{AppExecute, AppInit, AppSudo, AppWrapper, StoreContract},
    enum_str, ADMIN, NATIVE_DENOM,
};

use crate::{
    error::ContractError,
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg, SudoMsg},
};

use super::{CONSUMER_1, CONSUMER_2, VALIDATOR};

pub type AppWrapperType = AppWrapper<ContractError, InstantiateMsg, ExecuteMsg, QueryMsg, SudoMsg>;

// Enum to hold all contract names we gonna use in our testing
// Can be omitted and get the addr from the init msg instead of app_wrapper
enum_str! {
    enum ContractNames {
        MetaStaking,
        MeshConsumer,
    }
}

fn meta_staking_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        crate::contract::execute,
        crate::contract::instantiate,
        crate::contract::query,
    )
    .with_reply(crate::contract::reply)
    .with_sudo(crate::contract::sudo);
    Box::new(contract)
}

fn _mesh_consumer_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        mesh_consumer::contract::execute,
        mesh_consumer::contract::instantiate,
        mesh_consumer::contract::query,
    );
    Box::new(contract)
}

/// Basic app wrapper with defined module of our needs.
pub fn setup_app() -> AppWrapperType {
    let app_wrapper = AppWrapper::build_app(|router, api, storage| {
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
    });

    app_wrapper
}

pub fn setup_app_with_contract() -> (AppWrapperType, Addr) {
    let mut app_wrapper = setup_app();

    // Init meta-staking
    let meta_staking_addr = app_wrapper.init_contract(
        ADMIN.addr(),
        StoreContract::new(meta_staking_contract(), InstantiateMsg {}),
    );
    app_wrapper.fund_address(meta_staking_addr.clone());

    (app_wrapper, meta_staking_addr)
}

/// Init contracts we need for our test
pub fn setup_app_with_consumer() -> (AppWrapperType, Addr) {
    let (mut app_wrapper, meta_staking_addr) = setup_app_with_contract();

    app_wrapper
        .sudo_contract(
            &meta_staking_addr,
            SudoMsg::AddConsumer {
                consumer_address: CONSUMER_1.to_string(),
                funds_available_for_staking: coin(10000, NATIVE_DENOM),
            },
        )
        .unwrap();

    (app_wrapper, meta_staking_addr)
}

/// Setup contract with 2 delegations mainly to test rewards
pub fn setup_app_with_multiple_delegations() -> (AppWrapperType, Addr) {
    let (mut app_wrapper, meta_staking_addr) = setup_app_with_consumer();

    app_wrapper
        .sudo_contract(
            &meta_staking_addr,
            SudoMsg::AddConsumer {
                consumer_address: CONSUMER_2.addr().to_string(),
                funds_available_for_staking: coin(10000, NATIVE_DENOM),
            },
        )
        .unwrap();

    // Add delegations
    app_wrapper
        .execute(
            meta_staking_addr.clone(),
            CONSUMER_1.addr(),
            ExecuteMsg::Delegate {
                validator: VALIDATOR.addr().to_string(),
                amount: Uint128::new(7654_u128),
            },
        )
        .unwrap();

    app_wrapper
        .execute(
            meta_staking_addr.clone(),
            CONSUMER_2.addr(),
            ExecuteMsg::Delegate {
                validator: VALIDATOR.addr().to_string(),
                amount: Uint128::new(2346_u128),
            },
        )
        .unwrap();

    app_wrapper.next_block();

    (app_wrapper, meta_staking_addr)
}

// app_wrapper.init_contract(
//     ADMIN.addr(),
//     StoreContract::new_with_name(
//         ContractNames::MeshConsumer.to_str(),
//         mesh_consumer_contract(),
//         ConsumerInstantiateMsg {
//             provider: ProviderInfo {
//                 port_id: "port-id".to_string(),
//                 connection_id: "connection-id".to_string(),
//             },
//             remote_to_local_exchange_rate: Decimal::from_str("0.1").unwrap(),
//             meta_staking_contract_address: meta_staking_addr.to_string(),
//             ics20_channel: "channel-1".to_string(),
//             packet_lifetime: None,
//         },
//     ),
// );
