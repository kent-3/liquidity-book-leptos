pub mod constants;
pub mod contract_interfaces;
pub mod curves;
pub mod utils;

use contract_interfaces::*;

use crate::error::Error;
use cosmwasm_std::ContractInfo;
use hex_literal::hex;
use rsecret::{query::compute::ComputeQuerier, secret_client::CreateQuerierOptions};
use secretrs::grpc_clients::RegistrationQueryClient;
use secretrs::utils::EnigmaUtils;
use serde::Serialize;
use std::sync::{Arc, LazyLock, OnceLock};
use tonic_web_wasm_client::Client as WebWasmClient;
use tracing::info;

// NOTE: LB stuff is currently locked to testnet.

static CHAIN_ID: &str = "pulsar-3";
static GRPC_URL: &str = "https://grpc.pulsar.scrttestnet.com";

// static CHAIN_ID: &str = "secretdev-1";
// static GRPC_URL: &str = "http://localhost:9091";
// WARN: This key is randomly generated when localsecret is started for the first time.
// Reuse containers to avoid needing changing this every time.
pub static DEVNET_IO_PUBKEY: [u8; 32] =
    hex!("7e6cfbda947c3d7070a76136ed5bb9bbde6f99485713bf005eb4a2799b05de62");

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

static WEB_WASM_CLIENT: LazyLock<WebWasmClient> =
    LazyLock::new(|| WebWasmClient::new(GRPC_URL.to_string()));

static ENIGMA_UTILS: LazyLock<Arc<EnigmaUtils>> = LazyLock::new(|| {
    if CHAIN_ID == "secretdev-1" {
        EnigmaUtils::from_io_key(None, DEVNET_IO_PUBKEY).into()
    } else {
        EnigmaUtils::new(None, CHAIN_ID)
            .expect("Failed to create EnigmaUtils")
            .into()
    }
});

static COMPUTE_QUERIER: LazyLock<ComputeQuerier<WebWasmClient, EnigmaUtils>> =
    LazyLock::new(|| ComputeQuerier::new(WEB_WASM_CLIENT.clone(), ENIGMA_UTILS.clone()));

pub trait Querier {
    async fn do_query(&self, contract: &ContractInfo) -> Result<String, Error>;
}

impl<T: Serialize + Send + Sync> Querier for T {
    async fn do_query(&self, contract: &ContractInfo) -> Result<String, Error> {
        let contract_address = &contract.address;
        let code_hash = &contract.code_hash;
        let query = self;

        COMPUTE_QUERIER
            .query_secret_contract(contract_address, code_hash, query)
            .await
            .map_err(Into::into)
    }
}
