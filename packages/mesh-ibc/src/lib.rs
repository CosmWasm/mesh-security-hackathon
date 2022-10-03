mod ack;
mod checks;
mod ibc_msg;

use cosmwasm_std::IbcOrder;

pub use crate::ack::StdAck;
pub use crate::checks::{check_order, check_version, MeshSecurityError};
pub use crate::ibc_msg::{
    ConsumerMsg, ListValidatorsResponse, ProviderMsg, RewardsResponse, StakeResponse,
    UnstakeResponse, UpdateValidatorsResponse,
};

pub const IBC_APP_VERSION: &str = "mesh-security-v0.1";
pub const APP_ORDER: IbcOrder = IbcOrder::Unordered;
// we use this for tests to ensure it is rejected
pub const BAD_APP_ORDER: IbcOrder = IbcOrder::Ordered;
