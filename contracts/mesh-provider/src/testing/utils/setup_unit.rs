// File to setup unit testing for IBC stuff.

use cosmwasm_std::{
    testing::{mock_dependencies, MockApi, MockQuerier},
    Addr, Empty, MemoryStorage, OwnedDeps,
};

use crate::msg::InstantiateMsg;

use super::ibc_helpers::{ibc_open_channel, instantiate_provider};

type OwnedDepsType = OwnedDeps<MemoryStorage, MockApi, MockQuerier<Empty>, Empty>;

pub fn setup_unit(init_msg: Option<InstantiateMsg>) -> (OwnedDepsType, Addr) {
    let mut deps = mock_dependencies();
    let provider_addr = instantiate_provider(deps.as_mut(), init_msg);

    (deps, provider_addr)
}

pub fn setup_unit_with_channel(init_msg: Option<InstantiateMsg>) -> (OwnedDepsType, Addr) {
    let (mut deps, consumer_addr) = setup_unit(init_msg);

    ibc_open_channel(deps.as_mut()).unwrap();

    (deps, consumer_addr)
}
