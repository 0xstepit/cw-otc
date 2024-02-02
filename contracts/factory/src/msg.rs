use common::factory::Config;
use cosmwasm_schema::{cw_serde, QueryResponses};

/// This struct contains required variables to instantiate a new factory.
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
    /// Allows to update the contract's configuration. Only owner can update.
    UpdateConfig {
        new_owner: Option<String>,
        new_fee_collector: Option<String>,
    },
    CreateMarket {},
}

/// This enum describes available contract's query messages.
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Retrieve the contract allowed token.
    #[returns(Config)]
    Config {},
    #[returns(String)]
    /// Retrieve all markets.
    Markets {},
}
