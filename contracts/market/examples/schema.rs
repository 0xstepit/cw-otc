use cosmwasm_schema::write_api;
use market::msg::{ExecuteMsg, QueryMsg};
use common::market::InstantiateMsg;

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        execute: ExecuteMsg,
        query: QueryMsg
    }
}
