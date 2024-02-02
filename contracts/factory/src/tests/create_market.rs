use cosmwasm_std::{Addr, Coin, Decimal, Empty, Uint128};
use cw_multi_test::{App, BankSudo, Contract, ContractWrapper, Executor, SudoMsg};

use crate::{
    error::ContractError,
    msg::{ExecuteMsg, InstantiateMsg, MarketsResponse, QueryMsg},
};

use market::msg::QueryMsg as MarketQueryMsg;

// Creates a market contract.
pub fn market_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        market::contract::execute,
        market::contract::instantiate,
        market::contract::query,
    );
    Box::new(contract)
}

// Creates a factory contract
pub fn factory_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        crate::contract::execute,
        crate::contract::instantiate,
        crate::contract::query,
    )
    .with_reply_empty(crate::contract::reply);

    Box::new(contract)
}

#[test]
fn create_market_works() {
    let mut app: App = App::default();

    let owner = Addr::unchecked("owner".to_string());
    let stepit = Addr::unchecked("0xstepit".to_string());
    let not_a_scammer = Addr::unchecked("0xtrustme".to_string());

    // Store the market contract.
    let market_id = app.store_code(market_contract());

    // Store and instantiate the factory contract.
    let factory_id = app.store_code(factory_contract());
    let init_factory_msg = InstantiateMsg {
        owner: owner.to_string(),
        market_code_id: market_id,
        fee_collector: Some(owner.to_string()),
    };
    let factory_addr = app
        .instantiate_contract(
            factory_id,
            owner.clone(),
            &init_factory_msg,
            &[],
            "factory-otc",
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

    let create_market_msg = ExecuteMsg::CreateMarket {
        first_coin: "astro".to_string(),
        second_coin: "usdc".to_string(),
        fee: Decimal::percent(3),
    };
    app.execute_contract(owner.clone(), factory_addr.clone(), &create_market_msg, &[])
        .unwrap();

    let resp_markets: MarketsResponse = app
        .wrap()
        .query_wasm_smart(factory_addr.clone(), &QueryMsg::Markets {})
        .unwrap();

    let ((_, _), market_addr) = resp_markets.markets[0].clone();

    let resp_deals: market::msg::AllDealsResponse = app
        .wrap()
        .query_wasm_smart(market_addr.clone(), &MarketQueryMsg::AllDeals {})
        .unwrap();

    assert_eq!(
        resp_deals.deals.len(),
        0,
        "expected zero deal from first creator because expired"
    );
}

#[test]
fn create_market_handle_errors() {
    let mut app: App = App::default();

    let owner = Addr::unchecked("owner".to_string());
    let stepit = Addr::unchecked("0xstepit".to_string());
    let not_a_scammer = Addr::unchecked("0xtrustme".to_string());

    // Store the market contract.
    let market_id = app.store_code(market_contract());

    // Store and instantiate the factory contract.
    let factory_id = app.store_code(factory_contract());
    let init_factory_msg = InstantiateMsg {
        owner: owner.to_string(),
        market_code_id: market_id,
        fee_collector: Some(owner.to_string()),
    };
    let factory_addr = app
        .instantiate_contract(
            factory_id,
            owner.clone(),
            &init_factory_msg,
            &[],
            "factory-otc",
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

    let create_market_msg = ExecuteMsg::CreateMarket {
        first_coin: "astro".to_string(),
        second_coin: "usdc".to_string(),
        fee: Decimal::percent(3),
    };
    let err = app
        .execute_contract(
            stepit.clone(),
            factory_addr.clone(),
            &create_market_msg,
            &[],
        )
        .unwrap_err();

    assert_eq!(
        err.downcast_ref::<ContractError>().unwrap(),
        &ContractError::Unauthorized {},
        "expected error because stepit is not owner"
    );

    app.execute_contract(owner.clone(), factory_addr.clone(), &create_market_msg, &[])
        .unwrap();

    let err = app
        .execute_contract(owner.clone(), factory_addr.clone(), &create_market_msg, &[])
        .unwrap_err();

    assert_eq!(
        err.downcast_ref::<ContractError>().unwrap(),
        &ContractError::MarketAlreadyExists {},
        "expected error because market already exists"
    );
}
