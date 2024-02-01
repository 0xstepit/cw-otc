use common::{factory::Config, market::Deal};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Coin, Decimal};

/// This struct contains required variables to instantiate a new market.
#[cw_serde]
pub struct InstantiateMsg {
    // First coin exchanged in this market.
    pub first_coin: String, 
    // Second coin exchanged in this market.
    pub second_coin: String,
    // Fee deducted from each exchange in bps.
    pub fee: Decimal,
}

/// This enum describes available contract's execution messages.
#[cw_serde]
pub enum ExecuteMsg {
    /// Allows to update the contract's configuration. Only owner can update.
    CreateDeal {
        // Coin that the user wants to receive.
        coin_out: Coin,
        counterparty: Option<String>,
        // Duration in blocks for the deal.
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

#[cw_serde]
pub struct DealsByCreatorResponse {
    pub deals: Vec<(u64, Deal)>,
}

#[cw_serde]
pub struct AllDealsResponse {
    pub deals: Vec<((Addr, u64), Deal)>,
}

/// This enum describes available contract's query messages.
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Retrieve the market configuration.
    #[returns(Config)]
    Config {},
    #[returns(DealsByCreatorResponse)]
    /// Retrieve all deals from a creator.
    DealsByCreator{ creator: String },
    /// Retrieve all available deals.
    #[returns(AllDealsResponse)]
    AllDeals{  },
}
