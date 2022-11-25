use std::str::FromStr;

use cosmwasm_std::{DepsMut, Addr, testing::{mock_info, mock_env}, Decimal, MessageInfo, Response, Empty, IbcChannelOpenMsg, IbcChannelConnectMsg, IbcReceiveResponse, to_binary, IbcPacketReceiveMsg, Uint128};
use mesh_apis::ConsumerExecuteMsg;
use mesh_ibc::{IBC_APP_VERSION, ProviderMsg};
use mesh_testing::{constants::CREATOR_ADDR, addr};

use crate::{msg::{InstantiateMsg, ProviderInfo}, contract::{instantiate, execute}, ContractError, ibc::{ibc_channel_open, ibc_channel_connect, ibc_packet_receive}};

use super::helpers::{REMOTE_PORT, CONNECTION_ID, STAKING_ADDR, ICS20_CHANNEL_ID, mock_channel, CHANNEL_ID, mock_packet, RELAYER_ADDR};

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

pub fn _ibc_ack_rewards() {}

pub fn _ibc_ack_update_validators() {}
