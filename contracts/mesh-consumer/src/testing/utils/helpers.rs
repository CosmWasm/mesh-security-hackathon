use std::fmt;

use cosmwasm_std::{
    from_binary, Binary, IbcChannel, IbcEndpoint, IbcOrder, IbcPacket, IbcTimeout, Timestamp,
};
use mesh_ibc::{StdAck, IBC_APP_VERSION};
use serde::Deserialize;

pub const STAKING_ADDR: &str = "meta_staking";
pub const RELAYER_ADDR: &str = "relayer";
const CONTRACT_PORT: &str = "wasm.address1";
pub const REMOTE_PORT: &str = "stars.address1";
pub const CONNECTION_ID: &str = "connection-1";
pub const CHANNEL_ID: &str = "channel-1";
pub const ICS20_CHANNEL_ID: &str = "channel-2";
const DEFAULT_TIMEOUT: u64 = 60;

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
