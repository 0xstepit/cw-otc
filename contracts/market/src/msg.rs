use common::{factory::Config, market::Deal};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Coin};

/// This enum describes available contract's execution messages.
#[cw_serde]
pub enum ExecuteMsg {
    /// Allows to update the contract's configuration. Only owner can update.
    CreateDeal {
        /// Coin that the user wants to receive.
        coin_out: Coin,
        /// If specified, is the only counterparty accepted in the deal.
        counterparty: Option<String>,
        /// Duration in blocks for the deal.
        timeout: u64,
    },
    /// Allows to accept a deal.
    AcceptDeal {
        /// Address of the deal creator.
        creator: String,
        /// Coin that the user wants to exchange for.
        deal_id: u64,
    },
    /// Allows to withdraw tokens associated with a deal.
    Withdraw {
        /// Address of the deal creator.
        creator: String,
        /// Coin that the user wants to exchange for.
        deal_id: u64,
    },
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
    DealsByCreator { creator: String },
    /// Retrieve all available deals.
    #[returns(AllDealsResponse)]
    AllDeals {},
}
