use cosmwasm_std::StdError;

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use crate::global::{execute_swap};
use crate::state::{
    read_config, store_config, store_state, Config, State, SwapConfig, SWAP_CONFIG,
};
use crate::user::{execute_claim_rewards, query_accrued_rewards};
use cosmwasm_std::{
    to_binary, Binary, Decimal256, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128,
};

use basset::reward::{
    ConfigResponse, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg
};


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
        ExecuteMsg::SetCustodyContract { custody_contract } => {
            set_custody_contract(deps, info, custody_contract)
        }
    }
}

pub fn set_custody_contract(
    deps: DepsMut,
    info: MessageInfo,
    custody_contract: String,
) -> StdResult<Response> {
    let mut config = read_config(deps.storage)?;

    if info.sender != config.owner {
        return Err(StdError::generic_err("unauthorized"));
    }

    if config.custody_contract.is_some() {
        return Err(StdError::generic_err("unauthorized"));
    }

    config.custody_contract = Some(deps.api.addr_validate(&custody_contract)?);
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

fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config: Config = read_config(deps.storage)?;
    Ok(ConfigResponse {
        hub_contract: config.hub_contract.to_string(),
        reward_denom: config.reward_denom,
    })
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    Ok(Response::default())
}
