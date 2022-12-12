use thiserror::Error;

use cosmwasm_std::{
    CheckedFromRatioError, DecimalRangeExceeded, DivideByZeroError, OverflowError,
    StdError,
};
use cw_utils::ParseReplyError;

use mesh_ibc::MeshSecurityError;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    OverflowError(#[from] OverflowError),

    #[error("{0}")]
    DivideByZeroError(#[from] DivideByZeroError),

    #[error("{0}")]
    CheckedFromRatioError(#[from] CheckedFromRatioError),

    #[error("{0}")]
    DecimalRangeExceeded(#[from] DecimalRangeExceeded),

    #[error("{0}")]
    Parse(#[from] ParseReplyError),

    #[error("{0}")]
    MeshSecurity(#[from] MeshSecurityError),

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Contract already has a bound channel: {0}")]
    ChannelExists(String),

    #[error("Contract already has a bound port: {0}")]
    PortExists(String),

    #[error("Unauthorized counterparty chain, awaiting connection '{0}'")]
    WrongConnection(String),

    #[error("Refuse to respond on unregistered channel '{0}'")]
    UnknownChannel(String),

    #[error("Invalid reply id: {0}")]
    InvalidReplyId(u64),

    #[error("Insufficient stake to withdraw this")]
    InsufficientStake,

    #[error("Cannot send zero tokens to any methods")]
    ZeroAmount,

    #[error("No tokens are ready to be unbonded")]
    NothingToClaim,

    #[error("No rewards to be claimed")]
    NoRewardsToClaim,

    #[error("Balance is too low: {rewards} > {balance}")]
    WrongBalance { balance: String, rewards: String },

    #[error("Validator was never registered: {0}")]
    UnknownValidator(String),

    #[error("Validator was removed from valset: {0}")]
    RemovedValidator(String),

    #[error("Something went wrong in the rewards calculation of the validator")]
    ValidatorRewardsCalculationWrong {},

    #[error("Rewards amount is 0")]
    ZeroRewardsToSend {},

    #[error("Custom Error val: {val:?}")]
    CustomError { val: String },
    // Add any other custom errors you like here.
    // Look at https://docs.rs/thiserror/1.0.21/thiserror/ for details.
}
