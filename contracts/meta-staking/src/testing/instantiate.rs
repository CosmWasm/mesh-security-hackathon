use cosmwasm_std::{Addr, Empty};
use cw_multi_test::{App, Contract, ContractWrapper, Executor};

use crate::msg::InstantiateMsg;

use super::ADMIN;

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

pub fn get_meta_staking_instantiate_msg() -> InstantiateMsg {
    InstantiateMsg {}
}

pub fn instantiate_meta_staking(app: &mut App, init_msg: Option<InstantiateMsg>) -> Addr {
    let init_msg = init_msg.unwrap_or(get_meta_staking_instantiate_msg());

    let meta_staking_code_id = app.store_code(meta_staking_contract());

    let meta_staking_addr = app
        .instantiate_contract(
            meta_staking_code_id,
            ADMIN.addr(),
            &init_msg,
            &[],
            "meta-staking",
            Some(ADMIN.addr().to_string()),
        )
        .unwrap();

    meta_staking_addr
}
