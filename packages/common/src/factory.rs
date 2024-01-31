use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;

/// This struct contains configuration parameters for the contract.
#[cw_serde]
pub struct Config {
    // Contract owner
    pub owner: Addr,
    // Optional address used to collect markets fees.
    pub fee_collector: Option<Addr>,
}
