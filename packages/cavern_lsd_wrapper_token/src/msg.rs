use cw20::{Cw20Coin};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct TokenInitMsg {
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub initial_balances: Vec<Cw20Coin>,
    
    // only hub contract can call decompound
    pub hub_contract: String,

    pub lsd_hub_contract: String,
    pub lsd_token_contract: String,
}
