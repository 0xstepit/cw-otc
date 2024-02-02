use common::factory::Config;
use cosmwasm_schema::cw_serde;
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct MarketPair {
    pub first_coin: String,
    pub second_coin: String,
}

/// Single object storing contract's configuration.
pub const CONFIG: Item<Config> = Item::new("config");

pub const TMP_MARKET_KEY: Item<(String, String)> = Item::new("tmp_market_key");
pub const MARKETS: Map<(String, String), String> = Map::new("markets");
