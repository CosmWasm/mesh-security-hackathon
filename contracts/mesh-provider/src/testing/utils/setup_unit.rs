// File to setup unit testing for IBC stuff.

use cosmwasm_std::{
    testing::{mock_dependencies, mock_env, mock_info, MockApi, MockQuerier},
    to_binary, Addr, DepsMut, Empty, MemoryStorage, OwnedDeps,
};
use mesh_testing::{
    constants::{CONNECTION_ID, CREATOR_ADDR, LOCKUP_ADDR, REWARDS_IBC_DENOM},
    instantiates::get_mesh_slasher_init_msg,
};

use crate::{
    contract::instantiate,
    msg::{ConsumerInfo, InstantiateMsg, SlasherInfo},
};

type OwnedDepsType = OwnedDeps<MemoryStorage, MockApi, MockQuerier<Empty>, Empty>;

pub fn get_default_init_msg(slasher_code_id: u64) -> InstantiateMsg {
    InstantiateMsg {
        consumer: ConsumerInfo {
            connection_id: CONNECTION_ID.to_string(),
        },
        slasher: SlasherInfo {
            code_id: slasher_code_id,
            msg: to_binary(&get_mesh_slasher_init_msg()).unwrap(),
        },
        lockup: LOCKUP_ADDR.to_string(),
        unbonding_period: 86400 * 14,
        rewards_ibc_denom: REWARDS_IBC_DENOM.to_string(),
        packet_lifetime: None,
    }
}

pub fn instantiate_provider(mut deps: DepsMut, init_msg: Option<InstantiateMsg>) -> Addr {
    let info = mock_info(CREATOR_ADDR, &[]);
    let env = mock_env();
    let msg = init_msg.unwrap_or_else(|| get_default_init_msg(1));

    instantiate(deps.branch(), env.clone(), info, msg).unwrap();

    env.contract.address
}

pub fn setup_unit(init_msg: Option<InstantiateMsg>) -> (OwnedDepsType, Addr) {
    let mut deps = mock_dependencies();
    let provider_addr = instantiate_provider(deps.as_mut(), init_msg);

    (deps, provider_addr)
}
