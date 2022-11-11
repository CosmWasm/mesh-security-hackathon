use std::str::FromStr;

use cosmwasm_std::{to_binary, Addr, Coin, Decimal, Empty, Uint128};
use cw_multi_test::{App, Contract, ContractWrapper, Executor, SudoMsg, WasmSudo};

use mesh_consumer::msg::{InstantiateMsg as ConsumerInitMsg, ProviderInfo};

use crate::msg::{InstantiateMsg, SudoMsg as MetaStakingSudoMsg};

use super::{ADMIN, NATIVE_DENOM};

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

pub fn get_meta_staking_instantiate_msg() -> InstantiateMsg {
    InstantiateMsg {}
}

pub fn instantiate_meta_staking(app: &mut App, init_msg: Option<InstantiateMsg>) -> Addr {
    let init_msg = init_msg.unwrap_or(get_meta_staking_instantiate_msg());

    let meta_staking_code_id = app.store_code(meta_staking_contract());

    app.instantiate_contract(
        meta_staking_code_id,
        ADMIN.addr(),
        &init_msg,
        &[],
        "meta-staking",
        Some(ADMIN.addr().to_string()),
    )
    .unwrap()
}

pub fn instantiate_and_add_consumer(app: &mut App, meta_staking_addr: &Addr) -> Addr {
    let init_msg = ConsumerInitMsg {
        provider: ProviderInfo {
            port_id: "port-id".to_string(),
            connection_id: "connection-id".to_string(),
        },
        remote_to_local_exchange_rate: Decimal::from_str("0.1").unwrap(),
        meta_staking_contract_address: meta_staking_addr.to_string(),
        ics20_channel: "channel-1".to_string(),
        packet_lifetime: None,
    };

    let consumer_code_id = app.store_code(mesh_consumer_contract());

    let consumer_addr = app
        .instantiate_contract(
            consumer_code_id,
            ADMIN.addr(),
            &init_msg,
            &[],
            "mesh-consumer",
            Some(ADMIN.addr().to_string()),
        )
        .unwrap();

    // Gov adds consumer
    app.sudo(SudoMsg::Wasm(WasmSudo {
        contract_addr: meta_staking_addr.clone(),
        msg: to_binary(&MetaStakingSudoMsg::AddConsumer {
            consumer_address: consumer_addr.to_string(),
            funds_available_for_staking: Coin {
                denom: NATIVE_DENOM.to_string(),
                amount: Uint128::new(100000000),
            },
        })
        .unwrap(),
    }))
    .unwrap();

    consumer_addr
}
