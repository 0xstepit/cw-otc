use common::factory::Config;
use cosmwasm_schema::cw_serde;
use cw_storage_plus::{Item, Map};

#[cw_serde]
/// Defines the exchangable coins in the market.
pub struct MarketPair {
    pub first_coin: String,
    pub second_coin: String,
}

/// Single object storing contract's configuration.
pub const CONFIG: Item<Config> = Item::new("config");

/// Store used to temporarily store the key of a market that is being created.
pub const TMP_MARKET_KEY: Item<(String, String)> = Item::new("tmp_market_key");

// Store all available markets created through the factory.
pub const MARKETS: Map<(String, String), String> = Map::new("markets");
