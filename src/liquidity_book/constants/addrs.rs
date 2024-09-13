use shade_protocol::c_std::Addr;
use shade_protocol::c_std::ContractInfo;
use std::sync::LazyLock;

// TODO: get the contract information dynamically
//
// #[derive(serde::Deserialize)]
// struct ContractInfo {
//     address: String,
//     code_hash: String,
// }
//
// static CONTRACTS: LazyLock<Vec<ContractInfo>> = LazyLock::new(|| {
//     let contracts = include_str!("path/to/your/file.json");
//     serde_json::from_str(contracts).expect("Failed to deserialize contract info")
// });

pub static LB_FACTORY_CONTRACT: LazyLock<ContractInfo> = LazyLock::new(|| ContractInfo {
    address: Addr::unchecked("secret1dp50y8ehgrew2jne6jyews45k64ulfxtmqewjd"),
    code_hash: "0db90ee73825a5464f487655e030a8e5972f37a3f11536e5172d036a5ff6e96c".to_string(),
});

pub static LB_PAIR_CONTRACT: LazyLock<ContractInfo> = LazyLock::new(|| ContractInfo {
    address: Addr::unchecked("secret1pt5nd3fuevamy5lqcv53jqsvytspmknanf5c28"),
    code_hash: "9768cfd5753a7fa2b51b30a3fc41632df2b3bc31801dece2d6111f321a3e4252".to_string(),
});

pub static LB_STAKING_CONTRACT: LazyLock<ContractInfo> = LazyLock::new(|| ContractInfo {
    address: Addr::unchecked("secret1rqdgd33sg7kz7msz5dlury3dqvtghglrpu5dkj"),
    code_hash: "16946a1f044d2ad55baf4a108bae2090d491cb47f92400c4c99cdadd6a344cdd".to_string(),
});
