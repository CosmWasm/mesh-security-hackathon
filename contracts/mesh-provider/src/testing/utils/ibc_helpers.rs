// File to setup unit testing for IBC stuff.

use cosmwasm_std::{
    testing::{mock_env, mock_info},
    to_binary, Addr, Deps, DepsMut, Ibc3ChannelOpenResponse, IbcBasicResponse, IbcChannel,
    IbcChannelCloseMsg, IbcChannelConnectMsg, IbcChannelOpenMsg, IbcPacketReceiveMsg,
    IbcReceiveResponse, Response, StdError, Uint128,
};
use mesh_ibc::{ConsumerMsg, IBC_APP_VERSION};
use mesh_testing::{
    addr,
    constants::{
        CHANNEL_ID, CONNECTION_ID, CREATOR_ADDR, LOCKUP_ADDR, RELAYER_ADDR, REWARDS_IBC_DENOM,
    },
    ibc_helpers::{mock_channel, mock_packet},
    instantiates::get_mesh_slasher_init_msg,
};

use crate::{
    contract::{execute, instantiate},
    ibc::{ibc_channel_close, ibc_channel_connect, ibc_channel_open, ibc_packet_receive},
    msg::{ConsumerInfo, ExecuteMsg, InstantiateMsg, SlasherInfo},
    state::{Validator, VALIDATORS},
    ContractError,
};

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

pub fn update_unit_validator(
    deps: DepsMut,
    added: Vec<String>,
    removed: Vec<String>,
) -> IbcReceiveResponse {
    let packet = mock_packet(to_binary(&ConsumerMsg::UpdateValidators { added, removed }).unwrap());

    ibc_packet_receive(
        deps,
        mock_env(),
        IbcPacketReceiveMsg::new(packet, addr!(RELAYER_ADDR)),
    )
    .unwrap()
}

pub fn add_unit_stake(
    deps: DepsMut,
    delegator: &str,
    validator: &str,
    amount: Uint128,
) -> Result<Response, ContractError> {
    execute(
        deps,
        mock_env(),
        mock_info(LOCKUP_ADDR, &[]),
        ExecuteMsg::ReceiveClaim {
            owner: delegator.to_string(),
            amount,
            validator: validator.to_string(),
        },
    )
}

pub fn remove_unit_stake(
    deps: DepsMut,
    delegator: &str,
    validator: &str,
    amount: Uint128,
) -> Result<Response, ContractError> {
    execute(
        deps,
        mock_env(),
        mock_info(delegator, &[]),
        ExecuteMsg::Unstake {
            amount,
            validator: validator.to_string(),
        },
    )
}

// Queries
pub fn query_unit_validators(deps: Deps, validator: &str) -> Result<Validator, StdError> {
    VALIDATORS.load(deps.storage, validator)
}
