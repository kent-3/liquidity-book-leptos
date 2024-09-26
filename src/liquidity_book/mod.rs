pub mod constants;
pub mod contract_interfaces;
pub mod curves;

use constants::addrs::LB_PAIR_CONTRACT;
use contract_interfaces::*;

use rsecret::{query::compute::ComputeQuerier, secret_network_client::CreateQuerierOptions};
use tonic_web_wasm_client::Client;

static CHAIN_ID: &str = "pulsar-3";
static GRPC_URL: &str = "https://grpc.pulsar.scrttestnet.com";

pub trait Querier {
    async fn do_query(&self) -> String;
}

impl Querier for lb_factory::QueryMsg {
    async fn do_query(&self) -> String {
        let contract_address = "secret1dp50y8ehgrew2jne6jyews45k64ulfxtmqewjd";
        let code_hash = "0db90ee73825a5464f487655e030a8e5972f37a3f11536e5172d036a5ff6e96c";

        let client = Client::new(GRPC_URL.to_string());
        let encryption_utils = secretrs::EncryptionUtils::new(None, CHAIN_ID).unwrap();
        let compute = ComputeQuerier::new(client, encryption_utils);
        let query = self;
        compute
            .query_secret_contract(contract_address, code_hash, query)
            .await
            .unwrap_or_else(|err| err.to_string())
    }
}

impl Querier for lb_pair::QueryMsg {
    async fn do_query(&self) -> String {
        let contract_address = LB_PAIR_CONTRACT.address.clone();
        let code_hash = LB_PAIR_CONTRACT.code_hash.clone();

        let client = Client::new(GRPC_URL.to_string());
        let encryption_utils = secretrs::EncryptionUtils::new(None, CHAIN_ID).unwrap();
        let compute = ComputeQuerier::new(client, encryption_utils);
        let query = self;
        compute
            .query_secret_contract(contract_address, code_hash, query)
            .await
            .unwrap_or_else(|err| err.to_string())
    }
}
