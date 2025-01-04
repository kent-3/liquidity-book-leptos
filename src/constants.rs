use crate::{error::Error, keplr::tokens::ContractInfo};
use cosmwasm_std::Addr;
use hex_literal::hex;
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
// WARN: This key is randomly generated when localsecret is started for the first time.
// Reuse containers to avoid needing changing this every time.
pub static DEVNET_IO_PUBKEY: [u8; 32] =
    hex!("c8a3ef51d2d4a285ce2a97ccf60a7372fb978e41768556ba4450ecc26247ac4b");

// pub static CHAIN_ID: &str = "secretdev-1";
// pub static GRPC_URL: &str = "http://localhost:1317";

pub static CHAIN_ID: &str = "pulsar-3";
// pub static GRPC_URL: &str = "https://api.pulsar.scrttestnet.com";
pub static GRPC_URL: &str = "https://grpc.testnet.secretsaturn.net";

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
    if CHAIN_ID == "secretdev-1" {
        EnigmaUtils::from_io_key(None, DEVNET_IO_PUBKEY).into()
    } else {
        EnigmaUtils::new(None, CHAIN_ID)
            .expect("Failed to create EnigmaUtils")
            .into()
    }
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

// TODO: kinda awkward. it would be cooler to use those ILb* types but they take deps.querier. I
// bet we could make a compatible QuerierWrapper, but that sounds advanced.
// TODO: I really need to find a better location for this...

pub trait Querier {
    async fn do_query(&self, contract: &cosmwasm_std::ContractInfo) -> Result<String, Error>;
}

impl<T: Serialize + Send + Sync> Querier for T {
    async fn do_query(&self, contract: &cosmwasm_std::ContractInfo) -> Result<String, Error> {
        let contract_address = &contract.address;
        let code_hash = &contract.code_hash;
        let query = self;

        COMPUTE_QUERIER
            .query_secret_contract(contract_address, code_hash, query)
            .await
            .map_err(Into::into)
    }
}

// TODO: Querying of io key is problematic due to async. Explore further.
//
// pub static DEVNET_IO_PUBKEY: OnceLock<[u8; 32]> = OnceLock::new();
// pub async fn get_io_key(channel: tonic_web_wasm_client::Client) -> [u8; 32] {
//     let mut secret_registration = RegistrationQueryClient::new(channel.clone());
//     let enclave_key_bytes = secret_registration
//         .tx_key(())
//         .await
//         .expect("could not obtain IO key")
//         .into_inner()
//         .key;
//     // let enclave_key = hex::encode(&enclave_key_bytes);
//     // info!("Enclave IO Public Key: {:>4}", enclave_key.bright_blue());
//
//     let mut enclave_key = [0u8; 32];
//     enclave_key.copy_from_slice(&enclave_key_bytes[0..32]);
//
//     enclave_key
// }
