use basset::external::LSDStateResponseTrait;
use cosmwasm_std::Decimal;
use cosmwasm_std::Deps;
use cosmwasm_std::StdResult;
use cosmwasm_std::{
    to_binary, Binary, CosmosMsg, DepsMut, Env, MessageInfo, Response, Uint128, WasmMsg,
};
use cw20_base::contract::query_balance;
use serde::Deserialize;

use cw20::Cw20ExecuteMsg;
use cw20_base::allowances::{
    execute_burn_from as cw20_burn_from, execute_send_from as cw20_send_from,
    execute_transfer_from as cw20_transfer_from,
};
use cw20_base::contract::{
    execute_burn as cw20_burn, execute_mint as cw20_mint, execute_send as cw20_send,
    execute_transfer as cw20_transfer,
};
use cw20_base::ContractError;

use crate::querier::query_lsd_state;
use crate::state::read_lsd_contract;

pub fn execute_transfer(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    recipient: String,
    amount: Uint128,
) -> Result<Response, ContractError> {
    cw20_transfer(deps, env, info, recipient, amount)
}

fn _before_burn<T: LSDStateResponseTrait + for<'a> Deserialize<'a>>(
    deps: Deps,
    info: MessageInfo,
    amount: Uint128,
) -> StdResult<CosmosMsg> {
    let lsd_contracts = read_lsd_contract(deps.storage)?;

    // When burning some tokens from here, we transfer an equivalent amount of 1 Luna per each burned token to the burner
    let lsd_exchange_rate = query_lsd_state::<T>(deps, &lsd_contracts)?.exchange_rate();
    let lsd_amount = Decimal::from_ratio(amount, 1u128) / lsd_exchange_rate;

    Ok(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: lsd_contracts.token.to_string(),
        msg: to_binary(&Cw20ExecuteMsg::Transfer {
            recipient: info.sender.to_string(),
            amount: lsd_amount * Uint128::one(),
        })?,
        funds: vec![],
    }))
}

pub fn execute_burn<T: LSDStateResponseTrait + for<'a> Deserialize<'a>>(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let transfer_message = _before_burn::<T>(deps.as_ref(), info.clone(), amount)?;

    let res = cw20_burn(deps, env, info, amount)?;

    Ok(res.add_message(transfer_message))
}

pub fn execute_burn_all<T: LSDStateResponseTrait + for<'a> Deserialize<'a>>(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let amount = query_balance(deps.as_ref(), info.sender.to_string())?;

    if amount.balance.is_zero() {
        return Ok(Response::new());
    }
    execute_burn::<T>(deps, env, info, amount.balance)
}

pub fn execute_mint<T: LSDStateResponseTrait + for<'a> Deserialize<'a>>(
    deps: DepsMut,
    env: Env,
    mut info: MessageInfo,
    recipient: String,
    amount: Uint128,
) -> Result<Response, ContractError> {
    // In order to mint, we need to transfer the underlying lsd asset to the contract
    // Any sender can call this function as long as they have the sufficient lsd balance
    let lsd_contracts = read_lsd_contract(deps.storage)?;

    // We query the exchange rate with respect to the LSD at which we can mint some new wrapper token
    let lsd_state: T = query_lsd_state(deps.as_ref(), &lsd_contracts)?;

    // We add 1 to the send_lsd_amount here to make sure we are not undercollateralizing our token at the start
    let send_lsd_amount =
        Decimal::from_ratio(amount, 1u128) / lsd_state.exchange_rate() + Decimal::one();

    let cw20_transfer_message = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: lsd_contracts.token.to_string(),
        msg: to_binary(&Cw20ExecuteMsg::TransferFrom {
            owner: info.sender.to_string(),
            recipient: env.contract.address.to_string(),
            amount: send_lsd_amount * Uint128::one(),
        })?,
        funds: vec![],
    });
    info.sender = env.contract.address.clone();

    let res = cw20_mint(deps, env, info, recipient, amount)?;

    Ok(res.add_message(cw20_transfer_message))
}

pub fn execute_mint_with<T: LSDStateResponseTrait + for<'a> Deserialize<'a>>(
    deps: DepsMut,
    env: Env,
    mut info: MessageInfo,
    recipient: String,
    lsd_amount: Uint128,
) -> Result<Response, ContractError> {
    // In order to mint, we need to transfer the underlying lsd asset to the contract
    // Any sender can call this function as long as they have the sufficient lsd balance
    let lsd_contracts = read_lsd_contract(deps.storage)?;

    // We query the exchange rate with respect to the LSD at which we can mint some new wrapper token
    let lsd_state: T = query_lsd_state(deps.as_ref(), &lsd_contracts)?;

    let mint_amount = Decimal::from_ratio(lsd_amount, 1u128) * lsd_state.exchange_rate();

    let cw20_transfer_message = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: lsd_contracts.token.to_string(),
        msg: to_binary(&Cw20ExecuteMsg::TransferFrom {
            owner: info.sender.to_string(),
            recipient: env.contract.address.to_string(),
            amount: lsd_amount,
        })?,
        funds: vec![],
    });
    info.sender = env.contract.address.clone();

    let res = cw20_mint(deps, env, info, recipient, mint_amount * Uint128::one())?;

    Ok(res.add_message(cw20_transfer_message))
}

pub fn execute_send(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    contract: String,
    amount: Uint128,
    msg: Binary,
) -> Result<Response, ContractError> {
    cw20_send(deps, env, info, contract, amount, msg)
}

pub fn execute_transfer_from(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    owner: String,
    recipient: String,
    amount: Uint128,
) -> Result<Response, ContractError> {
    cw20_transfer_from(deps, env, info, owner, recipient, amount)
}

pub fn execute_burn_from<T: LSDStateResponseTrait + for<'a> Deserialize<'a>>(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    owner: String,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let transfer_message = _before_burn::<T>(deps.as_ref(), info.clone(), amount)?;

    let res = cw20_burn_from(deps, env, info, owner, amount)?;

    Ok(res.add_message(transfer_message))
}

pub fn execute_send_from(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    owner: String,
    contract: String,
    amount: Uint128,
    msg: Binary,
) -> Result<Response, ContractError> {
    cw20_send_from(deps, env, info, owner, contract, amount, msg)
}
