use cosmwasm_std::Addr;
use cosmwasm_std::StdError;

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use crate::global::execute_swap;
use crate::state::{
    read_config, store_config, store_state, Config, State, SwapConfig, SWAP_CONFIG,
};
use crate::user::{execute_claim_rewards, query_accrued_rewards};
use cosmwasm_std::{
    to_binary, Binary, Decimal256, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128,
};

use basset::reward::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    let conf = Config {
        owner: info.sender,
        hub_contract: deps.api.addr_validate(&msg.hub_contract)?,
        custody_contract: None,
        reward_denom: msg.reward_denom,

        known_cw20_tokens: msg
            .known_tokens
            .iter()
            .map(|addr| deps.api.addr_validate(addr))
            .collect::<StdResult<Vec<Addr>>>()?,
    };

    store_config(deps.storage, &conf)?;
    store_state(
        deps.storage,
        &State {
            global_index: Decimal256::zero(),
            total_balance: Uint128::zero(),
            prev_reward_balance: Uint128::zero(),
        },
    )?;

    SWAP_CONFIG.save(
        deps.storage,
        &SwapConfig {
            astroport_addr: deps.api.addr_validate(&msg.astroport_addr)?,
            phoenix_addr: deps.api.addr_validate(&msg.phoenix_addr)?,
            terraswap_addr: deps.api.addr_validate(&msg.terraswap_addr)?,
        },
    )?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    match msg {
        ExecuteMsg::ClaimRewards { recipient } => execute_claim_rewards(deps, env, info, recipient),
        ExecuteMsg::SwapToRewardDenom {} => execute_swap(deps, env, info),
        ExecuteMsg::UpdateConfig {
            custody_contract,
            known_tokens,
            reward_denom,
            owner,
        } => update_config(deps, info, custody_contract, known_tokens, reward_denom, owner),
    }
}

pub fn update_config(
    deps: DepsMut,
    info: MessageInfo,
    custody_contract: Option<String>,
    known_tokens: Option<Vec<String>>,
    reward_denom: Option<String>,
    owner: Option<String>,
) -> StdResult<Response> {
    let mut config = read_config(deps.storage)?;

    if info.sender != config.owner {
        return Err(StdError::generic_err("unauthorized"));
    }

    if let Some(custody_contract) = custody_contract {
        config.custody_contract = Some(deps.api.addr_validate(&custody_contract)?);
    }

    if let Some(known_tokens) = known_tokens {
        config.known_cw20_tokens = known_tokens
            .iter()
            .map(|token| deps.api.addr_validate(token))
            .collect::<StdResult<Vec<Addr>>>()?
    }

    if let Some(reward_denom) = reward_denom {
        config.reward_denom = reward_denom
    }

    if let Some(owner) = owner {
        config.owner = deps.api.addr_validate(&owner)?;
    }

    store_config(deps.storage, &config)?;

    Ok(Response::new().add_attribute("action", "set_custody_contract"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::AccruedRewards { address } => {
            to_binary(&query_accrued_rewards(deps, env, address)?)
        }
    }
}

fn query_config(deps: Deps) -> StdResult<Config> {
    let config: Config = read_config(deps.storage)?;
    Ok(config)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    Ok(Response::default())
}
