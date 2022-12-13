use std::str::FromStr;

use cosmwasm_std::Decimal;
use mesh_testing::constants::{CONNECTION_ID, ICS20_CHANNEL_ID, REMOTE_PORT};

use crate::msg::{InstantiateMsg, ProviderInfo};

pub const STAKING_ADDR: &str = "meta_staking";

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
