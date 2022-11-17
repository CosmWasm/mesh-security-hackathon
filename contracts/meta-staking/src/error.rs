use cosmwasm_std::{
    CheckedFromRatioError, DecimalRangeExceeded, DivideByZeroError, OverflowError, StdError,
};
use cw_utils::ParseReplyError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    OverflowErr(#[from] OverflowError),

    #[error(transparent)]
    ParseReplyError(#[from] ParseReplyError),

    #[error(transparent)]
    DivideByZeroError(#[from] DivideByZeroError),

    #[error(transparent)]
    CheckedFromRatioError(#[from] CheckedFromRatioError),

    #[error(transparent)]
    DecimalRangeExceeded(#[from] DecimalRangeExceeded),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Incorrect coin denom")]
    IncorrectDenom {},

    #[error("Cannot undelegate more than you previously delegated")]
    InsufficientDelegation {},

    #[error("Contract has run out of funds to delegate for consumer chain")]
    NoFundsToDelegate {},

    #[error("Cannot undelegate from a a validator that does not have delegations")]
    NoDelegationsForValidator {},

    #[error("Contract does not have enough funds for consumer")]
    NotEnoughFunds {},

    #[error("Consumer already exists")]
    ConsumerAlreadyExists {},

    #[error("Consumer does not exists")]
    NoConsumer {},

    #[error("Rewards amount is 0")]
    ZeroRewardsToSend {},

    #[error("Something went wrong in the rewards calculation of the validator")]
    ValidatorRewardsCalculationWrong {},

    #[error("We are missing the validator rewards info")]
    ValidatorRewardsIsMissing {},

    #[error("An unknown reply ID was received.")]
    UnknownReplyID {},
}
