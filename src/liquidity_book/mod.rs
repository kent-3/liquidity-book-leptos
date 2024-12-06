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

// static CHAIN_ID: &str = "pulsar-3";
// static GRPC_URL: &str = "https://api.pulsar.scrttestnet.com";

// TODO: don't rely on this. this module should not depend on the main crate.
use crate::constants::{CHAIN_ID, DEVNET_IO_PUBKEY, GRPC_URL};

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
