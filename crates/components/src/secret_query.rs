use crate::{
    constants::{CHAIN_ID, NODE, TOKEN_MAP},
    error::Error,
    state::{ChainId, Endpoint, KeplrSignals, TokenMap},
};
use keplr::Keplr;
use leptos::either::Either;
use leptos::prelude::*;
use rsecret::query::compute::ComputeQuerier;
use secretrs::proto::secret::compute::v1beta1::QuerySecretContractRequest;
use send_wrapper::SendWrapper;
use serde::{Deserialize, Serialize};
use tonic_web_wasm_client::Client as WebWasmClient;
use tracing::{debug, trace};
use web_sys::MouseEvent;

#[component]
pub fn SecretQuery(query: Resource<Result<String, Error>>) -> impl IntoView {
    // Potentially create the Resource internally
    // let endpoint = use_context::<Endpoint>().expect("endpoint context missing!");
    // let chain_id = use_context::<ChainId>().expect("chain_id context missing!");
    // let query_result: Resource<Result<String, Error>> = Resource::new(
    //     move || (endpoint.get(), chain_id.get()),
    //     move |(endpoint, chain_id)| SendWrapper::new({ async move { todo!() } }),
    // );

    view! {
        <div class="secret-contract-query">
            <Suspense fallback=|| {
                view! { <div class="py-0 px-2 text-sm">"Querying..."</div> }
            }>
                {move || Suspend::new(async move {
                    match query.await {
                        Ok(response) => {
                            Either::Left(
                                view! {
                                    <div class="py-0 px-2 text-ellipsis text-sm">{response}</div>
                                },
                            )
                        }
                        Err(error) => {
                            Either::Right(
                                view! {
                                    <div class="py-0 px-2 text-violet-400 text-bold text-sm">
                                        {error.to_string()}
                                    </div>
                                },
                            )
                        }
                    }
                })}
            </Suspense>
        </div>
    }
}

async fn do_query<T: Serialize + Send + Sync>(
    endpoint: String,
    contract_address: String,
    query: T,
) -> Result<String, Error> {
    let compute = ComputeQuerier::new(
        WebWasmClient::new(endpoint),
        Keplr::get_enigma_utils(CHAIN_ID).into(),
    );

    // TODO: make rsecret do this part internally?
    let code_hash = compute
        .code_hash_by_contract_address(&contract_address)
        .await?;

    let response = compute
        .query_secret_contract(&contract_address, code_hash, query)
        .await?;

    debug!("response: {response}");

    Ok(response)
}
