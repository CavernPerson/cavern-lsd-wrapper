use basset::external::LSDQueryMsg;

use basset::external::LSDStateResponseTrait;
use cosmwasm_std::BalanceResponse;
use cosmwasm_std::BankQuery;
use cosmwasm_std::StdResult;
use cosmwasm_std::to_binary;
use cosmwasm_std::Deps;
use cosmwasm_std::Env;

use cw20_base::contract::query_token_info;
use cw20_base::ContractError;

use cosmwasm_std::{Decimal, QueryRequest, WasmQuery};
use serde::Deserialize;

use crate::state::LsdContracts;
use crate::state::read_lsd_contract;
use crate::state::WrapperState;


pub fn get_current_exchange_rate<T: LSDStateResponseTrait + for<'a> Deserialize<'a>>(
    deps: Deps,
    env: Env,
    state: &mut WrapperState,
) -> Result<Decimal, ContractError> {
    let lsd_contracts = read_lsd_contract(deps.storage)?;

    // We query how much lsd tokens the contract holds
    let balance: BalanceResponse = deps.querier.query(&QueryRequest::Bank(BankQuery::Balance { address: env.contract.address.to_string(), denom: lsd_contracts.denom.to_string() }))?;


    // Then we query the corresponding luna value of that LSD
    // This is located in the state query of the LSD
    let lsd_state: T = query_lsd_state(deps, &lsd_contracts)?;

    // We now have the number of underlying lunas backing the token
    let luna_backing_token: Decimal = Decimal::from_ratio(balance.amount.amount, 1u128) * lsd_state.exchange_rate();

    // We can divide that by the number of issued tokens to get the exchange rate
    let total_wlsd_supply = query_token_info(deps)?.total_supply;

    state.lsd_exchange_rate = lsd_state.exchange_rate();
    state.wlsd_supply = total_wlsd_supply;
    state.backing_luna = luna_backing_token;
    state.lsd_balance = balance.amount.amount;

    // Luna / WLSD
    if total_wlsd_supply.is_zero(){
        Ok(Decimal::one())
    }else{
        Ok(luna_backing_token / total_wlsd_supply)
    }
}

pub fn query_lsd_state<T: for<'a> Deserialize<'a>>(deps: Deps, lsd_contracts: &LsdContracts) -> StdResult<T>{
 deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: lsd_contracts.hub.to_string(),
        msg: to_binary(&LSDQueryMsg::State {})?,
    }))
}