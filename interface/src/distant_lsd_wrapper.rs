use cw_orch::{
    interface,
    prelude::*,
};

use wrapper_implementations::coin::StrideInitMsg;
use basset::wrapper::ExecuteMsg;
use cw20_base::msg::QueryMsg;

use cavern_lsd_wrapper::{instantiate, execute, query};

#[interface(StrideInitMsg, ExecuteMsg, QueryMsg, Empty)]
pub struct LsdWrapper;

impl<Chain: CwEnv> Uploadable for LsdWrapper<Chain> {
    /// Return the path to the wasm file corresponding to the contract
    fn wasm(&self) -> WasmPath {
        artifacts_dir_from_workspace!()
            .find_wasm_path("st_luna_token")
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
        )
    }
}
