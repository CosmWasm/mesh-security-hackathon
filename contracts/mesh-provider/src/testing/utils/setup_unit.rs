// File to setup unit testing for IBC stuff.

use cosmwasm_std::{
    testing::{mock_dependencies, mock_env, mock_info, MockApi, MockQuerier},
    to_binary, Addr, DepsMut, Empty, MemoryStorage, OwnedDeps, IbcChannelCloseMsg, IbcChannelConnectMsg, IbcBasicResponse, IbcChannel, Ibc3ChannelOpenResponse, IbcChannelOpenMsg,
};
use mesh_ibc::IBC_APP_VERSION;
use mesh_testing::{
    constants::{CONNECTION_ID, CREATOR_ADDR, LOCKUP_ADDR, REWARDS_IBC_DENOM, CHANNEL_ID},
    instantiates::get_mesh_slasher_init_msg, ibc_helpers::mock_channel,
};

use crate::{
    contract::instantiate,
    msg::{ConsumerInfo, InstantiateMsg, SlasherInfo}, ibc::{ibc_channel_connect, ibc_channel_open, ibc_channel_close}, ContractError,
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

pub fn ibc_open(
    mut deps: DepsMut,
    channel: IbcChannel,
) -> Result<Option<Ibc3ChannelOpenResponse>, ContractError> {
    let open_msg = IbcChannelOpenMsg::new_init(channel);
    ibc_channel_open(deps.branch(), mock_env(), open_msg)
}

pub fn ibc_connect(
    mut deps: DepsMut,
    channel: IbcChannel,
) -> Result<IbcBasicResponse, ContractError> {
    let connect_msg = IbcChannelConnectMsg::new_ack(channel, IBC_APP_VERSION);
    ibc_channel_connect(deps.branch(), mock_env(), connect_msg)
}

pub fn ibc_open_channel(mut deps: DepsMut) -> Result<(), ContractError> {
    let channel = mock_channel(CHANNEL_ID, IBC_APP_VERSION);

    ibc_open(deps.branch(), channel.clone())?;
    ibc_connect(deps.branch(), channel)?;
    Ok(())
}

pub fn ibc_close_channel(mut deps: DepsMut) -> Result<(), ContractError> {
    let channel = mock_channel(CHANNEL_ID, IBC_APP_VERSION);

    let close_msg = IbcChannelCloseMsg::new_init(channel);
    ibc_channel_close(deps.branch(), mock_env(), close_msg)?;
    Ok(())
}

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
