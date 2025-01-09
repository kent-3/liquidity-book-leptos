use crate::error::Error;
use cosmwasm_std::{Addr, ContractInfo};
use hex_literal::hex;
use keplr::tokens::KeplrToken;
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
// Reuse containers to avoid needing to change this every time.
pub static DEVNET_IO_PUBKEY: [u8; 32] =
    hex!("80171b6f3b84eb5975b72ca51ee86b6ae113c22938e4866e5c2300077a06cd3e");

// pub static CHAIN_ID: &str = "secretdev-1";
// pub static NODE: &str = "http://localhost:1317";

pub static CHAIN_ID: &str = "pulsar-3";
// pub static NODE: &str = "https://api.pulsar.scrttestnet.com";
pub static NODE: &str = "https://grpc.testnet.secretsaturn.net";

// pub static CHAIN_ID: &str = "secret-4";
// pub static NODE: &str = "https://lcd.mainnet.secretsaturn.net";
// pub static NODE: &str = "https://grpc.mainnet.secretsaturn.net";

pub mod contracts {
    use super::CHAIN_ID;
    use ammber_sdk::constants::addrs::get_deployed_contracts;
    use cosmwasm_std::ContractInfo;
    use std::sync::LazyLock;

    // NOTE: I only need the LazyLock due to Addr::unchecked not being const... realistically we
    // shouldn't use the Addr type outside of contracts, but it's kinda pervasive.

    // Extract ContractInfo statics
    macro_rules! define_contract_static {
        ($name:ident, $field:ident) => {
            pub static $name: LazyLock<ContractInfo> =
                LazyLock::new(|| get_deployed_contracts(CHAIN_ID).$field.clone());
        };
    }

    // Define statics for specific contracts
    define_contract_static!(LB_QUOTER, lb_quoter);
    define_contract_static!(LB_ROUTER, lb_router);
    define_contract_static!(LB_FACTORY, lb_factory);
    // TODO: these 3 should not be needed!
    define_contract_static!(LB_PAIR, lb_pair);
    define_contract_static!(LB_AMBER, snip25);
    define_contract_static!(LB_SSCRT, snip20);
}

// TODO:
// need one map from token address to token info (the thing with the name, symbol, decimals)
// use a separate map for address -> code hash (can be used for any contract)
// or else include code hash in the token info (like the sf_token_map does)

// Example Keplr token mapping
// "secret1dtghxvrx35nznt8es3fwxrv4qh56tvxv22z79d": {
//   "contractAddress": "secret1dtghxvrx35nznt8es3fwxrv4qh56tvxv22z79d",
//   "imageUrl": "https://raw.githubusercontent.com/chainapsis/keplr-contract-registry/main/images/secret/sgraviton.svg",
//   "metadata": {
//     "name": "Secret GRAVITON",
//     "symbol": "SGRAVITON",
//     "decimals": 6
//   }
// },

pub static KEPLR_TOKEN_MAP: LazyLock<HashMap<String, KeplrToken>> = LazyLock::new(|| {
    let json = include_str!(concat!(env!("OUT_DIR"), "/keplr_token_map.json"));
    serde_json::from_str(json).expect("Failed to deserialize token_map")
});

// TODO: produce json files for these
pub static DEV_TOKEN_MAP: LazyLock<Arc<HashMap<String, Token>>> = LazyLock::new(|| {
    let json = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/public/test_tokens_dev.json"
    ));
    let assets: Vec<Token> = serde_json::from_str(json).expect("Failed to deserialize token_map");

    let mut token_map: HashMap<String, Token> = HashMap::new();

    for asset in assets {
        token_map.insert(asset.contract_address.clone(), asset);
    }

    Arc::new(token_map)
});

// TODO: produce json files for these
pub static PULSAR_TOKEN_MAP: LazyLock<Arc<HashMap<String, Token>>> = LazyLock::new(|| {
    let json = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/public/test_tokens_pulsar.json"
    ));
    let assets: Vec<Token> = serde_json::from_str(json).expect("Failed to deserialize token_map");

    let mut token_map: HashMap<String, Token> = HashMap::new();

    for asset in assets {
        token_map.insert(asset.contract_address.clone(), asset);
    }

    Arc::new(token_map)
});

pub static MAINNET_TOKEN_MAP: LazyLock<Arc<HashMap<String, Token>>> = LazyLock::new(|| {
    let json = include_str!(concat!(env!("OUT_DIR"), "/sf_token_map.json"));
    serde_json::from_str(json).expect("Failed to deserialize token_map")
});

pub fn get_token_map(chain_id: &str) -> Arc<HashMap<String, Token>> {
    match chain_id {
        "secretdev-1" => Arc::clone(&DEV_TOKEN_MAP),
        "pulsar-3" => Arc::clone(&PULSAR_TOKEN_MAP),
        "secret-4" => Arc::clone(&MAINNET_TOKEN_MAP),
        _ => panic!("invalid chain id!"),
    }
}

pub static TOKEN_MAP: LazyLock<Arc<HashMap<String, Token>>> =
    LazyLock::new(|| get_token_map(CHAIN_ID));

// For each token we know about at compile time, map from symbol to address
pub static SYMBOL_TO_ADDR: LazyLock<HashMap<String, String>> = LazyLock::new(|| {
    get_token_map(CHAIN_ID)
        .iter()
        .map(|(contract_address, token)| (token.symbol.clone(), contract_address.clone()))
        .collect()
});

pub static WEB_WASM_CLIENT: LazyLock<WebWasmClient> =
    LazyLock::new(|| WebWasmClient::new(NODE.to_string()));

// used for read-only client queries
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

// TODO: can we implement this Querier trait for something that performs the queries from the frontend?
// use cosmwasm_std::QuerierWrapper;
// use cosmwasm_std::Querier;

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
