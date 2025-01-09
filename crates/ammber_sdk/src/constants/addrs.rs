use super::ChainId;
use cosmwasm_std::{Addr, ContractInfo};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, LazyLock};

pub static BATCH_QUERY_ROUTER: LazyLock<HashMap<ChainId, ContractInfo>> = LazyLock::new(|| {
    HashMap::from_iter([
        (
            ChainId::Dev,
            ContractInfo {
                // WARN: this address needs to be updated whenever creating a new localsecret
                address: Addr::unchecked("secret15zvwtzf38yqhdzt2svdk7mnc5ha24493tqydn2"),
                code_hash: "1c7e86ba4fdb6760e70bf08a7df7f44b53eb0b23290e3e69ca96140810d4f432"
                    .to_string(),
            },
        ),
        (
            ChainId::Pulsar,
            ContractInfo {
                address: Addr::unchecked("secret19a9emj5ym504a5824vc7g5awaj2z5nwsl8jpcz"),
                code_hash: "1c7e86ba4fdb6760e70bf08a7df7f44b53eb0b23290e3e69ca96140810d4f432"
                    .to_string(),
            },
        ),
        (
            ChainId::Secret,
            ContractInfo {
                address: Addr::unchecked("secret15mkmad8ac036v4nrpcc7nk8wyr578egt077syt"),
                code_hash: "1c7e86ba4fdb6760e70bf08a7df7f44b53eb0b23290e3e69ca96140810d4f432"
                    .to_string(),
            },
        ),
    ])
});

// TODO: Once more stable, these values should be hardcoded instead.

pub static LB_QUOTER: LazyLock<HashMap<ChainId, ContractInfo>> = LazyLock::new(|| {
    HashMap::from_iter([
        (ChainId::Dev, DEV_CONTRACTS.lb_quoter.clone()),
        (ChainId::Pulsar, PULSAR_CONTRACTS.lb_quoter.clone()),
    ])
});

pub static LB_ROUTER: LazyLock<HashMap<ChainId, ContractInfo>> = LazyLock::new(|| {
    HashMap::from_iter([
        (ChainId::Dev, DEV_CONTRACTS.lb_router.clone()),
        (ChainId::Pulsar, PULSAR_CONTRACTS.lb_router.clone()),
    ])
});

pub static LB_FACTORY: LazyLock<HashMap<ChainId, ContractInfo>> = LazyLock::new(|| {
    HashMap::from_iter([
        (ChainId::Dev, DEV_CONTRACTS.lb_factory.clone()),
        (ChainId::Pulsar, PULSAR_CONTRACTS.lb_factory.clone()),
    ])
});

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeployedContracts {
    pub admin_auth: ContractInfo,
    pub query_auth: ContractInfo,
    pub snip20: ContractInfo,
    pub snip25: ContractInfo,
    pub lb_factory: ContractInfo,
    pub lb_pair: ContractInfo,
    pub lb_token: ContractInfo,
    pub lb_router: ContractInfo,
    pub lb_quoter: ContractInfo,
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
    let chain_id: ChainId = chain_id.parse().expect("invalid chain id");

    match chain_id {
        ChainId::Dev => Arc::clone(&DEV_CONTRACTS),
        ChainId::Pulsar => Arc::clone(&PULSAR_CONTRACTS),
        ChainId::Secret => todo!(),
    }
}
