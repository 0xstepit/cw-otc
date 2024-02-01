use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Coin};

/// This struct contains configuration parameters for the market.
#[cw_serde]
pub struct Config {
    // Address of the instantiatooor of the contract. It should be the factory contract.
    pub owner: Addr,
    // First coin exchanged in this market.
    pub first_coin: String, 
    // Second coin exchanged in this market.
    pub second_coin: String,
    // Fee deducted from each exchange in bps.
    pub fee: u16,
}

#[cw_serde]
pub struct Deal {
    // Coin that the user wants to swap.
    coin_in: Coin,
    // Coin that the user wants to receive.
    coin_out: Coin,
    // Only address that can accept the deal.
    counterparty: Option<Addr>,
    // Time after which the deal expire.
    timeout: u64,
}