use std::str::FromStr;

use cosmwasm_std::{to_binary, Addr, Decimal};
use cw_multi_test::{App, Executor};

use crate::{
    addr,
    constants::{CONNECTION_ID, CREATOR_ADDR},
    contracts::{
        mesh_consumer_contract, mesh_provider_contract, mesh_slasher_contract,
        meta_staking_contract,
    },
};

use mesh_provider::msg::InstantiateMsg as ProviderInit;

pub fn get_default_mesh_provider_init_msg(slasher_code_id: u64) -> ProviderInit {
    ProviderInit {
        consumer: mesh_provider::msg::ConsumerInfo {
            connection_id: CONNECTION_ID.to_string(),
        },
        slasher: mesh_provider::msg::SlasherInfo {
            code_id: slasher_code_id,
            msg: to_binary(&mesh_slasher::msg::InstantiateMsg {
                owner: CREATOR_ADDR.to_string(),
            })
            .unwrap(),
        },
        lockup: "lockup_contract".to_string(),
        unbonding_period: 86400 * 14,
        rewards_ibc_denom: "".to_string(),
        packet_lifetime: None,
    }
}

pub fn instantiate_mesh_provider(
    app: &mut App,
    init_msg: Option<mesh_provider::msg::InstantiateMsg>,
) -> Addr {
    let mesh_provider_id = app.store_code(mesh_provider_contract());
    let mesh_slasher_id = app.store_code(mesh_slasher_contract());
    let init_msg = init_msg.unwrap_or_else(|| get_default_mesh_provider_init_msg(mesh_slasher_id));

    app.instantiate_contract(
        mesh_provider_id,
        addr!(CREATOR_ADDR),
        &init_msg,
        &[],
        "mesh-provider",
        Some(CREATOR_ADDR.to_string()),
    )
    .unwrap()
}

pub fn instantiate_meta_staking(
    app: &mut App,
    init_msg: Option<meta_staking::msg::InstantiateMsg>,
) -> Addr {
    let meta_staking_id = app.store_code(meta_staking_contract());
    let init_msg = init_msg.unwrap_or(meta_staking::msg::InstantiateMsg {});

    app.instantiate_contract(
        meta_staking_id,
        addr!(CREATOR_ADDR),
        &init_msg,
        &[],
        "meta-staking",
        Some(CREATOR_ADDR.to_string()),
    )
    .unwrap()
}

pub fn instantiate_mesh_consumer(
    app: &mut App,
    init_msg: Option<mesh_consumer::msg::InstantiateMsg>,
    meta_staking_addr: Option<Addr>,
) -> Addr {
    let mesh_consumer_id = app.store_code(mesh_consumer_contract());
    let init_msg = init_msg.unwrap_or(mesh_consumer::msg::InstantiateMsg {
        provider: mesh_consumer::msg::ProviderInfo {
            port_id: "some_port".to_string(),
            connection_id: "come_connection".to_string(),
        },
        remote_to_local_exchange_rate: Decimal::from_str("0.1").unwrap(),
        meta_staking_contract_address: meta_staking_addr.unwrap().to_string(),
        ics20_channel: "some_channel".to_string(),
        packet_lifetime: None,
    });

    app.instantiate_contract(
        mesh_consumer_id,
        addr!(CREATOR_ADDR),
        &init_msg,
        &[],
        "mesh-consumer",
        Some(CREATOR_ADDR.to_string()),
    )
    .unwrap()
}
