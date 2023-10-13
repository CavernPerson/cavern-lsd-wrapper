use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cw_storage_plus::Item;

use basset::hub::{Config, State};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Parameters {
    pub reward_denom: String,
}

pub const CONFIG: Item<Config> = Item::new("\u{0}\u{6}config");
pub const PARAMETERS: Item<Parameters> = Item::new("\u{0}\u{b}parameteres");
pub const STATE: Item<State> = Item::new("\u{0}\u{5}state");
