use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Unknown reply ID")]
    UnknownReply {},

    #[error("The market for the given coins already exists")]
    MarketAlreadyExists {},
}
