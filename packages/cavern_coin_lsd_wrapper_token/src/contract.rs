use cosmwasm_std::Coin;

use basset::external::LSDStateResponseTrait;
use cosmwasm_std::BankMsg;
use serde::Deserialize;
use basset::reward::MigrateMsg;
use basset::wrapper::TokenInfoResponse;
use cw20_base::contract::query_token_info;
use crate::querier::get_current_exchange_rate;
use crate::state::HUB_CONTRACT_KEY;
use crate::state::LsdContracts;

use crate::state::WrapperState;
use crate::state::read_lsd_contract;


use crate::state::store_hub_contract;
use crate::state::store_lsd_contract;

use basset::wrapper::AccruedRewards;
use basset::wrapper::ExecuteMsg;
use cosmwasm_std::attr;
use cosmwasm_std::entry_point;
use cosmwasm_std::to_binary;
use cosmwasm_std::CosmosMsg;
use cosmwasm_std::Decimal;
use cosmwasm_std::Uint128;



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

pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: TokenInitMsg,
) -> StdResult<Response> {
    store_lsd_contract(deps.storage, &LsdContracts{
        hub: deps.api.addr_validate(&msg.lsd_hub_contract)?,
        denom: msg.lsd_denom,
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

pub fn execute<T: LSDStateResponseTrait + for<'a> Deserialize<'a>>(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Transfer { recipient, amount } => {
            execute_transfer(deps, env, info, recipient, amount)
        }
        ExecuteMsg::Burn { amount } => execute_burn::<T>(deps, env, info, amount),
        ExecuteMsg::BurnAll { } => execute_burn_all::<T>(deps, env, info),
        ExecuteMsg::Send {
            contract,
            amount,
            msg,
        } => execute_send(deps, env, info, contract, amount, msg),
        ExecuteMsg::Mint { recipient, amount } => execute_mint::<T>(deps, env, info, recipient, amount),
        ExecuteMsg::MintWith { recipient, lsd_amount } => execute_mint_with::<T>(deps, env, info, recipient, lsd_amount),
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
        ExecuteMsg::BurnFrom { owner, amount } => execute_burn_from::<T>(deps, env, info, owner, amount),
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
        ExecuteMsg::Decompound { recipient } => execute_decompound::<T>(deps, env, info, recipient),
    }
}

pub fn query<T: LSDStateResponseTrait + for<'a> Deserialize<'a>>(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {

    // If the token info are queried, we also add the current wLSD exchange rate to it
    match msg{
        QueryMsg::TokenInfo {  } => {
            let token_info = query_token_info(deps)?;
            let mut state = WrapperState::default();
            to_binary(&TokenInfoResponse{
                name: token_info.name,
                symbol: token_info.symbol,
                decimals: token_info.decimals,
                total_supply: token_info.total_supply,
                exchange_rate: get_current_exchange_rate::<T>(deps, env, &mut state).map_err(|err| StdError::generic_err(err.to_string()))?
            })
        },
        _ => cw20_query(deps, env, msg)
    }
}

fn compute_accrued_rewards<T: LSDStateResponseTrait + for<'a> Deserialize<'a>>(deps: Deps, env: Env) -> Result<AccruedRewards, ContractError> {

    // In this function, we have to make sure the token has a 1 exchange rate to Luna.
    let mut state = WrapperState::default();
    let current_exchange_rate = get_current_exchange_rate::<T>(deps, env, &mut state)?;

    // If the current exchange rate is lower than the previous one, we have just had a slashing event or something else
    // We can't decompound and we can't recompound 
    if current_exchange_rate < Decimal::one() {
        // There is no accrued rewards to decompound.
        return Err(ContractError::Std(StdError::generic_err(
            "No rewards to decompound",
        )));
    }
    // Else , we have some available rewards to decompound
    let mut luna_rewards = state.backing_luna * Uint128::one() - state.wlsd_supply;

    let mut rewards_to_decompound = (Decimal::from_ratio(state.lsd_balance, 1u128)
        - (Decimal::from_ratio(state.wlsd_supply, 1u128) / state.lsd_exchange_rate))
            * Uint128::one();

    // We substract 1 to the rewards to decompound so that we don't screw up the underlying value of the wrapper token
    // The underlying value should be if possible always above 1 luna per wrapper token (slashing events should happen as little often as possible)
    if rewards_to_decompound > Uint128::zero(){
        rewards_to_decompound -= Uint128::one();
        luna_rewards -= state.lsd_exchange_rate * Uint128::one();
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

pub fn execute_decompound<T: LSDStateResponseTrait + for<'a> Deserialize<'a>>(
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
    let slashing_error = ContractError::Std(StdError::generic_err("No rewards to decompound"));
    let (out_messages, accrued_rewards) = match compute_accrued_rewards::<T>(deps.as_ref(), env) {
        Err(err) => {
            if err == slashing_error {
                Ok((vec![], AccruedRewards::default()))
            } else {
                Err(err)
            }
        }
        Ok(rewards) => {
            let decompound_messages = if !rewards.lsd_rewards.is_zero() {
                vec![CosmosMsg::Bank(BankMsg::Send {
                    to_address: recipient.to_string(),
                    amount: vec![
                        Coin{
                            denom: lsd_contract.denom,
                            amount: rewards.lsd_rewards
                        }
                    ]
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
    //Err(StdError::GenericErr { msg: "No Migrate Implemented".to_string() })
    Ok(Response::default())
}