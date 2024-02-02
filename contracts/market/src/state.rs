use common::market::{Config, Deal};
use cosmwasm_std::{Addr, StdResult, Storage};
use cw_storage_plus::{Item, Map};

/// Retrieve the number of the next deal to be created and increment the counter by one.
pub fn next_id(store: &mut dyn Storage) -> StdResult<u64> {
    let id = COUNTER.may_load(store)?.unwrap_or_default();
    COUNTER.save(store, &(id + 1))?;
    Ok(id)
}

/// Data structure used to store the number of created deals.
pub const COUNTER: Item<u64> = Item::new("counter");
/// Data structure used to store all deals.
pub const DEALS: Map<(&Addr, u64), Deal> = Map::new("deals");
/// Single object storing contract's configuration.
pub const CONFIG: Item<Config> = Item::new("config");
