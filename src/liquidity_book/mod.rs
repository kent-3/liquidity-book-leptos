pub mod constants;
pub mod contract_interfaces;
pub mod curves;
pub mod utils;

use contract_interfaces::*;

use crate::error::Error;
use cosmwasm_std::ContractInfo;
use rsecret::{query::compute::ComputeQuerier, secret_client::CreateQuerierOptions};
use secretrs::utils::EnigmaUtils;
use serde::Serialize;
use std::sync::{Arc, LazyLock};
use tonic_web_wasm_client::Client as WebWasmClient;

// NOTE: LB stuff is currently locked to testnet.

static CHAIN_ID: &str = "pulsar-3";

static GRPC_URL: &str = "https://grpc.pulsar.scrttestnet.com";

static WEB_WASM_CLIENT: LazyLock<WebWasmClient> =
    LazyLock::new(|| WebWasmClient::new(GRPC_URL.to_string()));

static ENIGMA_UTILS: LazyLock<Arc<EnigmaUtils>> = LazyLock::new(|| {
    EnigmaUtils::new(None, CHAIN_ID)
        .expect("Failed to create EnigmaUtils")
        .into()
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
