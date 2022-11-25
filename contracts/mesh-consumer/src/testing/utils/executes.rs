use std::str::FromStr;

use cosmwasm_std::{
    coin,
    testing::{mock_env, mock_info},
    to_binary, Addr, Decimal, DepsMut, Empty, IbcAcknowledgement, IbcBasicResponse,
    IbcChannelConnectMsg, IbcChannelOpenMsg, IbcPacketAckMsg, IbcPacketReceiveMsg,
    IbcReceiveResponse, MessageInfo, Response, Uint128,
};
use mesh_apis::ConsumerExecuteMsg;
use mesh_ibc::{ConsumerMsg, ProviderMsg, IBC_APP_VERSION};
use mesh_testing::{
    addr,
    constants::{CREATOR_ADDR, NATIVE_DENOM},
};

use crate::{
    contract::{execute, instantiate},
    ibc::{ibc_channel_connect, ibc_channel_open, ibc_packet_ack, ibc_packet_receive},
    msg::{InstantiateMsg, ProviderInfo},
    ContractError,
};

use super::helpers::{
    mock_channel, mock_packet, CHANNEL_ID, CONNECTION_ID, ICS20_CHANNEL_ID, RELAYER_ADDR,
    REMOTE_PORT, STAKING_ADDR,
};

pub fn instantiate_consumer(mut deps: DepsMut) -> Addr {
    let info = mock_info(CREATOR_ADDR, &[]);
    let env = mock_env();
    let msg = InstantiateMsg {
        provider: ProviderInfo {
            port_id: REMOTE_PORT.to_string(),
            connection_id: CONNECTION_ID.to_string(),
        },
        remote_to_local_exchange_rate: Decimal::from_str("0.1").unwrap(),
        meta_staking_contract_address: STAKING_ADDR.to_string(),
        ics20_channel: ICS20_CHANNEL_ID.to_string(),
        packet_lifetime: None,
    };

    instantiate(deps.branch(), env.clone(), info, msg).unwrap();

    env.contract.address
}

pub fn execute_receive_rewards(
    deps: DepsMut,
    info: MessageInfo,
    validator: &str,
) -> Result<Response<Empty>, ContractError> {
    execute(
        deps,
        mock_env(),
        info,
        ConsumerExecuteMsg::MeshConsumerRecieveRewardsMsg {
            validator: validator.to_string(),
        },
    )
}

pub fn ibc_open_channel(mut deps: DepsMut) {
    let channel = mock_channel(CHANNEL_ID);
    let open_msg = IbcChannelOpenMsg::new_init(channel.clone());
    ibc_channel_open(deps.branch(), mock_env(), open_msg).unwrap();
    let connect_msg = IbcChannelConnectMsg::new_ack(channel.clone(), IBC_APP_VERSION);
    ibc_channel_connect(deps.branch(), mock_env(), connect_msg).unwrap();
}

pub fn ibc_receive_list_validators(deps: DepsMut) -> Result<IbcReceiveResponse, ContractError> {
    let packet = mock_packet(to_binary(&ProviderMsg::ListValidators {}).unwrap());

    ibc_packet_receive(
        deps,
        mock_env(),
        IbcPacketReceiveMsg::new(packet, addr!(RELAYER_ADDR)),
    )
}

pub fn ibc_receive_stake(
    deps: DepsMut,
    validator: &str,
    amount: u128,
    key: &str,
) -> Result<IbcReceiveResponse, ContractError> {
    let packet = mock_packet(
        to_binary(&ProviderMsg::Stake {
            validator: validator.to_string(),
            amount: Uint128::new(amount),
            key: key.to_string(),
        })
        .unwrap(),
    );

    ibc_packet_receive(
        deps,
        mock_env(),
        IbcPacketReceiveMsg::new(packet, addr!(RELAYER_ADDR)),
    )
}

pub fn ibc_receive_unstake(
    deps: DepsMut,
    validator: &str,
    amount: u128,
    key: &str,
) -> Result<IbcReceiveResponse, ContractError> {
    let packet = mock_packet(
        to_binary(&ProviderMsg::Unstake {
            validator: validator.to_string(),
            amount: Uint128::new(amount),
            key: key.to_string(),
        })
        .unwrap(),
    );

    ibc_packet_receive(
        deps,
        mock_env(),
        IbcPacketReceiveMsg::new(packet, addr!(RELAYER_ADDR)),
    )
}

pub fn ibc_ack_rewards(
    deps: DepsMut,
    validator: &str,
    amount: u128,
    ack: IbcAcknowledgement,
) -> Result<IbcBasicResponse, ContractError> {
    let original_packet = mock_packet(
        to_binary(&ConsumerMsg::Rewards {
            validator: validator.to_string(),
            total_funds: coin(amount, NATIVE_DENOM),
        })
        .unwrap(),
    );

    ibc_packet_ack(
        deps,
        mock_env(),
        IbcPacketAckMsg::new(ack, original_packet, addr!(RELAYER_ADDR)),
    )
}

pub fn _ibc_ack_update_validators() {}
