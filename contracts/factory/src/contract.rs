use cosmwasm_std::{
    ensure, entry_point, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response,
    StdError, StdResult,
};

use crate::{
    error::ContractError,
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg},
    state::{CONFIG, MARKETS},
};

use common::factory::Config;
use common::market::InstantiateMsg as MarketInstantiateMsg;

const CONTRACT_NAME: &str = "crates.io/cw-otc-factory";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// ID of the reply call expected from the market creation.
const INSTANTIATE_MARKET_REPLY_ID: u64 = 1;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let owner = deps.api.addr_validate(&msg.owner)?;
    let fee_collector = msg
        .fee_collector
        .as_ref()
        .map(|addr| deps.api.addr_validate(addr))
        .transpose()?;

    CONFIG.save(
        deps.storage,
        &Config {
            owner,
            market_code_id: msg.market_code_id,
            fee_collector,
        },
    )?;

    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    use ExecuteMsg::*;
    match msg {
        UpdateConfig {
            new_owner,
            new_fee_collector,
        } => execute::update_config(deps, env, &info.sender, new_owner, new_fee_collector),
        CreateMarket {
            first_coin,
            second_coin,
            fee,
        } => execute::create_market(deps, &info.sender, first_coin, second_coin, fee),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    use QueryMsg::*;
    match msg {
        Config {} => to_json_binary(&query::get_config(deps)?),
        Markets {} => to_json_binary(&query::get_markets(deps)?),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> Result<Response, ContractError> {
    let msg_id = msg.id;
    let res = cw_utils::parse_reply_instantiate_data(msg).map_err(|_| {
        StdError::parse_err("MsgInstantiateContractResponse", "failed to parse data")
    })?;
    match msg_id {
        INSTANTIATE_MARKET_REPLY_ID => reply::handle_instantiate_reply(deps, res),
        _ => Err(ContractError::UnknownReply {}),
    }
}

pub mod reply {
    use cw_utils::MsgInstantiateContractResponse;

    use crate::state::TMP_MARKET_KEY;

    use super::*;

    pub fn handle_instantiate_reply(
        deps: DepsMut,
        res: MsgInstantiateContractResponse,
    ) -> Result<Response, ContractError> {
        let market_key = TMP_MARKET_KEY.load(deps.storage)?;
        MARKETS.save(deps.storage, market_key, &res.contract_address)?;
        Ok(Response::new())
    }
}

pub mod execute {
    use cosmwasm_std::{Addr, Attribute, Decimal, ReplyOn, SubMsg, WasmMsg};

    use crate::state::TMP_MARKET_KEY;

    use super::*;

    pub fn update_config(
        deps: DepsMut,
        _env: Env,
        sender: &Addr,
        new_owner: Option<String>,
        new_fee_collector: Option<String>,
    ) -> Result<Response, ContractError> {
        let mut config = CONFIG.load(deps.storage)?;
        ensure!(config.owner == sender, ContractError::Unauthorized);

        let mut attributes = vec![];

        if let Some(new_owner_addr) = new_owner {
            config.owner = deps.api.addr_validate(&new_owner_addr)?;
            attributes.push(Attribute::new("new_owner", config.owner.clone()));
        }

        if let Some(new_fee_collector_addr) = new_fee_collector {
            let new_address = deps.api.addr_validate(&new_fee_collector_addr)?;
            config.fee_collector = Some(new_address.clone());
            attributes.push(Attribute::new("new_fee_collector", new_address));
        }

        CONFIG.save(deps.storage, &config)?;
        Ok(Response::new()
            .add_attribute("action", "update_config")
            .add_attributes(attributes))
    }

    pub fn create_market(
        deps: DepsMut,
        sender: &Addr,
        first_coin: String,
        second_coin: String,
        fee: Decimal,
    ) -> Result<Response, ContractError> {
        let config = CONFIG.load(deps.storage)?;
        ensure!(config.owner == sender, ContractError::Unauthorized);

        let ordered_coins = order_strings(first_coin.clone(), second_coin.clone());

        if MARKETS
            .may_load(deps.storage, ordered_coins.clone())?
            .is_some()
        {
            return Err(ContractError::MarketAlreadyExists {});
        }

        let sub_msg: Vec<SubMsg> = vec![SubMsg {
            id: INSTANTIATE_MARKET_REPLY_ID,
            msg: WasmMsg::Instantiate {
                admin: Some(config.owner.to_string()),
                code_id: config.market_code_id,
                msg: to_json_binary(&MarketInstantiateMsg {
                    first_coin: first_coin.clone(),
                    second_coin: second_coin.clone(),
                    fee,
                })?,
                funds: vec![],
                label: "Astroport pair".to_string(),
            }
            .into(),
            gas_limit: None,
            reply_on: ReplyOn::Success,
        }];

        TMP_MARKET_KEY.save(deps.storage, &ordered_coins)?;

        Ok(Response::new().add_submessages(sub_msg))
    }

    // Helper function used to order two coin denoms.
    pub fn order_strings(string_one: String, string_two: String) -> (String, String) {
        if string_one < string_two {
            (string_one, string_two)
        } else {
            (string_two, string_one)
        }
    }
}

pub mod query {
    use cosmwasm_std::Order;

    use crate::msg::MarketsResponse;

    use super::*;

    pub fn get_config(deps: Deps) -> StdResult<Config> {
        CONFIG.load(deps.storage)
    }

    pub fn get_markets(deps: Deps) -> StdResult<MarketsResponse> {
        let all_markets = MARKETS
            .range(deps.storage, None, None, Order::Ascending)
            .collect::<StdResult<Vec<((String, String), String)>>>()?;
        Ok(MarketsResponse {
            markets: all_markets,
        })
    }
}

// -------------------------------------------------------------------------------------------------
// Unit tests
// -------------------------------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use cosmwasm_std::{
        testing::{mock_dependencies, mock_env, mock_info},
        Addr, StdError,
    };

    use crate::msg::InstantiateMsg;
    use execute;

    use super::*;

    const OWNER: &str = "0xstepit000";

    #[test]
    fn instatiate_with_fee_collector_works() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("stepit", &[]);

        instantiate(
            deps.as_mut(),
            env,
            info,
            InstantiateMsg {
                owner: OWNER.to_string(),
                market_code_id: 0,
                fee_collector: Some(OWNER.to_string()),
            },
        )
        .unwrap();

        let owner = CONFIG.load(deps.as_ref().storage).unwrap().owner;
        let fee_collector = CONFIG.load(deps.as_ref().storage).unwrap().fee_collector;

        let owner_addr = Addr::unchecked(OWNER);
        assert_eq!(owner_addr, owner, "expect proper owner to be set");
        assert_eq!(
            Some(owner_addr),
            fee_collector,
            "expect proper fee_collector to be set"
        );
    }

    #[test]
    fn instatiate_without_fee_collector_works() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("stepit", &[]);

        instantiate(
            deps.as_mut(),
            env,
            info,
            InstantiateMsg {
                owner: OWNER.to_string(),
                market_code_id: 0,
                fee_collector: None,
            },
        )
        .unwrap();

        let owner = CONFIG.load(deps.as_ref().storage).unwrap().owner;
        let fee_collector = CONFIG.load(deps.as_ref().storage).unwrap().fee_collector;

        let owner_addr = Addr::unchecked(OWNER);
        assert_eq!(owner_addr, owner, "expect proper owner to be set");
        assert_eq!(None, fee_collector, "expect fee_collector to be None");
    }

    #[test]
    fn update_config_works() {
        let mut deps = mock_dependencies();
        let env = mock_env();

        let initial_owner = Addr::unchecked(OWNER);
        let initial_fee_collector = None;
        let config = Config {
            owner: initial_owner.clone(),
            market_code_id: 0,
            fee_collector: initial_fee_collector,
        };

        CONFIG.save(&mut deps.storage, &config).unwrap();

        // Change fee_collector
        execute::update_config(
            deps.as_mut(),
            env.clone(),
            &Addr::unchecked(OWNER),
            Some(OWNER.to_owned()),
            Some(OWNER.to_owned()),
        )
        .unwrap();

        let owner = CONFIG.load(deps.as_ref().storage).unwrap().owner;
        let fee_collector = CONFIG.load(deps.as_ref().storage).unwrap().fee_collector;

        assert_eq!(initial_owner, owner, "expect same owner");
        assert_eq!(
            Some(initial_owner.clone()),
            fee_collector,
            "expect fee_collector to be changed"
        );

        // Change owner
        execute::update_config(
            deps.as_mut(),
            env,
            &Addr::unchecked(OWNER),
            Some("spiderman".to_owned()),
            Some(OWNER.to_owned()),
        )
        .unwrap();

        let owner = CONFIG.load(deps.as_ref().storage).unwrap().owner;
        let fee_collector = CONFIG.load(deps.as_ref().storage).unwrap().fee_collector;

        assert_eq!(
            &Addr::unchecked("spiderman"),
            owner,
            "expect owner to be changed"
        );
        assert_eq!(
            Some(initial_owner),
            fee_collector,
            "expect same fee_collector"
        );
    }

    #[test]
    fn update_config_error_handling() {
        let mut deps = mock_dependencies();
        let env = mock_env();

        let owner = Addr::unchecked(OWNER);
        let fee_collector = None;
        let config = Config {
            owner: owner.clone(),
            market_code_id: 0,
            fee_collector,
        };

        CONFIG.save(&mut deps.storage, &config).unwrap();

        // Only owner can change
        let err = execute::update_config(
            deps.as_mut(),
            env.clone(),
            &Addr::unchecked("spiderman"),
            Some("spiderman".to_owned()),
            Some(OWNER.to_owned()),
        )
        .unwrap_err();

        assert_eq!(
            err,
            ContractError::Unauthorized {},
            "expected to fail because not the owner"
        );

        let new_config = CONFIG.load(deps.as_ref().storage).unwrap();

        assert_eq!(config, new_config, "expected unchanged config");

        // Fails when wrong new variables without changing the state
        let err = execute::update_config(
            deps.as_mut(),
            env.clone(),
            &owner,
            Some("Spiderman".to_owned()),
            Some(OWNER.to_owned()),
        )
        .unwrap_err();

        assert_eq!(
            err,
            ContractError::Std(StdError::generic_err(
                "Invalid input: address not normalized",
            )),
            "expect to fail because not valid owner address"
        );

        let new_config = CONFIG.load(deps.as_ref().storage).unwrap();
        assert_eq!(config, new_config, "expected unchanged config");

        let err = execute::update_config(
            deps.as_mut(),
            env.clone(),
            &owner,
            Some(OWNER.to_owned()),
            Some("Spiderman".to_owned()),
        )
        .unwrap_err();

        assert_eq!(
            err,
            ContractError::Std(StdError::generic_err(
                "Invalid input: address not normalized",
            )),
            "expect to fail because not valid fee_collector address"
        );

        let new_config = CONFIG.load(deps.as_ref().storage).unwrap();
        assert_eq!(config, new_config, "expected unchanged config");
    }
}
