use cosmwasm_std::{CheckedFromRatioError, Decimal, DivideByZeroError, OverflowError, StdError};
use cw_utils::ParseReplyError;
use thiserror::Error;

use mesh_ibc::MeshSecurityError;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    OverflowError(#[from] OverflowError),

    #[error("{0}")]
    CheckedFromRatioError(#[from] CheckedFromRatioError),

    #[error("{0}")]
    ParseReplyError(#[from] ParseReplyError),

    #[error("{0}")]
    DivideByZeroError(#[from] DivideByZeroError),

    #[error("Invalid reply id: {0}")]
    InvalidReplyId(u64),

    #[error("Balance is too low: {rewards:?} > {balance:?}")]
    WrongBalance { balance: Decimal, rewards: Decimal },

    #[error("{0}")]
    MeshSecurity(#[from] MeshSecurityError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Couldn't parse provider from port_id")]
    ProviderAddrParsing {},

    #[error("Contract already has a bound channel: {0}")]
    ChannelExists(String),

    #[error("Unauthorized counterparty chain, awaiting connection '{0}'")]
    WrongConnection(String),

    #[error("Unauthorized counterparty port, awaiting port '{0}'")]
    WrongPort(String),

    #[error("Refuse to respond on unregistered channel '{0}'")]
    UnknownChannel(String),

    #[error("Custom Error val: {val:?}")]
    CustomError { val: String },

    #[error("Acknowledgement failed")]
    AckFailed {},

    #[error("Acknowledgement data response is None")]
    AckDataIsNone {},

    #[error("Rewards acknowledgement failed")]
    RewardsFailed {},

    #[error("Update validators acknowledgement failed")]
    UpdateValidatorsFailed {},
}
