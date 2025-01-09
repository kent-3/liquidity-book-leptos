use cosmwasm_std::{Addr, ContractInfo};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{Arc, LazyLock},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeployedContractInfo {
    pub address: Addr,
    pub code_hash: String,
    pub code_id: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeployedContracts {
    pub admin_auth: DeployedContractInfo,
    pub query_auth: DeployedContractInfo,
    pub snip20: DeployedContractInfo,
    pub snip25: DeployedContractInfo,
    pub lb_factory: DeployedContractInfo,
    pub lb_pair: DeployedContractInfo,
    pub lb_token: DeployedContractInfo,
    pub lb_router: DeployedContractInfo,
    pub lb_quoter: DeployedContractInfo,
}

// Embed and deserialize the JSON files at compile time
static DEV_CONTRACTS: LazyLock<Arc<DeployedContracts>> = LazyLock::new(|| {
    let data = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/deployments/lb_contracts_dev.json"
    ));
    Arc::new(serde_json::from_str(data).expect("Failed to deserialize lb_contracts_dev.json"))
});

static PULSAR_CONTRACTS: LazyLock<Arc<DeployedContracts>> = LazyLock::new(|| {
    let data = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/deployments/lb_contracts_pulsar.json"
    ));
    Arc::new(serde_json::from_str(data).expect("Failed to deserialize lb_contracts_pulsar.json"))
});

// static MAINNET_CONTRACTS: LazyLock<Arc<DeployedContracts>> = LazyLock::new(|| {
//     let data = include_str!(concat!(
//         env!("CARGO_MANIFEST_DIR"),
//         "/deployments/lb_contracts_mainnet.json"
//     ));
//     Arc::new(serde_json::from_str(data).expect("Failed to deserialize lb_contracts_mainnet.json"))
// });

// Chain-to-contract mapping
pub fn get_deployed_contracts(chain_id: &str) -> Arc<DeployedContracts> {
    match chain_id {
        "secretdev-1" => Arc::clone(&DEV_CONTRACTS),
        "pulsar-3" => Arc::clone(&PULSAR_CONTRACTS),
        // "secret-4" => Arc::clone(&MAINNET_CONTRACTS),
        _ => panic!("invalid chain id!"),
    }
}
