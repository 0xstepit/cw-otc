use common::market::Config;
use cw_storage_plus::Item;

/// Single object storing contract's configuration.
pub const CONFIG: Item<Config> = Item::new("config");
