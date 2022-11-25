use std::{fmt, str::FromStr};

use cosmwasm_std::{
    from_binary,
    testing::{mock_env, mock_info},
    to_binary, Addr, Binary, Decimal, DepsMut, Empty, IbcChannel, IbcChannelConnectMsg,
    IbcChannelOpenMsg, IbcEndpoint, IbcOrder, IbcPacket, IbcPacketReceiveMsg, IbcReceiveResponse,
    IbcTimeout, MessageInfo, Response, Timestamp, Uint128,
};
use mesh_apis::ConsumerExecuteMsg;
use mesh_ibc::{ProviderMsg, StdAck, IBC_APP_VERSION};
use mesh_testing::{addr, constants::CREATOR_ADDR};
use serde::Deserialize;

use crate::{
    contract::{execute, instantiate},
    ibc::{ibc_channel_connect, ibc_channel_open, ibc_packet_receive},
    msg::{InstantiateMsg, ProviderInfo},
    ContractError,
};

pub const STAKING_ADDR: &str = "meta_staking";
pub const RELAYER_ADDR: &str = "relayer";
const CONTRACT_PORT: &str = "wasm.address1";
const REMOTE_PORT: &str = "stars.address1";
const CONNECTION_ID: &str = "connection-1";
pub const CHANNEL_ID: &str = "channel-1";
const ICS20_CHANNEL_ID: &str = "channel-2";
const DEFAULT_TIMEOUT: u64 = 60;

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

// IBC related helpers
pub fn mock_channel(channel_id: &str) -> IbcChannel {
    IbcChannel::new(
        IbcEndpoint {
            port_id: CONTRACT_PORT.to_string(),
            channel_id: channel_id.to_string(),
        },
        IbcEndpoint {
            port_id: REMOTE_PORT.to_string(),
            channel_id: format!("{}0", channel_id),
        },
        IbcOrder::Unordered,
        IBC_APP_VERSION,
        CONNECTION_ID,
    )
}

pub fn mock_packet(data: Binary) -> IbcPacket {
    IbcPacket::new(
        data,
        IbcEndpoint {
            port_id: REMOTE_PORT.to_string(),
            channel_id: CHANNEL_ID.to_string(),
        },
        IbcEndpoint {
            port_id: CONTRACT_PORT.to_string(),
            channel_id: CHANNEL_ID.to_string(),
        },
        1, // Packet sequence number.
        IbcTimeout::with_timestamp(Timestamp::from_seconds(DEFAULT_TIMEOUT)),
    )
}

pub fn ack_unwrap<R: for<'de> Deserialize<'de> + fmt::Debug + PartialEq>(res: Binary) -> R {
    from_binary::<R>(&(from_binary::<StdAck>(&res).unwrap()).unwrap()).unwrap()
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
