use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Decimal256, Uint128};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub hub_contract: String,
    pub reward_denom: String,
    pub astroport_addr: String,
    pub phoenix_addr: String,
    pub terraswap_addr: String,
    // Known tokens to swap from to the stable_token
    pub known_tokens: Vec<String>
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    ////////////////////
    /// Owner's operations
    ///////////////////

    /// Swap all of the balances to uusd.
    SwapToRewardDenom {},

    /// Updates the contract config
    UpdateConfig { 
        custody_contract: Option<String>,
        known_tokens: Option<Vec<String>>,
        owner: Option<String>
    },
    ////////////////////
    /// User's operations
    ///////////////////

    /// return the accrued reward in uusd to the user.
    ClaimRewards { recipient: Option<String> },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    AccruedRewards { address: String },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub hub_contract: String,
    pub reward_denom: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AccruedRewardsResponse {
    pub rewards: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct HolderResponse {
    pub address: String,
    pub balance: Uint128,
    pub index: Decimal256,
    pub pending_rewards: Decimal256,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct HoldersResponse {
    pub holders: Vec<HolderResponse>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {}
