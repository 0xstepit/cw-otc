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
        /// New contract owner.
        new_owner: Option<String>,
        /// New fee collector address.
        new_fee_collector: Option<String>,
    },
    /// Allows to instantiate a new market contract. The order of the coin is not relevant.
    CreateMarket {
        /// First coin exchanged in the market.
        first_coin: String,
        /// Second coins exchanged in the market.
        second_coin: String,
        /// Fee deducted from each clsoed deal.
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
    #[returns(MarketResponse)]
    /// Retrieve all markets.
    Market {
        first_denom: String,
        second_denom: String,
    },
    #[returns(AllMarketsResponse)]
    /// Retrieve all markets.
    AllMarkets {},
}

#[cw_serde]
pub struct MarketResponse {
    /// Address of the marekt if exists or empty.
    pub address: String,
}

#[cw_serde]
pub struct AllMarketsResponse {
    /// List all available markets.
    pub markets: Vec<((String, String), String)>,
}
