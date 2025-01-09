use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::debug;

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Metadata {
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct KeplrToken {
    pub contract_address: String,
    pub image_url: String,
    pub metadata: Metadata,
}

// pub fn keplr_contract_registry_tokens() -> HashMap<String, ContractInfo> {
//     let json = include_str!(concat!(env!("OUT_DIR"), "/keplr_token_map.json"));
//     let token_map: HashMap<String, ContractInfo> =
//         serde_json::from_str(json).expect("Failed to deserialize token_map");
//
//     debug!("Loaded {} tokens", token_map.len());
//
//     token_map
// }
