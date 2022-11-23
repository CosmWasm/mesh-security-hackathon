use std::str::FromStr;

use cosmwasm_std::{Addr, Decimal};
use cw_multi_test::{App, Executor};

use crate::{
    contracts::{mesh_consumer_contract, meta_staking_contract},
    CREATOR_ADDR,
};

pub fn instantiate_meta_staking(
    app: &mut App,
    init_msg: Option<meta_staking::msg::InstantiateMsg>,
) -> Addr {
    let meta_staking_id = app.store_code(meta_staking_contract());
    let init_msg = init_msg.unwrap_or(meta_staking::msg::InstantiateMsg {});

    app.instantiate_contract(
        meta_staking_id,
        Addr::unchecked(CREATOR_ADDR),
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
        Addr::unchecked(CREATOR_ADDR),
        &init_msg,
        &[],
        "mesh-consumer",
        Some(CREATOR_ADDR.to_string()),
    )
    .unwrap()
}
