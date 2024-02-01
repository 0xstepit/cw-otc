use cosmwasm_std::{
    coin, ensure, entry_point, to_json_binary, Binary, Decimal, Deps, DepsMut, Env, MessageInfo, Response, StdResult
};

use crate::{
    error::ContractError,
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg},
    state::CONFIG,
};

use astroport::asset::validate_native_denom;

use common::market::Config;

const CONTRACT_NAME: &str = "crates.io/cw-otc-market";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

// Maximum allowed fee is 5%.
pub const MAX_FEE: Decimal = Decimal::percent(5); 

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    validate_native_denom(&msg.first_coin)?;
    validate_native_denom(&msg.second_coin)?;
    if msg.first_coin == msg.second_coin {
        return Err(ContractError::CoinError {
            first_coin: msg.first_coin,
            second_coin: msg.second_coin
        })
    }
    if msg.fee > MAX_FEE {
        return Err(ContractError::OverFeeMax {});
    };

    CONFIG.save(
        deps.storage,
        &Config {
            owner: info.sender,
            first_coin: msg.first_coin,
            second_coin: msg.second_coin,
            fee: msg.fee
        },
    )?;

    Ok(Response::new())
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    use ExecuteMsg::*;
    match msg {
        CreateDeal {
            coin_out, 
            counterparty, 
            timeout,
        } => execute::create_deal(deps, env, info, coin_out, counterparty, timeout),
        AcceptDeal {} => execute::accept_deal(deps, &info.sender),
        Withdraw {} => execute::withdraw(deps, &info.sender),
    }
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    use QueryMsg::*;
    match msg {
        QueryConfig {} => to_json_binary(&query::get_config(deps)?),
        QueryAllDeals {} => to_json_binary(&query::get_markets(deps)?),
    }
}

pub mod execute {
    use std::ops::Add;

    use common::market::Deal;
    use cosmwasm_std::{Addr, Attribute, Coin, Coins, StdError};

    use crate::state::{next_id, COUNTER, DEALS};

    use super::*;

    pub fn create_deal(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        coin_out: Coin,
        counterparty: Option<String>,
        timeout: u64,
    ) -> Result<Response, ContractError> {
        let config = CONFIG.load(deps.storage)?;

        check_only_one_coin(&info.funds)?;
        check_allowed_coin(&info.funds[0].denom, &config)?;
        check_allowed_coin(&coin_out.denom, &config)?;

        let counterparty = counterparty
            .as_ref()
            .map(|addr| deps.api.addr_validate(addr))
            .transpose()?;

        let deal = Deal {
            coin_in: info.funds[0].clone(),
            coin_out,
            counterparty,
            timeout: env.block.height.add(timeout)
        };

        let deal_id = next_id(deps.storage)?;
        DEALS.save(deps.storage, (&info.sender, deal_id), &deal)?;

        Ok(Response::new()
            .add_attribute("action", "create_dial")
            .add_attribute("deal_id", deal_id.to_string())
            .add_attribute("creator", info.sender)
        )
    }

    pub fn accept_deal(_deps: DepsMut, _sender: &Addr) -> Result<Response, ContractError> {
        unimplemented!()
    }

    pub fn withdraw(_deps: DepsMut, _sender: &Addr) -> Result<Response, ContractError> {
        unimplemented!()
    }

    // Check that only one coin has been sent to the contract.
    pub fn check_only_one_coin(funds: &Vec<Coin>) -> Result<(), ContractError>{
        if funds.len() != 1 {
            return Err(ContractError::FundsError {})
        }
        Ok(())
    }

    // Check that the denom is an allowed coin for the market.
    pub fn check_allowed_coin(denom: &str, config: &Config) -> Result<(), ContractError> {
        if denom != config.first_coin && denom != config.second_coin {
            return Err(ContractError::CoinNotAllowed {})
        }
        Ok(())
    }

}

pub mod query {
    use super::*;

    pub fn get_config(deps: Deps) -> StdResult<Config> {
        CONFIG.load(deps.storage)
    }

    pub fn get_markets(_deps: Deps) -> StdResult<()> {
        unimplemented!()
    }
}

// -------------------------------------------------------------------------------------------------
// Unit tests
// -------------------------------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use std::ops::Add;

    use cosmwasm_std::{
        testing::{mock_dependencies, mock_env, mock_info}, Addr, StdError
    };

    use crate::msg::InstantiateMsg;
    use execute;

    use super::*;

    const OWNER: &str = "0xstepit000";

    #[test]
    fn instatiate_native_works() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("stepit", &[]);

        instantiate(
            deps.as_mut(),
            env,
            info,
            InstantiateMsg {
                first_coin: "astro".to_owned(),
                second_coin: "usdc".to_owned(),
                fee: Decimal::percent(1)
            },
        )
        .unwrap();

        let config = CONFIG.load(deps.as_ref().storage).unwrap();
        let expected_config = Config {
            owner: Addr::unchecked("stepit"),
            first_coin: "astro".to_owned(),
            second_coin: "usdc".to_owned(),
            fee: Decimal::percent(1)
        };
        assert_eq!(expected_config, config, "expected different config")
    }

    #[test]
    fn instatiate_ibc_token_works() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("stepit", &[]);

        instantiate(
            deps.as_mut(),
            env,
            info,
            InstantiateMsg {
                first_coin: "ibc/EBD5A24C554198EBAF44979C5B4D2C2D312E6EBAB71962C92F735499C7575839".to_owned(),
                second_coin: "usdc".to_owned(),
                fee: Decimal::percent(1)
            },
        )
        .unwrap();

        let config = CONFIG.load(deps.as_ref().storage).unwrap();
        let expected_config = Config {
            owner: Addr::unchecked("stepit"),
            first_coin: "ibc/EBD5A24C554198EBAF44979C5B4D2C2D312E6EBAB71962C92F735499C7575839".to_owned(),
            second_coin: "usdc".to_owned(),
            fee: Decimal::percent(1)
        };
        assert_eq!(expected_config, config, "expected different config")
    }

    #[test]
    fn instatiate_tokefactory_token_works() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("stepit", &[]);

        instantiate(
            deps.as_mut(),
            env,
            info,
            InstantiateMsg {
                first_coin: "factory/wasm1jdppe6fnj2q7hjsepty5crxtrryzhuqsjrj95y/astro".to_owned(),
                second_coin: "usdc".to_owned(),
                fee: Decimal::percent(1)
            },
        )
        .unwrap();

        let config = CONFIG.load(deps.as_ref().storage).unwrap();
        let expected_config = Config {
            owner: Addr::unchecked("stepit"),
            first_coin: "factory/wasm1jdppe6fnj2q7hjsepty5crxtrryzhuqsjrj95y/astro".to_owned(),
            second_coin: "usdc".to_owned(),
            fee: Decimal::percent(1)
        };
        assert_eq!(expected_config, config, "expected different config")
    }


    #[test]
    fn instatiate_error_handling() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("stepit", &[]);

        let err = instantiate(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            InstantiateMsg {
                first_coin: "astro".to_owned(),
                second_coin: "usdc".to_owned(),
                fee: Decimal::percent(6)
            },
        )
        .unwrap_err();

        assert_eq!(err, ContractError::OverFeeMax {  }, "expected different error for over max fee");

        let err = instantiate(
            deps.as_mut(),
            env,
            info,
            InstantiateMsg {
                first_coin: "astro".to_owned(),
                second_coin: "astro".to_owned(),
                fee: Decimal::percent(1)
            },
        )
        .unwrap_err();

        assert_eq!(
            err, 
            ContractError::CoinError { first_coin: "astro".to_owned(), second_coin: "astro".to_owned() }, 
            "expected different error for same coin");
    }
}