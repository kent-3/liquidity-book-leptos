use crate::prelude::*;
use crate::Error;
use ammber_sdk::contract_interfaces::{
    lb_factory::{self, *},
    lb_pair::{self, *},
    lb_quoter::{self, *},
};
use cosmwasm_std::{Addr, ContractInfo, StdResult, Uint128};
use leptos::prelude::*;
use liquidity_book::core::TokenType;
use rsecret::query::compute::ComputeQuerier;
use secretrs::utils::EnigmaUtils;
use send_wrapper::SendWrapper;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::sync::{Arc, LazyLock};
use tonic_web_wasm_client::Client as WebWasmClient;
use tracing::{debug, error};

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

// TODO: can we implement this Querier trait for something that performs the queries from the frontend?
// use cosmwasm_std::QuerierWrapper;
// use cosmwasm_std::Querier;

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

// TODO: Each response can be either the specific expected response struct, or any of the potential
// error types within the contract. Figure out how to handle this.

// this works somehow
pub fn chain_query<T>(
    code_hash: String,
    contract_address: String,
    query: impl Serialize + Send + Sync + 'static,
) -> impl std::future::Future<Output = Result<T, Error>> + Send
where
    T: DeserializeOwned + 'static,
{
    SendWrapper::new(async move {
        COMPUTE_QUERIER
            .query_secret_contract(contract_address, code_hash, query)
            .await
            .inspect(|response| debug!("{response:?}"))
            .inspect_err(|e| error!("{e}"))
            .and_then(|response| Ok(serde_json::from_str::<T>(&response)?))
            .map_err(Into::into)
    })
}

/// A thin wrapper around `ContractInfo` that provides additional
/// methods to interact with the LB Factory contract.
#[derive(Serialize, Deserialize, Clone)]
pub struct ILbFactory(pub ContractInfo);

impl std::ops::Deref for ILbFactory {
    type Target = ContractInfo;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[allow(unused)]
impl ILbFactory {
    pub async fn get_number_of_lb_pairs(&self) -> Result<u32, Error> {
        chain_query::<NumberOfLbPairsResponse>(
            self.0.code_hash.clone(),
            self.0.address.to_string(),
            lb_factory::QueryMsg::GetNumberOfLbPairs {},
        )
        .await
        .map(|response| response.lb_pair_number)
    }
    pub async fn get_lb_pair_at_index(&self, index: u32) -> Result<LbPair, Error> {
        chain_query::<LbPairAtIndexResponse>(
            self.0.code_hash.clone(),
            self.0.address.to_string(),
            lb_factory::QueryMsg::GetLbPairAtIndex { index },
        )
        .await
        .map(|response| response.lb_pair)
    }
    pub async fn get_lb_pair_information(
        &self,
        token_x: ContractInfo,
        token_y: ContractInfo,
        bin_step: u16,
    ) -> Result<LbPairInformation, Error> {
        chain_query::<LbPairInformationResponse>(
            self.0.code_hash.clone(),
            self.0.address.to_string(),
            lb_factory::QueryMsg::GetLbPairInformation {
                token_x: token_x.into(),
                token_y: token_y.into(),
                bin_step,
            },
        )
        .await
        .map(|response| response.lb_pair_information)
    }

    pub async fn get_all_lb_pairs(
        &self,
        token_x: TokenType,
        token_y: TokenType,
    ) -> Result<Vec<LbPairInformation>, Error> {
        chain_query::<AllLbPairsResponse>(
            self.0.code_hash.clone(),
            self.0.address.to_string(),
            lb_factory::QueryMsg::GetAllLbPairs { token_x, token_y },
        )
        .await
        .map(|response| response.lb_pairs_available)
    }
}

/// A thin wrapper around `ContractInfo` that provides additional
/// methods to interact with the LB Pair contract.
#[derive(Serialize, Deserialize, Clone)]
pub struct ILbPair(pub ContractInfo);

impl std::ops::Deref for ILbPair {
    type Target = ContractInfo;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[allow(unused)]
impl ILbPair {
    pub async fn get_active_id(&self) -> Result<u32, Error> {
        chain_query::<ActiveIdResponse>(
            self.0.code_hash.clone(),
            self.0.address.to_string(),
            lb_pair::QueryMsg::GetActiveId {},
        )
        .await
        .map(|response| response.active_id)
    }
    pub async fn get_reserves(&self) -> Result<ReservesResponse, Error> {
        chain_query::<ReservesResponse>(
            self.0.code_hash.clone(),
            self.0.address.to_string(),
            lb_pair::QueryMsg::GetReserves {},
        )
        .await
    }
    pub async fn get_bin(&self, id: u32) -> Result<BinResponse, Error> {
        chain_query::<BinResponse>(
            self.0.code_hash.clone(),
            self.0.address.to_string(),
            lb_pair::QueryMsg::GetBin { id },
        )
        .await
    }
    pub async fn get_next_non_empty_bin(&self, swap_for_y: bool, id: u32) -> Result<u32, Error> {
        chain_query::<NextNonEmptyBinResponse>(
            self.0.code_hash.clone(),
            self.0.address.to_string(),
            lb_pair::QueryMsg::GetNextNonEmptyBin { swap_for_y, id },
        )
        .await
        .map(|response| response.next_id)
    }
}

/// A thin wrapper around `ContractInfo` that provides additional
/// methods to interact with the LB Quoter contract.
#[derive(Serialize, Deserialize, Clone)]
pub struct ILbQuoter(pub ContractInfo);

impl std::ops::Deref for ILbQuoter {
    type Target = ContractInfo;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[allow(unused)]
impl ILbQuoter {
    pub async fn find_best_path_from_amount_in(
        &self,
        route: Vec<TokenType>,
        amount_in: Uint128,
    ) -> Result<Quote, Error> {
        chain_query::<Quote>(
            self.0.code_hash.clone(),
            self.0.address.to_string(),
            lb_quoter::QueryMsg::FindBestPathFromAmountIn { route, amount_in },
        )
        .await
    }
}
