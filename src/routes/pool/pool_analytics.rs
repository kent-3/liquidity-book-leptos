use crate::{
    error::Error,
    prelude::*,
    state::*,
    support::{chain_query, ILbPair, Querier, COMPUTE_QUERIER},
};
use ammber_sdk::contract_interfaces::lb_pair::{self, BinResponse, LbPair};
use batch_query::{
    msg_batch_query, parse_batch_query, BatchItemResponseStatus, BatchQuery, BatchQueryParams,
    BatchQueryParsedResponse, BatchQueryResponse, BATCH_QUERY_ROUTER,
};
use cosmwasm_std::{Addr, ContractInfo};
use leptos::prelude::*;
use leptos_router::{
    components::A,
    hooks::{
        query_signal_with_options, use_location, use_navigate, use_params, use_params_map,
        use_query_map,
    },
    nested_router::Outlet,
    NavigateOptions,
};
use lucide_leptos::{ArrowLeft, ExternalLink, Plus};
use secret_toolkit_snip20::TokenInfoResponse;
use send_wrapper::SendWrapper;
use serde::Serialize;
use tracing::{debug, info, trace};

#[component]
pub fn PoolAnalytics() -> impl IntoView {
    info!("rendering <PoolAnalytics/>");

    let navigate = use_navigate();
    let location = use_location();

    let endpoint = use_context::<Endpoint>().expect("endpoint context missing!");
    let keplr = use_context::<KeplrSignals>().expect("keplr signals context missing!");
    let token_map = use_context::<TokenMap>().expect("tokens context missing!");

    // TODO: I should provide a context here with all the pool information. that way child
    // components like the Add/Remove liquidity ones can access it. I don't think putting the
    // active_id as a query param in the url is a good idea (it should be updated frequently).

    let params = use_params_map();
    // TODO: decide on calling these a/b or x/y
    let token_a = move || {
        params
            .read()
            .get("token_a")
            .expect("Missing token_a URL param")
    };
    let token_b = move || {
        params
            .read()
            .get("token_b")
            .expect("Missing token_b URL param")
    };
    let basis_points = move || {
        params
            .read()
            .get("bps")
            .and_then(|value| value.parse::<u16>().ok())
            .expect("Missing bps URL param")
    };

    view! { hello }
}
