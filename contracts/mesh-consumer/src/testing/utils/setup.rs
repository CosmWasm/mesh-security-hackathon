use cosmwasm_std::{
    testing::{mock_dependencies, MockApi, MockQuerier},
    Addr, Empty, MemoryStorage, OwnedDeps,
};

use crate::msg::InstantiateMsg;

use super::executes::{ibc_open_channel, instantiate_consumer};

type OwnedDepsType = OwnedDeps<MemoryStorage, MockApi, MockQuerier<Empty>, Empty>;

pub fn setup(init_msg: Option<InstantiateMsg>) -> (OwnedDepsType, Addr) {
    let mut deps = mock_dependencies();
    let consumer_addr = instantiate_consumer(deps.as_mut(), init_msg);

    (deps, consumer_addr)
}

pub fn setup_with_channel(init_msg: Option<InstantiateMsg>) -> (OwnedDepsType, Addr) {
    let (mut deps, consumer_addr) = setup(init_msg);

    ibc_open_channel(deps.as_mut()).unwrap();

    (deps, consumer_addr)
}
