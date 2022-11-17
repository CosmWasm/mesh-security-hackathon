use std::str::FromStr;

use cosmwasm_std::{testing::mock_env, Decimal, Empty, Validator};
use cw_multi_test::{Contract, ContractWrapper, StakingInfo};

use mesh_consumer::msg::{InstantiateMsg as ConsumerInstantiateMsg, ProviderInfo};
use mesh_testing::{
    app_wrapper::{AppWrapper, StoreContract, AppInit},
    ADMIN, NATIVE_DENOM, enum_str,
};

use crate::{
    error::ContractError,
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg},
};

use super::VALIDATOR;

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

fn mesh_consumer_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        mesh_consumer::contract::execute,
        mesh_consumer::contract::instantiate,
        mesh_consumer::contract::query,
    );
    Box::new(contract)
}

/// Basic app wrapper with defined module of our needs.
fn setup_app() -> AppWrapper<ContractError, InstantiateMsg, ExecuteMsg, QueryMsg> {
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

/// Init contracts we need for our test
pub fn setup_app_with_contracts() -> AppWrapper<ContractError, InstantiateMsg, ExecuteMsg, QueryMsg> {
    let mut app_wrapper = setup_app();

    // Init meta-staking
    let meta_staking_addr = app_wrapper.init_contract(
        ADMIN.addr(),
        StoreContract::new_with_name(
            ContractNames::MetaStaking.to_str(),
            meta_staking_contract(),
            InstantiateMsg {},
        ),
    );

    app_wrapper.fund_address(meta_staking_addr.clone());

    app_wrapper.init_contract(
        ADMIN.addr(),
        StoreContract::new_with_name(
            ContractNames::MeshConsumer.to_str(),
            mesh_consumer_contract(),
            ConsumerInstantiateMsg {
                provider: ProviderInfo {
                    port_id: "port-id".to_string(),
                    connection_id: "connection-id".to_string(),
                },
                remote_to_local_exchange_rate: Decimal::from_str("0.1").unwrap(),
                meta_staking_contract_address: meta_staking_addr.to_string(),
                ics20_channel: "channel-1".to_string(),
                packet_lifetime: None,
            },
        ),
    );

    app_wrapper
}

#[test]
fn test() {
    setup_app_with_contracts();
}
