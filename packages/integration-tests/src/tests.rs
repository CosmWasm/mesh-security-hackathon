use cosmwasm_std::{to_binary, Addr, Empty, IbcTimeout, IbcTimeoutBlock, WasmMsg};
use cw_multi_test::{App, Contract, ContractWrapper, Executor};

fn provider_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        mesh_provider::contract::execute,
        mesh_provider::contract::instantiate,
        mesh_provider::contract::query,
    ).with_reply(mesh_provider::contract::reply);
    Box::new(contract)
}

fn consumer_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        mesh_consumer::contract::execute,
        mesh_consumer::contract::instantiate,
        mesh_consumer::contract::query,
    )
    .with_reply(mesh_consumer::ibc::reply);
    Box::new(contract)
}

fn staking_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        meta_staking::contract::execute,
        meta_staking::contract::instantiate,
        meta_staking::contract::query,
    );
    Box::new(contract)
}


