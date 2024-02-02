use common::factory::Config;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Decimal;

/// This struct contains required variables to instantiate a new factory.
#[cw_serde]
pub struct InstantiateMsg {
    /// Contract owner address.
    pub owner: String,
    /// Code ID of the otc market contract.
    pub market_code_id: u64,
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
    CreateMarket {
        first_coin: String,
        second_coin: String,
        fee: Decimal,
    },
}

/// This enum describes available contract's query messages.
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Retrieve the contract allowed token.
    #[returns(Config)]
    Config {},
    #[returns(MarketsResponse)]
    /// Retrieve all markets.
    Markets {},
}

#[cw_serde]
pub struct MarketsResponse {
    pub markets: Vec<((String, String), String)>,
}
