use crate::state::{read_config};

use cosmwasm_std::{
    attr, CosmosMsg, DepsMut, Env, MessageInfo, Response, StdError, StdResult,
};

use crate::swap::create_swap_msgs;

/// Swap all native tokens to reward_denom
/// Only hub_contract is allowed to execute
#[allow(clippy::if_same_then_else)]
#[allow(clippy::needless_collect)]
pub fn execute_swap(deps: DepsMut, env: Env, info: MessageInfo) -> StdResult<Response> {
    let config = read_config(deps.storage)?;

    if info.sender != config.hub_contract {
        return Err(StdError::generic_err("unauthorized"));
    }

    let contr_addr = env.contract.address.clone();
    let balance = deps.querier.query_all_balances(contr_addr)?;
    let mut messages: Vec<CosmosMsg> = Vec::new();

    let reward_denom = config.reward_denom;

    let denoms: Vec<String> = balance.iter().map(|item| item.denom.clone()).collect();

    for coin in balance {
        if coin.denom == reward_denom.clone() || !denoms.contains(&coin.denom) {
            continue;
        }

        messages.append(&mut create_swap_msgs(
            deps.as_ref(),
            env.clone(),
            coin,
            reward_denom.to_string(),
        )?);
    }

    let res = Response::new()
        .add_messages(messages)
        .add_attributes(vec![attr("action", "swap")]);

    Ok(res)
}