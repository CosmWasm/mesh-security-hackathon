use std::fmt;

use cosmwasm_std::{
    from_binary, to_binary, Binary, IbcChannel, IbcEndpoint, IbcOrder, IbcPacket, IbcTimeout,
    Timestamp,
};
use mesh_ibc::StdAck;
use serde::Deserialize;

use crate::constants::{CHANNEL_ID, CONNECTION_ID, CONTRACT_PORT, DEFAULT_TIMEOUT, REMOTE_PORT};

pub fn mock_channel(channel_id: &str, ibc_version: &str) -> IbcChannel {
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
        ibc_version,
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
