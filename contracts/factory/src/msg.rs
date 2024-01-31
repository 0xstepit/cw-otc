use common::factory::Config;
use cosmwasm_schema::{cw_serde, QueryResponses};

/// This struct contains required variables to instantiate a new contract.
#[cw_serde]
pub struct InstantiateMsg {
    /// Contract owner address.
    pub owner: String,
    /// The address that will receive otc markets fees.
    pub fee_collector: Option<String>,
}

/// This enum describes available contract's execution messages.
#[cw_serde]
pub enum ExecuteMsg {
    /// Allows to update the contract's `allowed_token`. Only owner can update.
    UpdateConfig {},
    CreateMarket {},
}

/// This enum describes available contract's query messages.
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Retrieve the contract allowed token.
    #[returns(Config)]
    QueryConfig {},
    #[returns(String)]
    /// Retrieve all markets.
    QueryMarkets {},
}
