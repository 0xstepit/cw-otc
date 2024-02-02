use std::ops::Sub;

use cosmwasm_std::{Addr, Coin, Decimal, Empty, Uint128};
use cw_multi_test::{App, BankSudo, Contract, ContractWrapper, Executor, SudoMsg};

use crate::{
    error::ContractError,
    msg::{DealsByCreatorResponse, ExecuteMsg, InstantiateMsg, QueryMsg},
};

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
fn withdraw_works_no_fee() {
    let mut app: App = App::default();

    // SETUP

    let owner = Addr::unchecked("owner".to_string());
    let stepit = Addr::unchecked("0xstepit".to_string());
    let not_a_scammer = Addr::unchecked("0xtrustme".to_string());

    // Store and instantiate the market contract.
    let market_id = app.store_code(market_contract());
    let init_market_msg = InstantiateMsg {
        first_coin: "astro".to_string(),
        second_coin: "usdc".to_string(),
        fee: Decimal::percent(0),
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

    let creator_balance_pre = app
        .wrap()
        .query_balance(not_a_scammer.clone(), "astro")
        .unwrap();
    let counterparty_balance_pre = app.wrap().query_balance(stepit.clone(), "usdc").unwrap();

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

    let creator_deals: DealsByCreatorResponse = app
        .wrap()
        .query_wasm_smart(
            market_addr.to_string(),
            &QueryMsg::DealsByCreator {
                creator: not_a_scammer.to_string(),
            },
        )
        .unwrap();
    assert_eq!(
        creator_deals.deals.len(),
        1,
        "expected one deal from the creator"
    );

    // WITHDRAW TESTING FROM HERE

    let contract_balance = app
        .wrap()
        .query_balance(market_addr.clone(), "usdc")
        .unwrap();
    assert_eq!(contract_balance.amount, Uint128::new(1_000));
    let contract_balance = app
        .wrap()
        .query_balance(market_addr.clone(), "astro")
        .unwrap();
    assert_eq!(contract_balance.amount, Uint128::new(1_000));

    // Withdraw creator
    let withdraw_msg = ExecuteMsg::Withdraw {
        creator: not_a_scammer.to_string(),
        deal_id: 0,
    };
    app.execute_contract(
        not_a_scammer.clone(),
        market_addr.clone(),
        &withdraw_msg,
        &[],
    )
    .unwrap();

    let creator_balance = app
        .wrap()
        .query_balance(not_a_scammer.clone(), "usdc")
        .unwrap();
    assert_eq!(
        creator_balance, counterparty_balance_pre,
        "expected creator to have withdrawn usdc"
    );

    // Withdraw counterparty
    let withdraw_msg = ExecuteMsg::Withdraw {
        creator: not_a_scammer.to_string(),
        deal_id: 0,
    };
    app.execute_contract(stepit.clone(), market_addr.clone(), &withdraw_msg, &[])
        .unwrap();

    let counterparty_balance = app.wrap().query_balance(stepit, "astro").unwrap();
    assert_eq!(
        counterparty_balance, creator_balance_pre,
        "expected counterparty to have withdrawn astro"
    );

    let contract_balance = app
        .wrap()
        .query_balance(market_addr.clone(), "usdc")
        .unwrap();
    assert_eq!(
        contract_balance.amount,
        Uint128::new(0),
        "expected empty contract"
    );
    let contract_balance = app
        .wrap()
        .query_balance(market_addr.clone(), "astro")
        .unwrap();
    assert_eq!(
        contract_balance.amount,
        Uint128::new(0),
        "expected empty contract"
    );

    let creator_deals: DealsByCreatorResponse = app
        .wrap()
        .query_wasm_smart(
            market_addr.to_string(),
            &QueryMsg::DealsByCreator {
                creator: not_a_scammer.to_string(),
            },
        )
        .unwrap();
    assert_eq!(
        creator_deals.deals.len(),
        0,
        "expected no more deals from the creator"
    )
}

#[test]
fn withdraw_works_with_fee() {
    let mut app: App = App::default();

    // SETUP

    let owner = Addr::unchecked("owner".to_string());
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

    let creator_balance_pre = app
        .wrap()
        .query_balance(not_a_scammer.clone(), "astro")
        .unwrap();
    let counterparty_balance_pre = app.wrap().query_balance(stepit.clone(), "usdc").unwrap();

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

    // WITHDRAW TESTING FROM HERE

    let contract_balance = app
        .wrap()
        .query_balance(market_addr.clone(), "usdc")
        .unwrap();
    assert_eq!(contract_balance.amount, Uint128::new(1_000));
    let contract_balance = app
        .wrap()
        .query_balance(market_addr.clone(), "astro")
        .unwrap();
    assert_eq!(contract_balance.amount, Uint128::new(1_000));

    // Withdraw creator
    let withdraw_msg = ExecuteMsg::Withdraw {
        creator: not_a_scammer.to_string(),
        deal_id: 0,
    };
    app.execute_contract(
        not_a_scammer.clone(),
        market_addr.clone(),
        &withdraw_msg,
        &[],
    )
    .unwrap();

    let creator_balance = app
        .wrap()
        .query_balance(not_a_scammer.clone(), "usdc")
        .unwrap();
    assert_eq!(
        creator_balance.amount,
        counterparty_balance_pre.amount.sub(Uint128::new(20)),
        "expected creator to have withdrawn usdc less the fee"
    );

    // Withdraw counterparty
    let withdraw_msg = ExecuteMsg::Withdraw {
        creator: not_a_scammer.to_string(),
        deal_id: 0,
    };
    app.execute_contract(stepit.clone(), market_addr.clone(), &withdraw_msg, &[])
        .unwrap();

    let counterparty_balance = app.wrap().query_balance(stepit, "astro").unwrap();
    assert_eq!(
        counterparty_balance.amount,
        creator_balance_pre.amount.sub(Uint128::new(20)),
        "expected counterparty to have withdrawn astro less the fee"
    );

    let contract_balance = app
        .wrap()
        .query_balance(market_addr.clone(), "usdc")
        .unwrap();
    assert_eq!(
        contract_balance.amount,
        Uint128::new(0),
        "expected empty contract"
    );
    let contract_balance = app
        .wrap()
        .query_balance(market_addr.clone(), "astro")
        .unwrap();
    assert_eq!(
        contract_balance.amount,
        Uint128::new(0),
        "expected empty contract"
    );

    let owner_balance = app.wrap().query_balance(owner.clone(), "usdc").unwrap();
    assert_eq!(
        owner_balance.amount,
        Uint128::new(20),
        "expected owner to accrue usdc fee"
    );

    let owner_balance = app.wrap().query_balance(owner.clone(), "astro").unwrap();
    assert_eq!(
        owner_balance.amount,
        Uint128::new(20),
        "expected owner to accrue astro fee"
    );
}

#[test]
fn withdraw_before_matching() {
    let mut app: App = App::default();

    // SETUP

    let owner = Addr::unchecked("owner".to_string());
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
    let coin = Coin {
        denom: "astro".to_string(),
        amount: Uint128::new(1_000),
    };
    app.sudo(SudoMsg::Bank(BankSudo::Mint {
        to_address: not_a_scammer.to_string(),
        amount: vec![coin.clone()],
    }))
    .unwrap();

    let creator_balance_pre = app
        .wrap()
        .query_balance(not_a_scammer.clone(), "astro")
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

    // WITHDRAW TESTING FROM HERE

    let contract_balance = app
        .wrap()
        .query_balance(market_addr.clone(), "astro")
        .unwrap();
    assert_eq!(contract_balance.amount, Uint128::new(1_000));

    // Withdraw creator
    let withdraw_msg = ExecuteMsg::Withdraw {
        creator: not_a_scammer.to_string(),
        deal_id: 0,
    };
    app.execute_contract(
        not_a_scammer.clone(),
        market_addr.clone(),
        &withdraw_msg,
        &[],
    )
    .unwrap();

    let creator_balance = app
        .wrap()
        .query_balance(not_a_scammer.clone(), "astro")
        .unwrap();
    assert_eq!(
        creator_balance.amount, creator_balance_pre.amount,
        "expected creator to have withdrawn deposited amount"
    );

    let contract_balance = app
        .wrap()
        .query_balance(market_addr.clone(), "astro")
        .unwrap();
    assert_eq!(
        contract_balance.amount,
        Uint128::new(0),
        "expected empty contract"
    );
}

#[test]
fn withdraw_handle_errors() {
    let mut app: App = App::default();

    // SETUP

    let owner = Addr::unchecked("owner".to_string());
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

    // WITHDRAW TESTING FROM HERE

    let contract_balance = app
        .wrap()
        .query_balance(market_addr.clone(), "usdc")
        .unwrap();
    assert_eq!(contract_balance.amount, Uint128::new(1_000));
    let contract_balance = app
        .wrap()
        .query_balance(market_addr.clone(), "astro")
        .unwrap();
    assert_eq!(contract_balance.amount, Uint128::new(1_000));

    // Withdraw from random
    let withdraw_msg = ExecuteMsg::Withdraw {
        creator: not_a_scammer.to_string(),
        deal_id: 0,
    };
    let err = app
        .execute_contract(owner.clone(), market_addr.clone(), &withdraw_msg, &[])
        .unwrap_err();

    assert_eq!(
        err.downcast_ref::<ContractError>().unwrap(),
        &ContractError::Unauthorized {},
        "expected error because not part of the deal"
    );

    // Withdraw creator
    let withdraw_msg = ExecuteMsg::Withdraw {
        creator: not_a_scammer.to_string(),
        deal_id: 0,
    };
    app.execute_contract(
        not_a_scammer.clone(),
        market_addr.clone(),
        &withdraw_msg,
        &[],
    )
    .unwrap();

    // Second withdraw from the creator
    let err = app
        .execute_contract(
            not_a_scammer.clone(),
            market_addr.clone(),
            &withdraw_msg,
            &[],
        )
        .unwrap_err();

    assert_eq!(
        err.downcast_ref::<ContractError>().unwrap(),
        &ContractError::Unauthorized {},
        "expected error because second withdraw"
    );

    // Withdraw counterparty
    let withdraw_msg = ExecuteMsg::Withdraw {
        creator: not_a_scammer.to_string(),
        deal_id: 0,
    };
    app.execute_contract(stepit.clone(), market_addr.clone(), &withdraw_msg, &[])
        .unwrap();

    // Second withdraw from the counterparty. Expected error because deal removed.
    app.execute_contract(stepit.clone(), market_addr.clone(), &withdraw_msg, &[])
        .unwrap_err();
}
