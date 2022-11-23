pub use crate::{APP_ORDER, IBC_APP_VERSION};
use cosmwasm_std::IbcOrder;

use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum MeshSecurityError {
    #[error("Only supports unordered channels")]
    InvalidChannelOrder,

    #[error("Counterparty version must be '{0}'")]
    InvalidChannelVersion(&'static str),
}

pub fn check_order(order: &IbcOrder) -> Result<(), MeshSecurityError> {
    if order != &APP_ORDER {
        Err(MeshSecurityError::InvalidChannelOrder)
    } else {
        Ok(())
    }
}

pub fn check_version(version: &str) -> Result<(), MeshSecurityError> {
    if version != IBC_APP_VERSION {
        Err(MeshSecurityError::InvalidChannelVersion(IBC_APP_VERSION))
    } else {
        Ok(())
    }
}
