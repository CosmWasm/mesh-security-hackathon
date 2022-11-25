use cosmwasm_std::{OwnedDeps, Addr, testing::{mock_dependencies, MockApi, MockQuerier}, MemoryStorage, Empty};

use super::helpers::{instantiate_consumer, ibc_open_channel};

pub fn setup_with_channel() -> (OwnedDeps<MemoryStorage, MockApi, MockQuerier<Empty>, Empty>, Addr){
    let mut deps = mock_dependencies();
    let consumer_addr = instantiate_consumer(deps.as_mut());

    ibc_open_channel(deps.as_mut());

    (deps, consumer_addr)
}
