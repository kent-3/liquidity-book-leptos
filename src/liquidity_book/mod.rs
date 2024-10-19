pub mod constants;
pub mod contract_interfaces;
pub mod curves;

use constants::addrs::{LB_FACTORY_CONTRACT, LB_PAIR_CONTRACT};
use contract_interfaces::*;

use cosmwasm_std::ContractInfo;
use rsecret::{query::compute::ComputeQuerier, secret_client::CreateQuerierOptions};
use secretrs::utils::EnigmaUtils;
use tonic_web_wasm_client::Client;

use crate::keplr::Keplr;

static CHAIN_ID: &str = "pulsar-3";
static GRPC_URL: &str = "https://grpc.pulsar.scrttestnet.com";

pub trait Querier {
    async fn do_query(&self, contract: &ContractInfo) -> String;
}

impl Querier for lb_factory::QueryMsg {
    async fn do_query(&self, contract: &ContractInfo) -> String {
        let contract_address = &contract.address;
        let code_hash = &contract.code_hash;

        let client = Client::new(GRPC_URL.to_string());
        let encryption_utils = EnigmaUtils::new(None, CHAIN_ID).unwrap();
        let compute = ComputeQuerier::new(client, encryption_utils.into());
        let query = self;
        compute
            .query_secret_contract(contract_address, code_hash, query)
            .await
            .unwrap_or_else(|err| err.to_string())
    }
}

impl Querier for lb_pair::QueryMsg {
    async fn do_query(&self, contract: &ContractInfo) -> String {
        let contract_address = &contract.address;
        let code_hash = &contract.code_hash;

        let client = Client::new(GRPC_URL.to_string());
        let encryption_utils = EnigmaUtils::new(None, CHAIN_ID).unwrap();
        let compute = ComputeQuerier::new(client, encryption_utils.into());
        let query = self;
        compute
            .query_secret_contract(contract_address, code_hash, query)
            .await
            .unwrap_or_else(|err| err.to_string())
    }
}
