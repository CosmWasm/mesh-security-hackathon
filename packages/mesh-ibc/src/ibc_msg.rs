use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{from_slice, to_binary, Binary, Coin, CosmosMsg, Empty, QueryRequest};

/// This is the message we send over the IBC channel
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PacketMsg {
    Dispatch {
        sender: String,
        msgs: Vec<CosmosMsg>,
        callback_id: Option<String>,
    },
    IbcQuery {
        sender: String,
        msgs: Vec<QueryRequest<Empty>>,
        callback_id: Option<String>,
    },
    WhoAmI {},
    Balances {},
}

/// Return the data field for each message
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct DispatchResponse {
    pub results: Vec<Binary>,
}

/// Return the data field for each message
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct IbcQueryResponse {
    pub results: Vec<Binary>,
}

/// This is the success response we send on ack for PacketMsg::WhoAmI.
/// Return the caller's account address on the remote chain
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct WhoAmIResponse {
    pub account: String,
}

/// This is the success response we send on ack for PacketMsg::Balance.
/// Just acknowledge success or error
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct BalancesResponse {
    pub account: String,
    pub balances: Vec<Coin>,
}
