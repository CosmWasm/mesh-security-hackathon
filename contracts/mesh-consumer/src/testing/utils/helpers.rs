use std::{fmt, str::FromStr};

use cosmwasm_std::{
    from_binary, to_binary, Binary, Decimal, IbcChannel, IbcEndpoint, IbcOrder, IbcPacket,
    IbcTimeout, Timestamp,
};
use mesh_ibc::{StdAck, IBC_APP_VERSION};
use serde::Deserialize;

use crate::msg::{InstantiateMsg, ProviderInfo};

pub const STAKING_ADDR: &str = "meta_staking";
pub const RELAYER_ADDR: &str = "relayer";
const CONTRACT_PORT: &str = "wasm.address1";
pub const REMOTE_PORT: &str = "stars.address1";
pub const CONNECTION_ID: &str = "connection-1";
pub const CHANNEL_ID: &str = "channel-1";
pub const ICS20_CHANNEL_ID: &str = "channel-2";
const DEFAULT_TIMEOUT: u64 = 60;

pub fn get_default_instantiate_msg() -> InstantiateMsg {
    InstantiateMsg {
        provider: ProviderInfo {
            port_id: REMOTE_PORT.to_string(),
            connection_id: CONNECTION_ID.to_string(),
        },
        remote_to_local_exchange_rate: Decimal::from_str("0.1").unwrap(),
        meta_staking_contract_address: STAKING_ADDR.to_string(),
        ics20_channel: ICS20_CHANNEL_ID.to_string(),
        packet_lifetime: None,
    }
}

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

pub fn to_ack_success<T: serde::Serialize>(response: T) -> Binary {
    to_binary(&StdAck::Result(to_binary(&response).unwrap())).unwrap()
}

pub fn to_ack_error(response: &str) -> Binary {
    to_binary(&StdAck::Error(response.to_string())).unwrap()
}
