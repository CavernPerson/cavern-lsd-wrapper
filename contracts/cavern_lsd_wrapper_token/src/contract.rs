use basset::reward::MigrateMsg;
use crate::querier::get_current_exchange_rate;
use crate::state::HUB_CONTRACT_KEY;
use crate::state::LsdContracts;
use crate::state::WrapperState;
use crate::state::read_lsd_contract;
use crate::state::read_wrapper_state;
use crate::state::store_hub_contract;
use crate::state::store_lsd_contract;
use crate::state::store_wrapper_state;
use basset::wrapper::AccruedRewards;
use basset::wrapper::ExecuteMsg;
use cosmwasm_std::attr;
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::to_binary;
use cosmwasm_std::CosmosMsg;
use cosmwasm_std::Decimal;
use cosmwasm_std::Uint128;
use cosmwasm_std::WasmMsg;
use cw20::Cw20ExecuteMsg;

use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult};

use cw20_base::allowances::{execute_decrease_allowance, execute_increase_allowance};
use cw20_base::contract::query as cw20_query;
use cw20_base::contract::{
    execute_update_marketing, execute_update_minter, execute_upload_logo, instantiate as cw20_init,
};
use cw20_base::msg::{InstantiateMsg, QueryMsg};

use crate::handler::*;
use crate::msg::TokenInitMsg;
use cw20::MinterResponse;
use cw20_base::ContractError;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: TokenInitMsg,
) -> StdResult<Response> {
    store_lsd_contract(deps.storage, &LsdContracts{
        hub: deps.api.addr_validate(&msg.lsd_hub_contract)?,
        token: deps.api.addr_validate(&msg.lsd_token_contract)?
    })?;

    store_wrapper_state(deps.storage, &WrapperState{
        prev_backing_luna: Decimal::zero(),
        prev_wlsd_supply: Uint128::zero(),
        prev_lsd_balance: Uint128::zero(),
        prev_lsd_exchange_rate: Decimal::one()
    })?;

    store_hub_contract(deps.storage, &deps.api.addr_validate(&msg.hub_contract)?)?;

    cw20_init(
        deps,
        env.clone(),
        info,
        InstantiateMsg {
            name: msg.name,
            symbol: msg.symbol,
            decimals: msg.decimals,
            initial_balances: msg.initial_balances,
            mint: Some(MinterResponse {
                /// Only this contract can mint new tokens in exchange of the underlying lsd
                minter: env.contract.address.to_string(),
                cap: None,
            }),
            marketing: None,
        },
    )
    .map_err(|_| StdError::generic_err("CW20 Token init error"))?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Transfer { recipient, amount } => {
            execute_transfer(deps, env, info, recipient, amount)
        }
        ExecuteMsg::Burn { amount } => execute_burn(deps, env, info, amount),
        ExecuteMsg::Send {
            contract,
            amount,
            msg,
        } => execute_send(deps, env, info, contract, amount, msg),
        ExecuteMsg::Mint { recipient, amount } => execute_mint(deps, env, info, recipient, amount),
        ExecuteMsg::MintWith { recipient, lsd_amount } => execute_mint_with(deps, env, info, recipient, lsd_amount),
        ExecuteMsg::IncreaseAllowance {
            spender,
            amount,
            expires,
        } => execute_increase_allowance(deps, env, info, spender, amount, expires),
        ExecuteMsg::DecreaseAllowance {
            spender,
            amount,
            expires,
        } => execute_decrease_allowance(deps, env, info, spender, amount, expires),
        ExecuteMsg::TransferFrom {
            owner,
            recipient,
            amount,
        } => execute_transfer_from(deps, env, info, owner, recipient, amount),
        ExecuteMsg::BurnFrom { owner, amount } => execute_burn_from(deps, env, info, owner, amount),
        ExecuteMsg::SendFrom {
            owner,
            contract,
            amount,
            msg,
        } => execute_send_from(deps, env, info, owner, contract, amount, msg),
        ExecuteMsg::UpdateMinter { new_minter } => {
            execute_update_minter(deps, env, info, new_minter)
        }
        ExecuteMsg::UpdateMarketing {
            project,
            description,
            marketing,
        } => execute_update_marketing(deps, env, info, project, description, marketing),
        ExecuteMsg::UploadLogo(logo) => execute_upload_logo(deps, env, info, logo),
        ExecuteMsg::Decompound { recipient } => execute_decompound(deps, env, info, recipient),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    cw20_query(deps, _env, msg)
}

fn compute_accrued_rewards(deps: Deps, env: Env) -> Result<AccruedRewards, ContractError> {

    // In this function, we have to make sure the token has a 1 exchange rate to Luna.
    let mut state = read_wrapper_state(deps.storage)?;
    let current_exchange_rate = get_current_exchange_rate(deps, env, &mut state)?;

    // If the current exchange rate is lower than 1, we have just had a slashing event TODO
    if current_exchange_rate < Decimal::one() {
        // An error, ok, what ?
        // We execute hub slashing ?
        return Err(ContractError::Std(StdError::generic_err(
            "Need to execute slashing",
        )));
    }
    // Else , we have some available rewards to decompound
    let mut luna_rewards = state.prev_backing_luna * Uint128::one() - state.prev_wlsd_supply;

    let mut rewards_to_decompound = (Decimal::from_ratio(state.prev_lsd_balance, 1u128)
        - (Decimal::from_ratio(state.prev_wlsd_supply, 1u128) / state.prev_lsd_exchange_rate))
            * Uint128::one();

    // We substract 1 to the rewards to decompound so that we don't screw up the underlying value of the wrapper token
    // The underlying value should be if possible always above 1 luna per wrapper token (slashing events should happen as little often as possible)
    if rewards_to_decompound > Uint128::zero(){
        rewards_to_decompound -= Uint128::one();
        luna_rewards -= state.prev_lsd_exchange_rate * Uint128::one();
    }

    Ok(AccruedRewards {
        luna_rewards: luna_rewards * Uint128::one(),
        lsd_rewards: rewards_to_decompound * Uint128::one(),
    })
}

/*
let luna_rewards = (wlsd_exchange_rate - 1) * wlsd_supply = current_luna_amount - wanted_luna_amount
let rewards_to_decompound = luna_rewards / lsd_exchange_rate = current_lsd_balance - wanted_lsd_balance


lr = (cr - 1) * wlsd
rtd = (cr - 1) * wlsd / exchange_rate

cr = b*exchange_rate / wlsd

lr = (b * exchange_rate - wlsd);
rtd = (b - wlsd/exchange_rate);

*/

pub fn execute_decompound(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    recipient: Option<String>,
) -> Result<Response, ContractError> {

    let hub_contract = HUB_CONTRACT_KEY.load(deps.storage)?;
    if info.sender != hub_contract{
        return Err(ContractError::Unauthorized {  })
    }

    let recipient = recipient
        .map(|x| deps.api.addr_validate(&x))
        .transpose()?
        .unwrap_or(info.sender);


    let lsd_contract = read_lsd_contract(deps.storage)?;
    let slashing_error = ContractError::Std(StdError::generic_err("Need to execute slashing"));
    let (out_messages, accrued_rewards) = match compute_accrued_rewards(deps.as_ref(), env) {
        Err(err) => {
            if err == slashing_error {
                Ok((vec![], AccruedRewards::default()))
            } else {
                Err(err)
            }
        }
        Ok(rewards) => {
            let decompound_messages = if !rewards.lsd_rewards.is_zero() {
                vec![CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: lsd_contract.token.to_string(),
                    msg: to_binary(&Cw20ExecuteMsg::Transfer {
                        recipient: recipient.to_string(),
                        amount: rewards.lsd_rewards,
                    })?,
                    funds: vec![],
                })]
            } else {
                vec![]
            };
            Ok((decompound_messages, rewards))
        }
    }?;

    let res = Response::new()
        .add_attributes(vec![
            attr("action", "claim_reward"),
            attr(
                "total_luna_rewards",
                accrued_rewards.luna_rewards.to_string(),
            ),
        ])
        .add_messages(out_messages);

    Ok(res)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    Err(StdError::GenericErr { msg: "No Migrate Implemented".to_string() })
    //Ok(Response::default())
}