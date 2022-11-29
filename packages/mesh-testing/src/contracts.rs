use cosmwasm_std::Empty;
use cw_multi_test::{Contract, ContractWrapper};

pub fn meta_staking_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        meta_staking::contract::execute,
        meta_staking::contract::instantiate,
        meta_staking::contract::query,
    )
    .with_sudo(meta_staking::contract::sudo);
    Box::new(contract)
}

pub fn mesh_consumer_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        mesh_consumer::contract::execute,
        mesh_consumer::contract::instantiate,
        mesh_consumer::contract::query,
    );
    Box::new(contract)
}

pub fn mesh_provider_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        mesh_provider::contract::execute,
        mesh_provider::contract::instantiate,
        mesh_provider::contract::query,
    )
    .with_reply(mesh_provider::contract::reply);
    Box::new(contract)
}

pub fn mesh_slasher_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        mesh_slasher::contract::execute,
        mesh_slasher::contract::instantiate,
        mesh_slasher::contract::query,
    );
    Box::new(contract)
}
