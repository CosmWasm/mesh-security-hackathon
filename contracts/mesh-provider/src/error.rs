use cosmwasm_std::StdError;
use thiserror::Error;

use mesh_ibc::MeshSecurityError;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    MeshSecurity(#[from] MeshSecurityError),

    #[error("Unauthorized")]
    Unauthorized {},

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
    // Add any other custom errors you like here.
    // Look at https://docs.rs/thiserror/1.0.21/thiserror/ for details.
}
