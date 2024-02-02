use cosmwasm_std::{StdError, Uint128};
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
    CoinError {
        first_coin: String,
        second_coin: String,
    },

    #[error("Only one coin is accepted for the deposit")]
    FundsError {},

    #[error("Sent coin is not allowed")]
    CoinNotAllowed {},

    #[error("Deal not available: expired or already matched")]
    DealNotAvailable {},

    #[error("Sent coins not allowed. Expected {amount}{denom}")]
    WrongCoin { denom: String, amount: Uint128 },

    #[error("The deal ")]
    DealNotMatched {},

    #[error("Creator cannot accept the deal")]
    SenderIsCreator {},
}
