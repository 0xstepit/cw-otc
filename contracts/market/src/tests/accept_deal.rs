use cosmwasm_std::{Addr, Coin, Decimal, Empty, Uint128};
use cw_multi_test::{App, BankSudo, Contract, ContractWrapper, Executor, SudoMsg};

use crate::{error::ContractError, msg::ExecuteMsg};

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
fn accept_deal_without_counterparty_works() {
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

    // Mint tokens to creator and counterparty
    let mut coin = Coin {
        denom: "astro".to_string(),
        amount: Uint128::new(1_000),
    };
    app.sudo(SudoMsg::Bank(BankSudo::Mint {
        to_address: not_a_scammer.to_string(),
        amount: vec![coin.clone()],
    }))
    .unwrap();

    coin.denom = "usdc".to_string();
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

    // Accept the deal
    let accept_deal_msg = ExecuteMsg::AcceptDeal {
        creator: not_a_scammer.to_string(),
        deal_id: 0,
    };
    app.execute_contract(
        stepit.clone(),
        market_addr.clone(),
        &accept_deal_msg,
        &[Coin::new(1_000, "usdc")],
    )
    .unwrap();
}

#[test]
fn accept_deal_with_counterparty_works() {
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

    // Mint tokens to creator and counterparty
    let mut coin = Coin {
        denom: "astro".to_string(),
        amount: Uint128::new(1_000),
    };
    app.sudo(SudoMsg::Bank(BankSudo::Mint {
        to_address: not_a_scammer.to_string(),
        amount: vec![coin.clone()],
    }))
    .unwrap();

    coin.denom = "usdc".to_string();
    app.sudo(SudoMsg::Bank(BankSudo::Mint {
        to_address: stepit.to_string(),
        amount: vec![coin],
    }))
    .unwrap();

    // Create first deal
    let create_deal_msg = ExecuteMsg::CreateDeal {
        coin_out: Coin::new(1_000, "usdc"),
        counterparty: Some(stepit.to_string()),
        timeout: 10,
    };
    app.execute_contract(
        not_a_scammer.clone(),
        market_addr.clone(),
        &create_deal_msg,
        &[Coin::new(1_000, "astro")],
    )
    .unwrap();

    // Accept the deal
    let accept_deal_msg = ExecuteMsg::AcceptDeal {
        creator: not_a_scammer.to_string(),
        deal_id: 0,
    };
    app.execute_contract(
        stepit.clone(),
        market_addr.clone(),
        &accept_deal_msg,
        &[Coin::new(1_000, "usdc")],
    )
    .unwrap();
}

#[test]
fn accept_deal_error_handling() {
    let mut app: App = App::default();

    let owner = Addr::unchecked(OWNER);
    let stepit = Addr::unchecked("0xstepit".to_string());
    let spiderman = Addr::unchecked("0xspider".to_string());
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

    // Mint tokens to creator and counterparty
    let mut coin = Coin {
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
        amount: vec![coin.clone()],
    }))
    .unwrap();

    coin.denom = "usdc".to_string();
    app.sudo(SudoMsg::Bank(BankSudo::Mint {
        to_address: not_a_scammer.to_string(),
        amount: vec![coin.clone()],
    }))
    .unwrap();
    app.sudo(SudoMsg::Bank(BankSudo::Mint {
        to_address: stepit.to_string(),
        amount: vec![coin.clone()],
    }))
    .unwrap();
    app.sudo(SudoMsg::Bank(BankSudo::Mint {
        to_address: spiderman.to_string(),
        amount: vec![coin],
    }))
    .unwrap();

    // Create first deal
    let create_deal_msg = ExecuteMsg::CreateDeal {
        coin_out: Coin::new(500, "usdc"),
        counterparty: Some(stepit.to_string()),
        timeout: 10,
    };
    app.execute_contract(
        not_a_scammer.clone(),
        market_addr.clone(),
        &create_deal_msg,
        &[Coin::new(1_000, "astro")],
    )
    .unwrap();

    // Let the deal expire
    app.update_block(|block| {
        block.height += 11;
        block.time = block.time.plus_seconds(11 * 5);
    });

    let accept_deal_msg = ExecuteMsg::AcceptDeal {
        creator: not_a_scammer.to_string(),
        deal_id: 0,
    };
    let err = app
        .execute_contract(
            stepit.clone(),
            market_addr.clone(),
            &accept_deal_msg,
            &[Coin::new(500, "usdc")],
        )
        .unwrap_err();

    assert_eq!(
        err.downcast_ref::<ContractError>().unwrap(),
        &ContractError::DealNotAvailable {},
        "expected error because deal expired"
    );

    // Time machine to go back when deal is not expired
    app.update_block(|block| {
        block.height -= 11;
        block.time = block.time.minus_seconds(11 * 5);
    });

    let err = app
        .execute_contract(
            stepit.clone(),
            market_addr.clone(),
            &accept_deal_msg,
            &[Coin::new(499, "usdc")],
        )
        .unwrap_err();

    assert_eq!(
        err.downcast_ref::<ContractError>().unwrap(),
        &ContractError::WrongCoin {
            denom: "usdc".to_string(),
            amount: Uint128::new(500)
        },
        "expected error because sent tokens are less than the requested"
    );

    let err = app
        .execute_contract(
            stepit.clone(),
            market_addr.clone(),
            &accept_deal_msg,
            &[Coin::new(500, "astro")],
        )
        .unwrap_err();

    assert_eq!(
        err.downcast_ref::<ContractError>().unwrap(),
        &ContractError::WrongCoin {
            denom: "usdc".to_string(),
            amount: Uint128::new(500)
        },
        "expected error because sent token is different than the requested"
    );

    let err = app
        .execute_contract(
            spiderman.clone(),
            market_addr.clone(),
            &accept_deal_msg,
            &[Coin::new(500, "usdc")],
        )
        .unwrap_err();

    assert_eq!(
        err.downcast_ref::<ContractError>().unwrap(),
        &ContractError::Unauthorized {},
        "expected error because sender is not the requested counterparty"
    );

    let err = app
        .execute_contract(
            not_a_scammer.clone(),
            market_addr.clone(),
            &accept_deal_msg,
            &[Coin::new(500, "usdc")],
        )
        .unwrap_err();

    assert_eq!(
        err.downcast_ref::<ContractError>().unwrap(),
        &ContractError::SenderIsCreator {},
        "expected error because creator cannot accept their deal"
    );

    // Now the deal is accepted correctly
    app.execute_contract(
        stepit.clone(),
        market_addr.clone(),
        &accept_deal_msg,
        &[Coin::new(500, "usdc")],
    )
    .unwrap();

    let err = app
        .execute_contract(
            stepit.clone(),
            market_addr.clone(),
            &accept_deal_msg,
            &[Coin::new(500, "usdc")],
        )
        .unwrap_err();

    assert_eq!(
        err.downcast_ref::<ContractError>().unwrap(),
        &ContractError::DealNotAvailable {},
        "expected error because deal matched"
    );
}
