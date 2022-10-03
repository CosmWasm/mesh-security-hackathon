use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Custom Error val: {val:?}")]
    CustomError { val: String },

    #[error("Incorrect coin denom")]
    IncorrectDenom {},

    #[error("Contract has run out of funds to delegate for consumer chain")]
    NoFundsToDelegate {},

    #[error("Cannot undelegate from a a validator that does not have delegations")]
    NoDelegationsForValidator {},
}
