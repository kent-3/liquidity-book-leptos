#![allow(unused)]
use crate::keplr::tokens::{keplr_contract_registry_tokens, ContractInfo};
use rsecret::query::compute::ComputeQuerier;
use secretrs::utils::EnigmaUtils;
use std::{
    collections::HashMap,
    sync::{Arc, LazyLock, RwLock},
};
use tonic_web_wasm_client::Client as WebWasmClient;

pub static CHAIN_ID: &str = "secret-4";
pub static LCD_URL: &str = "https://lcd.mainnet.secretsaturn.net";
pub static GRPC_URL: &str = "https://grpc.mainnet.secretsaturn.net";

pub static TOKEN_MAP: LazyLock<HashMap<String, ContractInfo>> = LazyLock::new(|| {
    let json = include_str!(concat!(env!("OUT_DIR"), "/token_map.json"));
    serde_json::from_str(json).expect("Failed to deserialize token_map")
});

pub static WEB_WASM_CLIENT: LazyLock<WebWasmClient> =
    LazyLock::new(|| WebWasmClient::new(GRPC_URL.to_string()));

pub static COMPUTE_QUERY: LazyLock<ComputeQuerier<WebWasmClient, EnigmaUtils>> =
    LazyLock::new(|| {
        ComputeQuerier::new(
            // WebWasmClient::new(GRPC_URL.to_string()),
            WEB_WASM_CLIENT.clone(),
            EnigmaUtils::new(None, CHAIN_ID)
                .expect("Failed to create EnigmaUtils")
                .into(),
        )
    });

pub fn compute_querier(
    url: impl Into<String>,
    chain_id: &str,
) -> ComputeQuerier<WebWasmClient, EnigmaUtils> {
    ComputeQuerier::new(
        WebWasmClient::new(url.into()),
        EnigmaUtils::new(None, chain_id)
            .expect("Failed to create EnigmaUtils")
            .into(),
    )
}
