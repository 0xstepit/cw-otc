use cosmwasm_std::{Addr, Coin, Decimal, Empty, Uint128};
use cw_multi_test::{App, BankSudo, Contract, ContractWrapper, Executor, SudoMsg};

use crate::{
    error::ContractError,
    msg::{AllDealsResponse, DealsByCreatorResponse, ExecuteMsg, QueryMsg},
};

use common::market::InstantiateMsg;

const OWNER: &str = "0xstepit000";

// Creates a market contract.
pub fn market_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        crate::contract::execute,
        crate::contract::instantiate,
        crate::contract::query,
    );
    Box::new(contract)
}

#[test]
fn create_deal_works() {
    let mut app: App = App::default();

    let owner = Addr::unchecked(OWNER);
    let stepit = Addr::unchecked("0xstepit".to_string());
    let not_a_scammer = Addr::unchecked("0xtrustme".to_string());

    // Store and instantiate the market contract.
    let market_id = app.store_code(market_contract());
    let init_market_msg = InstantiateMsg {
        first_coin: "astro".to_string(),
        second_coin: "usdc".to_string(),
        fee: Decimal::percent(2),
    };
    let market_addr = app
        .instantiate_contract(
            market_id,
            owner.clone(),
            &init_market_msg,
            &[],
            "otc-market",
            None,
        )
        .unwrap();

    // Mint tokens to two accounts.
    let coin = Coin {
        denom: "astro".to_string(),
        amount: Uint128::new(1_000),
    };
    app.sudo(SudoMsg::Bank(BankSudo::Mint {
        to_address: not_a_scammer.to_string(),
        amount: vec![coin.clone()],
    }))
    .unwrap();
    app.sudo(SudoMsg::Bank(BankSudo::Mint {
        to_address: stepit.to_string(),
        amount: vec![coin],
    }))
    .unwrap();

    // Create first deal
    let create_deal_msg = ExecuteMsg::CreateDeal {
        coin_out: Coin::new(1_000, "usdc"),
        counterparty: None,
        timeout: 10,
    };
    app.execute_contract(
        not_a_scammer.clone(),
        market_addr.clone(),
        &create_deal_msg,
        &[Coin::new(1_000, "astro")],
    )
    .unwrap();

    // Create second deal with another account
    let create_deal_msg = ExecuteMsg::CreateDeal {
        coin_out: Coin::new(1_000, "usdc"),
        counterparty: None,
        timeout: 100,
    };
    app.execute_contract(
        stepit.clone(),
        market_addr.clone(),
        &create_deal_msg,
        &[Coin::new(1_000, "astro")],
    )
    .unwrap();

    let resp: DealsByCreatorResponse = app
        .wrap()
        .query_wasm_smart(
            market_addr.clone(),
            &QueryMsg::DealsByCreator {
                creator: not_a_scammer.to_string(),
            },
        )
        .unwrap();
    let resp_all: AllDealsResponse = app
        .wrap()
        .query_wasm_smart(market_addr.clone(), &QueryMsg::AllDeals {})
        .unwrap();

    assert_eq!(resp.deals.len(), 1, "expected one deal from the creator");
    assert_eq!(resp_all.deals.len(), 2, "expected two deals");

    // Let the deal expire (assumption 1 block == 5s)
    app.update_block(|block| {
        block.height += 11;
        block.time = block.time.plus_seconds(11 * 5);
    });

    let resp: DealsByCreatorResponse = app
        .wrap()
        .query_wasm_smart(
            market_addr.clone(),
            &QueryMsg::DealsByCreator {
                creator: not_a_scammer.to_string(),
            },
        )
        .unwrap();
    let resp_all: AllDealsResponse = app
        .wrap()
        .query_wasm_smart(market_addr.clone(), &QueryMsg::AllDeals {})
        .unwrap();

    assert_eq!(
        resp.deals.len(),
        0,
        "expected zero deal from first creator because expired"
    );
    assert_eq!(resp_all.deals.len(), 1, "expected one deal still active");
}

#[test]
fn create_deal_handle_errors() {
    let mut app: App = App::default();

    let owner = Addr::unchecked(OWNER);
    let not_a_scammer = Addr::unchecked("0xtrustme".to_string());

    // Store and instantiate the market contract.
    let market_id = app.store_code(market_contract());
    let init_market_msg = InstantiateMsg {
        first_coin: "astro".to_string(),
        second_coin: "usdc".to_string(),
        fee: Decimal::percent(2),
    };
    let market_addr = app
        .instantiate_contract(
            market_id,
            owner.clone(),
            &init_market_msg,
            &[],
            "otc-market",
            None,
        )
        .unwrap();

    // Mint tokens to two accounts.
    let mut coin = Coin {
        denom: "astro".to_string(),
        amount: Uint128::new(1_000),
    };
    app.sudo(SudoMsg::Bank(BankSudo::Mint {
        to_address: not_a_scammer.to_string(),
        amount: vec![coin.clone()],
    }))
    .unwrap();
    coin.denom = "osmo".to_string();
    app.sudo(SudoMsg::Bank(BankSudo::Mint {
        to_address: not_a_scammer.to_string(),
        amount: vec![coin],
    }))
    .unwrap();

    // The deal is valid
    let create_deal_msg = ExecuteMsg::CreateDeal {
        coin_out: Coin::new(1_000, "usdc"),
        counterparty: None,
        timeout: 10,
    };
    let err = app
        .execute_contract(
            not_a_scammer.clone(),
            market_addr.clone(),
            &create_deal_msg,
            &[Coin::new(1_000, "astro"), Coin::new(1_000, "osmo")],
        )
        .unwrap_err();

    assert_eq!(
        err.downcast_ref::<ContractError>().unwrap(),
        &ContractError::FundsError {},
        "expected error because sent two coins"
    );

    let create_deal_msg = ExecuteMsg::CreateDeal {
        coin_out: Coin::new(1_000, "osmo"),
        counterparty: None,
        timeout: 100,
    };
    let err = app
        .execute_contract(
            not_a_scammer.clone(),
            market_addr.clone(),
            &create_deal_msg,
            &[Coin::new(1_000, "astro")],
        )
        .unwrap_err();

    assert_eq!(
        err.downcast_ref::<ContractError>().unwrap(),
        &ContractError::CoinNotAllowed {},
        "expected error because output coin not allowed"
    );

    let create_deal_msg = ExecuteMsg::CreateDeal {
        coin_out: Coin::new(1_000, "astro"),
        counterparty: None,
        timeout: 100,
    };
    let err = app
        .execute_contract(
            not_a_scammer.clone(),
            market_addr.clone(),
            &create_deal_msg,
            &[Coin::new(1_000, "osmo")],
        )
        .unwrap_err();

    assert_eq!(
        err.downcast_ref::<ContractError>().unwrap(),
        &ContractError::CoinNotAllowed {},
        "expected error because sent coin not allowed"
    );
}
