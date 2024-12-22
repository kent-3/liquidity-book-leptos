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

// TODO: don't rely on this. this module should not depend on the main crate.
use crate::constants::CHAIN_ID;

pub static LB_CONTRACTS: LazyLock<Arc<DeployedContracts>> = LazyLock::new(|| {
    let lb_contracts = if CHAIN_ID == "secretdev-1" {
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/public/lb_contracts_dev.json"
        ))
    } else if CHAIN_ID == "pulsar-3" {
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/public/lb_contracts_pulsar.json"
        ))
    } else {
        todo!()
    };
    serde_json::from_str::<DeployedContracts>(lb_contracts)
        .expect("Failed to deserialize contract info")
        .into()
});

// TODO: I only need the LazyLock due to Addr::unchecked not being const...
pub static LB_FACTORY: LazyLock<Arc<ContractInfo>> = LazyLock::new(|| {
    ContractInfo {
        address: LB_CONTRACTS.lb_factory.address.clone(),
        code_hash: LB_CONTRACTS.lb_factory.code_hash.clone(),
    }
    .into()
});

// NOTE: This should not be a static value, but it is here for dev purposes.
pub static LB_PAIR: LazyLock<Arc<ContractInfo>> = LazyLock::new(|| {
    ContractInfo {
        address: LB_CONTRACTS.lb_pair.address.clone(),
        code_hash: LB_CONTRACTS.lb_pair.code_hash.clone(),
    }
    .into()
});

pub static LB_ROUTER: LazyLock<Arc<ContractInfo>> = LazyLock::new(|| {
    ContractInfo {
        address: LB_CONTRACTS.lb_router.address.clone(),
        code_hash: LB_CONTRACTS.lb_router.code_hash.clone(),
    }
    .into()
});

pub static LB_QUOTER: LazyLock<Arc<ContractInfo>> = LazyLock::new(|| {
    ContractInfo {
        address: LB_CONTRACTS.lb_quoter.address.clone(),
        code_hash: LB_CONTRACTS.lb_quoter.code_hash.clone(),
    }
    .into()
});
