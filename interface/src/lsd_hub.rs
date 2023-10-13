use cw_orch::{
    interface,
    prelude::*,
};
use basset::hub::{
    ExecuteMsg, InstantiateMsg, QueryMsg,
};

use cavern_lsd_hub::contract::{instantiate, execute, query, migrate};

#[interface(InstantiateMsg, ExecuteMsg, QueryMsg, Empty)]
pub struct LsdHub;

impl<Chain: CwEnv> Uploadable for LsdHub<Chain> {
    /// Return the path to the wasm file corresponding to the contract
    fn wasm(&self) -> WasmPath {
        artifacts_dir_from_workspace!()
            .find_wasm_path("cavern_lsd_hub")
            .unwrap()
    }
    /// Returns a CosmWasm contract wrapper
    fn wrapper(&self) -> Box<dyn MockContract<Empty>> {
        Box::new(
            ContractWrapper::new_with_empty(
                execute,
                instantiate,
                query,
            )
            .with_migrate(migrate)
        )
    }
}
