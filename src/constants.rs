use crate::keplr::tokens::ContractInfo;
use cosmwasm_std::Addr;
use rsecret::query::compute::ComputeQuerier;
use secretrs::utils::EnigmaUtils;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{Arc, LazyLock},
};
use tonic_web_wasm_client::Client as WebWasmClient;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Token {
    pub contract_address: String,
    pub code_hash: String,
    pub decimals: u8,
    pub name: String,
    pub symbol: String,
    pub display_name: Option<String>,
    pub denom: Option<String>,
    pub version: Option<String>,
}
// pub static CHAIN_ID: &str = "secretdev-1";
// pub static GRPC_URL: &str = "http://localhost:9091";

pub static CHAIN_ID: &str = "pulsar-3";
pub static GRPC_URL: &str = "https://grpc.pulsar.scrttestnet.com";

// pub static CHAIN_ID: &str = "secret-4";
// pub static LCD_URL: &str = "https://lcd.mainnet.secretsaturn.net";
// pub static GRPC_URL: &str = "https://grpc.mainnet.secretsaturn.net";

pub static KEPLR_TOKEN_MAP: LazyLock<HashMap<String, ContractInfo>> = LazyLock::new(|| {
    let json = include_str!(concat!(env!("OUT_DIR"), "/keplr_token_map.json"));
    serde_json::from_str(json).expect("Failed to deserialize token_map")
});

pub static TOKEN_MAP: LazyLock<HashMap<String, Token>> = LazyLock::new(|| {
    let json = include_str!(concat!(env!("OUT_DIR"), "/sf_token_map.json"));
    serde_json::from_str(json).expect("Failed to deserialize token_map")
});

pub static SYMBOL_TO_ADDR: LazyLock<HashMap<String, Addr>> = LazyLock::new(|| {
    TOKEN_MAP
        .iter()
        .map(|(contract_address, token)| {
            (
                token.symbol.clone(),
                Addr::unchecked(contract_address.clone()),
            )
        })
        .collect()
});

pub static WEB_WASM_CLIENT: LazyLock<WebWasmClient> =
    LazyLock::new(|| WebWasmClient::new(GRPC_URL.to_string()));

pub static ENIGMA_UTILS: LazyLock<Arc<EnigmaUtils>> = LazyLock::new(|| {
    EnigmaUtils::new(None, CHAIN_ID)
        .expect("Failed to create EnigmaUtils")
        .into()
});

pub static COMPUTE_QUERIER: LazyLock<ComputeQuerier<WebWasmClient, EnigmaUtils>> =
    LazyLock::new(|| ComputeQuerier::new(WEB_WASM_CLIENT.clone(), ENIGMA_UTILS.clone()));

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
