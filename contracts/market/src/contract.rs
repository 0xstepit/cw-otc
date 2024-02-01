use cosmwasm_std::{
    ensure, entry_point, to_json_binary, Binary, Decimal, Deps, DepsMut, Env, MessageInfo, Response, StdResult
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
pub const MAX_FEE: u16 = 500; 

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
        CreateDeal {coin_out, counterparty, timeout} => execute::create_deal(deps, env, &info.sender),
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
    use cosmwasm_std::{Addr, Attribute, StdError};

    use super::*;

    pub fn create_deal(
        _deps: DepsMut,
        _env: Env,
        _sender: &Addr,
    ) -> Result<Response, ContractError> {
        unimplemented!()
    }

    pub fn accept_deal(_deps: DepsMut, _sender: &Addr) -> Result<Response, ContractError> {
        unimplemented!()
    }

    pub fn withdraw(_deps: DepsMut, _sender: &Addr) -> Result<Response, ContractError> {
        unimplemented!()
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