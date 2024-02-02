use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Coin, Decimal};

/// This struct contains required variables to instantiate a new market.
#[cw_serde]
pub struct InstantiateMsg {
    /// First coin exchanged in this market.
    pub first_coin: String,
    /// Second coin exchanged in this market.
    pub second_coin: String,
    /// Fee deducted from each exchange in bps.
    pub fee: Decimal,
}

/// This struct contains configuration parameters for the market.
#[cw_serde]
pub struct Config {
    /// Address of the instantiatooor of the contract. It should be the factory contract.
    pub owner: Addr,
    /// First coin exchanged in this market.
    pub first_coin: String,
    /// Second coin exchanged in this market.
    pub second_coin: String,
    /// Fee deducted from each exchange in percentage.
    pub fee: Decimal,
}

/// Contains all information of a Deal.
#[cw_serde]
pub struct Deal {
    /// Coin that the user wants to swap.
    pub coin_in: Coin,
    /// Coin that the user wants to receive.
    pub coin_out: Coin,
    /// Only address that can accept the deal.
    pub counterparty: Option<Addr>,
    /// Block after which the deal expire.
    pub timeout: u64,
    /// Already matched by a counterparty.
    pub status: DealStatus,
}

/// Describes the possible status of a deal.
#[cw_serde]
pub enum DealStatus {
    NotMatched,
    Matched(WithdrawStatus),
}

/// Describes the possible status of a matched deal.
#[cw_serde]
pub enum WithdrawStatus {
    /// No one performed a withdraw.
    NoWithdraw,
    /// Only the creator performed a withdraw.
    CreatorWithdrawed,
    /// Only the counterparty performed the withdraw.
    CounterpartyWithdrawed,
    /// Closed deal.
    Completed,
}

impl DealStatus {
    pub fn matched_no_withdraw() -> Self {
        DealStatus::Matched(WithdrawStatus::NoWithdraw)
    }

    pub fn matched_creator_withdraw() -> Self {
        DealStatus::Matched(WithdrawStatus::CreatorWithdrawed)
    }

    pub fn matched_counterparty_withdraw() -> Self {
        DealStatus::Matched(WithdrawStatus::CounterpartyWithdrawed)
    }

    pub fn matched_and_completed() -> Self {
        DealStatus::Matched(WithdrawStatus::Completed)
    }
}
