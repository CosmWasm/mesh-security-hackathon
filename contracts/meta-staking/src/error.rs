use cosmwasm_std::StdError;
use cw_utils::ParseReplyError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error(transparent)]
    ParseReplyError(#[from] ParseReplyError),

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

    #[error("An unknown reply ID was received.")]
    UnknownReplyID {},
}
