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
    pub snip25: DeployedContractInfo,
    pub lb_factory: DeployedContractInfo,
    pub lb_pair: DeployedContractInfo,
    pub lb_token: DeployedContractInfo,
    pub lb_router: DeployedContractInfo,
    pub lb_staking: DeployedContractInfo,
}

pub static LB_CONTRACTS: LazyLock<Arc<DeployedContracts>> = LazyLock::new(|| {
    let lb_contracts = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/public/lb_contracts.json"
    ));
    serde_json::from_str::<DeployedContracts>(lb_contracts)
        .expect("Failed to deserialize contract info")
        .into()
});

// TODO: I only need the LazyLock due to Addr::unchecked not being const...
pub static LB_FACTORY_CONTRACT: LazyLock<Arc<ContractInfo>> = LazyLock::new(|| {
    // ContractInfo {
    //     address: Addr::unchecked("secret1dp50y8ehgrew2jne6jyews45k64ulfxtmqewjd"),
    //     code_hash: "0db90ee73825a5464f487655e030a8e5972f37a3f11536e5172d036a5ff6e96c".to_string(),
    // }
    // .into()
    ContractInfo {
        address: LB_CONTRACTS.lb_factory.address.clone(),
        code_hash: LB_CONTRACTS.lb_factory.code_hash.clone(),
    }
    .into()
});

// NOTE: This should not be a static value, but it's here for dev purposes.
pub static LB_PAIR_CONTRACT: LazyLock<Arc<ContractInfo>> = LazyLock::new(|| {
    // ContractInfo {
    //     address: Addr::unchecked("secret1pt5nd3fuevamy5lqcv53jqsvytspmknanf5c28"),
    //     code_hash: "9768cfd5753a7fa2b51b30a3fc41632df2b3bc31801dece2d6111f321a3e4252".to_string(),
    // }
    // .into()
    ContractInfo {
        address: LB_CONTRACTS.lb_pair.address.clone(),
        code_hash: LB_CONTRACTS.lb_pair.code_hash.clone(),
    }
    .into()
});

// NOTE: This should not be a static value, but it's here for dev purposes.
pub static LB_STAKING_CONTRACT: LazyLock<Arc<ContractInfo>> = LazyLock::new(|| {
    // ContractInfo {
    //     address: Addr::unchecked("secret1rqdgd33sg7kz7msz5dlury3dqvtghglrpu5dkj"),
    //     code_hash: "16946a1f044d2ad55baf4a108bae2090d491cb47f92400c4c99cdadd6a344cdd".to_string(),
    // }
    // .into()

    ContractInfo {
        address: LB_CONTRACTS.lb_staking.address.clone(),
        code_hash: LB_CONTRACTS.lb_staking.code_hash.clone(),
    }
    .into()
});
