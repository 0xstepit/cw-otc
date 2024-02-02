use cosmwasm_std::{
    coin, entry_point, to_json_binary, Binary, Decimal, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};

use crate::{
    error::ContractError,
    msg::{ExecuteMsg, QueryMsg},
    state::CONFIG,
};

use astroport::asset::validate_native_denom;

use common::market::{Config, InstantiateMsg};

const CONTRACT_NAME: &str = "crates.io/cw-otc-market";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

// Maximum allowed fee is 5%.
pub const MAX_FEE: Decimal = Decimal::percent(5);

#[cfg_attr(not(feature = "library"), entry_point)]
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
            second_coin: msg.second_coin,
        });
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
            fee: msg.fee,
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
        CreateDeal {
            coin_out,
            counterparty,
            timeout,
        } => execute::create_deal(deps, env, info, coin_out, counterparty, timeout),
        AcceptDeal { creator, deal_id } => execute::accept_deal(deps, info, env, creator, deal_id),
        Withdraw { creator, deal_id } => execute::withdraw(deps, info, env, creator, deal_id),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    use QueryMsg::*;
    match msg {
        Config {} => to_json_binary(&query::get_config(deps)?),
        DealsByCreator { creator } => {
            to_json_binary(&query::get_deals_by_creator(deps, env, creator)?)
        }
        AllDeals {} => to_json_binary(&query::get_all_deals(deps, env)?),
    }
}

pub mod execute {
    use std::ops::Add;

    use common::market::{Deal, DealStatus, WithdrawStatus};
    use cosmwasm_std::{Addr, BankMsg, Coin, CosmosMsg, Uint128};

    use crate::state::{next_id, DEALS};

    use super::*;

    /// Crerate a new deal. The deal can be open of specific for one counterparty.
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
            timeout: env.block.height.add(timeout),
            status: DealStatus::NotMatched,
        };

        let deal_id = next_id(deps.storage)?;
        DEALS.save(deps.storage, (&info.sender, deal_id), &deal)?;

        Ok(Response::new()
            .add_attribute("action", "create_dial")
            .add_attribute("deal_id", deal_id.to_string())
            .add_attribute("creator", info.sender))
    }

    // To allow an address to accept a deal, we have to check the following conditions:
    // 1. deal has not ben previously matched and is not expired.
    // 2. sent funds are the same requested by the creator of the deal.
    // 3. if the deal is associated with an address, sender must be that address
    pub fn accept_deal(
        deps: DepsMut,
        info: MessageInfo,
        env: Env,
        creator: String,
        deal_id: u64,
    ) -> Result<Response, ContractError> {
        check_only_one_coin(&info.funds)?;

        let creator = Addr::unchecked(creator);
        if info.sender == creator {
            return Err(ContractError::SenderIsCreator {});
        }
        let mut deal = DEALS.load(deps.storage, (&creator, deal_id))?;

        // Return error if the deal is expired or already matched.
        if deal.status != DealStatus::NotMatched || deal.timeout < env.block.height {
            return Err(ContractError::DealNotAvailable {});
        }

        // Check if sent coins are the same of the selected deal.
        if deal.coin_out != info.funds[0] {
            return Err(ContractError::WrongCoin {
                denom: deal.coin_out.denom.clone(),
                amount: deal.coin_out.amount,
            });
        }

        // Check if the deal is reserved and sender is not the lucky one.
        if deal.counterparty.is_some() && Some(info.sender.clone()) != deal.counterparty {
            return Err(ContractError::Unauthorized {});
        }

        // We set the counterparty as sender and deal matched.
        // When counterparty is set and the deal matched, counterparty address
        // and the creator are allowed to withdraw.
        deal.counterparty = Some(info.sender);
        deal.status = DealStatus::matched_no_withdraw();

        DEALS.save(deps.storage, (&creator, deal_id), &deal)?;

        Ok(Response::new()
            .add_attribute("action", "accept_deal")
            .add_attribute("deal_counterparty", deal.counterparty.unwrap()))
    }

    /// Allows to withdraw tokens asscoiated with a deal. If no one accepted the deal, the creator can
    /// close it and withdraw coins without deducted fee. If a deal is close, fee are deducted from
    /// both the parties.
    pub fn withdraw(
        deps: DepsMut,
        info: MessageInfo,
        _env: Env,
        creator: String,
        deal_id: u64,
    ) -> Result<Response, ContractError> {
        let config = CONFIG.load(deps.storage)?;

        let creator = Addr::unchecked(creator);

        let mut deal = DEALS.load(deps.storage, (&creator, deal_id))?;

        let is_creator = creator == info.sender;
        let is_counterparty = Some(info.sender.clone()) == deal.counterparty;

        if !is_counterparty && !is_creator {
            return Err(ContractError::Unauthorized);
        }

        // Separate the withdraw in two cases for readability

        // First consider the case of unmatched deal
        let msgs: Vec<CosmosMsg> = match deal.status {
            DealStatus::NotMatched if is_creator => {
                deal.status = DealStatus::Matched(WithdrawStatus::Completed);
                create_withdraw_msg_not_matched(info.sender, deal.coin_in.clone())
            }
            DealStatus::Matched(WithdrawStatus::NoWithdraw) => {
                let withdraw_coin = if is_creator {
                    deal.status = DealStatus::Matched(WithdrawStatus::CreatorWithdrawed);
                    deal.coin_out.clone()
                } else {
                    deal.status = DealStatus::Matched(WithdrawStatus::CounterpartyWithdrawed);
                    deal.coin_in.clone()
                };
                create_withdraw_msg_matched(info.sender, withdraw_coin, config)
            }
            DealStatus::Matched(WithdrawStatus::CreatorWithdrawed) if !is_creator => {
                deal.status = DealStatus::Matched(WithdrawStatus::Completed);
                create_withdraw_msg_matched(info.sender, deal.coin_in.clone(), config)
            }
            DealStatus::Matched(WithdrawStatus::CounterpartyWithdrawed) if is_creator => {
                deal.status = DealStatus::Matched(WithdrawStatus::Completed);
                create_withdraw_msg_matched(info.sender, deal.coin_out.clone(), config)
            }
            _ => vec![],
        };

        if msgs.is_empty() {
            return Err(ContractError::Unauthorized {});
        }

        if deal.status == DealStatus::matched_and_completed() {
            DEALS.remove(deps.storage, (&creator, deal_id))
        } else {
            DEALS.save(deps.storage, (&creator, deal_id), &deal)?;
        }

        Ok(Response::new()
            .add_attribute("action", "withdraw")
            .add_messages(msgs))
    }

    /// Check that only one coin has been sent to the contract.
    pub fn check_only_one_coin(funds: &Vec<Coin>) -> Result<(), ContractError> {
        if funds.len() != 1 {
            return Err(ContractError::FundsError {});
        }
        Ok(())
    }

    /// Check that the denom is an allowed coin for the market.
    pub fn check_allowed_coin(denom: &str, config: &Config) -> Result<(), ContractError> {
        if denom != config.first_coin && denom != config.second_coin {
            return Err(ContractError::CoinNotAllowed {});
        }
        Ok(())
    }

    /// Create a bank transfer message to refund the entire amount.
    pub fn create_withdraw_msg_not_matched(receiver: Addr, coin: Coin) -> Vec<CosmosMsg> {
        let msg: CosmosMsg = BankMsg::Send {
            to_address: receiver.to_string(),
            amount: vec![coin],
        }
        .into();
        vec![msg]
    }

    /// Create a bank transfer message to the receiver and a bank transfer message for th fee if any.
    pub fn create_withdraw_msg_matched(
        receiver: Addr,
        withdraw_coin: Coin,
        config: Config,
    ) -> Vec<CosmosMsg> {
        let mut msgs = vec![];

        let fee_amount = withdraw_coin.amount * config.fee;
        let receiver_amount = withdraw_coin.amount - fee_amount;
        msgs.push(
            BankMsg::Send {
                to_address: receiver.to_string(),
                amount: vec![coin(receiver_amount.u128(), withdraw_coin.denom.clone())],
            }
            .into(),
        );

        if fee_amount != Uint128::zero() {
            msgs.push(
                BankMsg::Send {
                    to_address: config.owner.to_string(),
                    amount: vec![coin(fee_amount.u128(), withdraw_coin.denom)],
                }
                .into(),
            );
        }
        msgs
    }
}

pub mod query {
    use common::market::Deal;
    use cosmwasm_std::{Addr, Order};

    use crate::{
        msg::{AllDealsResponse, DealsByCreatorResponse},
        state::DEALS,
    };

    use super::*;

    pub fn get_config(deps: Deps) -> StdResult<Config> {
        CONFIG.load(deps.storage)
    }

    /// Returns the active deals associated with a creator.
    pub fn get_deals_by_creator(
        deps: Deps,
        env: Env,
        creator: String,
    ) -> StdResult<DealsByCreatorResponse> {
        let creator = Addr::unchecked(creator);
        let deals = DEALS
            .prefix(&creator)
            .range(deps.storage, None, None, Order::Ascending)
            .filter_map(|item| {
                item.ok().and_then(|(id, deal)| {
                    if deal.timeout >= env.block.height {
                        Some(Ok((id, deal)))
                    } else {
                        None
                    }
                })
            })
            .collect::<StdResult<Vec<(u64, Deal)>>>()?;

        Ok(DealsByCreatorResponse { deals })
    }

    /// Returns all active deals.
    pub fn get_all_deals(deps: Deps, env: Env) -> StdResult<AllDealsResponse> {
        let deals = DEALS
            .range(deps.storage, None, None, Order::Ascending)
            .filter_map(|item| {
                item.ok().and_then(|(id, deal)| {
                    if deal.timeout >= env.block.height {
                        Some(Ok((id, deal)))
                    } else {
                        None
                    }
                })
            })
            .collect::<StdResult<Vec<((Addr, u64), Deal)>>>()?;
        Ok(AllDealsResponse { deals })
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

    use common::market::InstantiateMsg;

    use super::*;

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
                fee: Decimal::percent(1),
            },
        )
        .unwrap();

        let config = CONFIG.load(deps.as_ref().storage).unwrap();
        let expected_config = Config {
            owner: Addr::unchecked("stepit"),
            first_coin: "astro".to_owned(),
            second_coin: "usdc".to_owned(),
            fee: Decimal::percent(1),
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
                first_coin: "ibc/EBD5A24C554198EBAF44979C5B4D2C2D312E6EBAB71962C92F735499C7575839"
                    .to_owned(),
                second_coin: "usdc".to_owned(),
                fee: Decimal::percent(1),
            },
        )
        .unwrap();

        let config = CONFIG.load(deps.as_ref().storage).unwrap();
        let expected_config = Config {
            owner: Addr::unchecked("stepit"),
            first_coin: "ibc/EBD5A24C554198EBAF44979C5B4D2C2D312E6EBAB71962C92F735499C7575839"
                .to_owned(),
            second_coin: "usdc".to_owned(),
            fee: Decimal::percent(1),
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
                fee: Decimal::percent(1),
            },
        )
        .unwrap();

        let config = CONFIG.load(deps.as_ref().storage).unwrap();
        let expected_config = Config {
            owner: Addr::unchecked("stepit"),
            first_coin: "factory/wasm1jdppe6fnj2q7hjsepty5crxtrryzhuqsjrj95y/astro".to_owned(),
            second_coin: "usdc".to_owned(),
            fee: Decimal::percent(1),
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
                fee: Decimal::percent(6),
            },
        )
        .unwrap_err();

        assert_eq!(
            err,
            ContractError::OverFeeMax {},
            "expected different error for over max fee"
        );

        let err = instantiate(
            deps.as_mut(),
            env,
            info,
            InstantiateMsg {
                first_coin: "astro".to_owned(),
                second_coin: "astro".to_owned(),
                fee: Decimal::percent(1),
            },
        )
        .unwrap_err();

        assert_eq!(
            err,
            ContractError::CoinError {
                first_coin: "astro".to_owned(),
                second_coin: "astro".to_owned()
            },
            "expected different error for same coin"
        );
    }
}
