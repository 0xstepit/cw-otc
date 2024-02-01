use cosmwasm_std::StdError;
use thiserror::Error;

use crate::contract::MAX_FEE;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Market fee cannot exceeds maximum allowed of {} bps", MAX_FEE)]
    OverFeeMax {},

    #[error("First coin {first_coin} is equal to second coin {second_coin}")]
    CoinError { first_coin: String, second_coin: String},

    #[error("Only one coin is accepted for the deposit.")]
    FundsError {  },

    #[error("Sent coin is not allowed")]
    CoinNotAllowed {  },
}
