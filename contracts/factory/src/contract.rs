use cosmwasm_std::{
    entry_point, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};

use crate::{
    erorr::ContractError,
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg},
    state::CONFIG,
};

use common::factory::Config;

pub const CONTRACT_NAME: &str = "crates.io/cw-otc";
pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    let fee_collector = msg
        .fee_collector
        .as_ref()
        .map(|addr| deps.api.addr_validate(addr))
        .transpose()?;

    CONFIG.save(
        deps.storage,
        &Config {
            owner: deps.api.addr_validate(&msg.owner)?,
            fee_collector,
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
        UpdateConfig {} => execute::update_config(deps, env, &info.sender),
        CreateMarket {} => execute::create_market(deps, &info.sender),
    }
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    use QueryMsg::*;
    match msg {
        QueryConfig {} => to_json_binary(&query::get_config(deps)?),
        QueryMarkets {} => to_json_binary(&query::get_markets(deps)?),
    }
}

pub mod execute {
    use cosmwasm_std::Addr;

    use super::*;

    pub fn update_config(
        _deps: DepsMut,
        _env: Env,
        _sender: &Addr,
    ) -> Result<Response, ContractError> {
        unimplemented!()
    }

    pub fn create_market(_deps: DepsMut, _sender: &Addr) -> Result<Response, ContractError> {
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

// -------------------------------------------------------------------------------------------------
// Unit tests
// -------------------------------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use cosmwasm_std::{
        testing::{mock_dependencies, mock_env, mock_info},
        Addr,
    };

    use crate::msg::InstantiateMsg;

    use super::*;

    const OWNER: &str = "0xstepit000";

    #[test]
    fn instatiate_with_fee_collector() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("stepit", &[]);

        instantiate(
            deps.as_mut(),
            env,
            info,
            InstantiateMsg {
                owner: OWNER.to_string(),
                fee_collector: Some(OWNER.to_string()),
            },
        )
        .unwrap();

        let owner = CONFIG.load(deps.as_ref().storage).unwrap().owner;
        let fee_collector = CONFIG.load(deps.as_ref().storage).unwrap().fee_collector;

        let owner_addr = Addr::unchecked(OWNER);
        assert_eq!(owner_addr, owner);
        assert_eq!(Some(owner_addr), fee_collector);
    }

    #[test]
    fn instatiate_without_fee_collector() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("stepit", &[]);

        instantiate(
            deps.as_mut(),
            env,
            info,
            InstantiateMsg {
                owner: OWNER.to_string(),
                fee_collector: None,
            },
        )
        .unwrap();

        let owner = CONFIG.load(deps.as_ref().storage).unwrap().owner;
        let fee_collector = CONFIG.load(deps.as_ref().storage).unwrap().fee_collector;

        let owner_addr = Addr::unchecked(OWNER);
        assert_eq!(owner_addr, owner);
        assert_eq!(None, fee_collector);
    }
}
