use cw_orch::{daemon::{ChainInfo, ChainKind, NetworkInfo}, prelude::ContractInstance};
use cw_orch_fork_mock::ForkMock;
use interface::{LsdHub, HubExecuteMsgFns};
use cw_orch::prelude::Addr;

pub const MIGALOO: NetworkInfo = NetworkInfo{
    id: "migaloo",
    pub_address_prefix: "migaloo",
    coin_type: 118
};

pub const MIGALOO_1: ChainInfo = ChainInfo{
    chain_id: "migaloo-1",
    gas_denom: "uwhale",
    gas_price: 1f64,
    grpc_urls: &["migaloo-grpc.polkachu.com:20790"],
    lcd_url: None,
    fcd_url: None,
    network_info: MIGALOO,
    kind: ChainKind::Mainnet
};

fn update_global() -> anyhow::Result<()>{

    // First we create a fork testing object
    pretty_env_logger::init();

    let app = ForkMock::new(MIGALOO_1);

    let hub_contract = LsdHub::new("ampWhale:hub", app.clone());
    hub_contract.set_address(&Addr::unchecked("migaloo196slekmpf56972v6456lurvuma92pq265gs700zztkx2j83zy7xstmmqy8"));

    // We try to update the global index
    hub_contract.update_global_index()?;


    Ok(())
}

fn main(){
    update_global().unwrap()
}