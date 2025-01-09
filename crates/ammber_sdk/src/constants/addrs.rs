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

// TODO: I only need the LazyLock due to Addr::unchecked not being const... realistically we
// shouldn't use the Addr type outside of contracts, but it's kinda pervasive.

// Extract ContractInfo statics
macro_rules! define_contract_static {
    ($name:ident, $field:ident) => {
        pub static $name: LazyLock<Arc<ContractInfo>> = LazyLock::new(|| {
            let contracts = get_deployed_contracts("pulsar-3"); // Default to pulsar-3 or dynamically fetch
            Arc::new(ContractInfo {
                address: contracts.$field.address.clone(),
                code_hash: contracts.$field.code_hash.clone(),
            })
        });
    };
}

// Define statics for specific contracts
define_contract_static!(LB_FACTORY, lb_factory);
define_contract_static!(LB_PAIR, lb_pair);
define_contract_static!(LB_AMBER, snip25);
define_contract_static!(LB_SSCRT, snip20);
define_contract_static!(LB_ROUTER, lb_router);
define_contract_static!(LB_QUOTER, lb_quoter);
