use basset::external::LSDStateResponseTrait;
use basset::wrapper::ExecuteMsg;
use cavern_lsd_wrapper_token::msg::TokenInitMsg;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::Binary;
use cosmwasm_std::Decimal;
use cosmwasm_std::Deps;
use cosmwasm_std::DepsMut;
use cosmwasm_std::Env;
use cosmwasm_std::MessageInfo;
use cosmwasm_std::Response;
use cosmwasm_std::StdResult;
use cosmwasm_std::Uint128;
use cosmwasm_std::{entry_point, Coin};
use cw20_base::msg::MigrateMsg;
use cw20_base::msg::QueryMsg;
use cw20_base::ContractError;

#[cw_serde]
pub struct AmpLunaStateResponse {
    pub total_usteak: Uint128,
    pub total_native: Uint128,
    pub exchange_rate: Decimal,
    pub unlocked_coins: Vec<Coin>,
}

impl LSDStateResponseTrait for AmpLunaStateResponse {
    fn exchange_rate(&self) -> Decimal {
        self.exchange_rate
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: TokenInitMsg,
) -> StdResult<Response> {
    cavern_lsd_wrapper_token::contract::instantiate(deps, env, info, msg)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    cavern_lsd_wrapper_token::contract::execute::<AmpLunaStateResponse>(deps, env, info, msg)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    cavern_lsd_wrapper_token::contract::query::<AmpLunaStateResponse>(deps, env, msg)
}
