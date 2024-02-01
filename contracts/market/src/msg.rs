use common::{factory::Config, market::Deal};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Coin;

/// This struct contains required variables to instantiate a new market.
#[cw_serde]
pub struct InstantiateMsg {
    // First coin exchanged in this market.
    pub first_coin: String, 
    // Second coin exchanged in this market.
    pub second_coin: String,
    // Fee deducted from each exchange in bps.
    pub fee: u16,
}

/// This enum describes available contract's execution messages.
#[cw_serde]
pub enum ExecuteMsg {
    /// Allows to update the contract's configuration. Only owner can update.
    CreateDeal {
        // Coin that the user wants to receive.
        coin_out: Coin,
        counterparty: Option<String>,
        // Duration of the deal.
        timeout: u64,
    },
    AcceptDeal {
        // Address of the deal creator.
        //creator: String,
        // Coin that the user wants to exchange for.
        //coin_in: Coin,
    },
    Withdraw {},
}

/// This enum describes available contract's query messages.
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Retrieve the market configuration.
    #[returns(Config)]
    QueryConfig {},
    #[returns(Vec<Deal>)]
    /// Retrieve all available deals.
    QueryAllDeals{},
}
