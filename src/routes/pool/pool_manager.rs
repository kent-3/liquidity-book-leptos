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
use secret_toolkit_snip20::TokenInfoResponse;
use send_wrapper::SendWrapper;
use serde::Serialize;
use tracing::{debug, info, trace};

mod add_liquidity;
mod remove_liquidity;

pub use add_liquidity::AddLiquidity;
pub use remove_liquidity::RemoveLiquidity;

#[component]
pub fn PoolManager() -> impl IntoView {
    info!("rendering <PoolManager/>");

    let navigate = use_navigate();
    let location = use_location();

    let endpoint = use_context::<Endpoint>().expect("endpoint context missing!");
    let keplr = use_context::<KeplrSignals>().expect("keplr signals context missing!");
    let token_map = use_context::<TokenMap>().expect("tokens context missing!");

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

    // TODO: these 2 functions feel very convoluted

    async fn addr_2_contract(contract_address: impl Into<String>) -> Result<ContractInfo, Error> {
        let contract_address = contract_address.into();

        if let Some(token) = TOKEN_MAP.get(&contract_address) {
            Ok(ContractInfo {
                address: Addr::unchecked(token.contract_address.clone()),
                code_hash: token.code_hash.clone(),
            })
        } else {
            COMPUTE_QUERIER
                .code_hash_by_contract_address(&contract_address)
                .await
                .map_err(Error::from)
                .map(|code_hash| ContractInfo {
                    address: Addr::unchecked(contract_address),
                    code_hash,
                })
        }
    }

    async fn token_symbol_convert(address: String) -> String {
        if let Some(token) = TOKEN_MAP.get(&address) {
            return token.symbol.clone();
        }
        let contract = addr_2_contract(&address).await.unwrap();

        chain_query::<TokenInfoResponse>(
            contract.address.to_string(),
            contract.code_hash,
            secret_toolkit_snip20::QueryMsg::TokenInfo {},
        )
        .await
        .map(|x| x.token_info.symbol)
        .unwrap_or(address)
    }

    let token_a_symbol =
        AsyncDerived::new_unsync(move || async move { token_symbol_convert(token_a()).await });

    let token_b_symbol =
        AsyncDerived::new_unsync(move || async move { token_symbol_convert(token_b()).await });

    // TODO: how about instead, we have a contract query that can return the nearby liquidity, so
    // we don't have to mess with the complicated batch query router? That might be the purpose of
    // the LiquidityHelper contract (I haven't looked at it yet)

    async fn query_nearby_bins<T: Serialize>(
        queries: Vec<BatchQueryParams<T>>,
    ) -> Result<Vec<BinResponse>, Error> {
        msg_batch_query(queries)
            .do_query(&BATCH_QUERY_ROUTER.pulsar)
            .await
            .inspect(|response| trace!("{:?}", response))
            .and_then(|response| Ok(serde_json::from_str::<BatchQueryResponse>(&response)?))
            .map(parse_batch_query)
            .map(extract_bins_from_batch_response)
    }

    fn extract_bins_from_batch_response(
        batch_response: BatchQueryParsedResponse,
    ) -> Vec<BinResponse> {
        batch_response
            .items
            .into_iter()
            .filter(|item| item.status == BatchItemResponseStatus::SUCCESS)
            .map(|item| {
                serde_json::from_str::<BinResponse>(&item.response)
                    .expect("Invalid BinResponse JSON")
            })
            .collect()
    }

    // SendWrapper required due to addr_2_contract function
    let lb_pair: Resource<LbPair> = Resource::new(
        move || (token_a(), token_b(), basis_points()),
        move |(token_a, token_b, basis_points)| {
            SendWrapper::new(async move {
                let token_x = addr_2_contract(token_a).await.unwrap();
                let token_y = addr_2_contract(token_b).await.unwrap();
                let bin_step = basis_points;

                LB_FACTORY
                    .get_lb_pair_information(token_x, token_y, bin_step)
                    .await
                    .map(|lb_pair_information| lb_pair_information.lb_pair)
                    .unwrap()
            })
        },
    );

    let nav_options = NavigateOptions {
        // prevents scrolling to the top of the page each time a query param changes
        scroll: false,
        ..Default::default()
    };

    // This component only needs to write to the signal
    let (_, set_active_id) = query_signal_with_options::<String>("active_id", nav_options.clone());

    let active_id = Resource::new(
        move || lb_pair.track(),
        move |_| {
            async move {
                ILbPair(lb_pair.await.contract)
                    .get_active_id()
                    .await
                    // This will set a URL query param "active_id" for nested routes to use
                    .inspect(|id| set_active_id.set(Some(id.to_string())))
            }
        },
    );

    // NOTE: We have a lot of Resources depending on other Resources.
    //       It works, but I wonder if there is a better way.

    let total_reserves = Resource::new(
        move || lb_pair.track(),
        move |_| async move { ILbPair(lb_pair.await.contract).get_reserves().await },
    );

    let bin_reserves = Resource::new(
        move || (lb_pair.track(), active_id.track()),
        move |_| async move {
            let lb_pair = ILbPair(lb_pair.await.contract);
            let id = active_id.await?;

            lb_pair.get_bin(id).await
        },
    );

    let nearby_bins = Resource::new(
        move || (lb_pair.track(), active_id.track()),
        move |_| {
            SendWrapper::new(async move {
                let lb_pair_contract = lb_pair.await.contract;
                let id = active_id.await?;
                let mut batch = Vec::new();

                let radius = 10;

                for i in 0..(radius * 2 + 1) {
                    let offset_id = if i < radius {
                        id - (radius - i) as u32 // Subtract for the first half
                    } else {
                        id + (i - radius) as u32 // Add for the second half
                    };

                    batch.push(BatchQueryParams {
                        id: offset_id.to_string(),
                        contract: lb_pair_contract.clone(),
                        query_msg: lb_pair::QueryMsg::GetBin { id: offset_id },
                    });
                }

                query_nearby_bins(batch).await
            })
        },
    );

    view! {
        <a
            href="/liquidity-book-leptos/pool"
            class="block text-neutral-200/50 text-sm font-bold cursor-pointer no-underline"
        >
            "🡨 Back to pools list"
        </a>

        <div class="flex flex-wrap py-2 items-center gap-x-4 gap-y-2">
            <Suspense fallback=move || {
                view! { <div class="text-3xl font-bold">{token_a}" / "{token_b}</div> }
            }>
                <div class="text-3xl font-bold">
                    {move || Suspend::new(async move { token_a_symbol.await })}" / "
                    {move || Suspend::new(async move { token_b_symbol.await })}
                </div>
            </Suspense>

            <div class="flex items-center gap-x-4">
                <div class="text-md font-bold p-1 outline outline-1 outline-offset-2 outline-neutral-500/50">
                    {basis_points}" bps"
                </div>
                <a href="about:blank" target="_blank" rel="noopener">
                    <div class="text-md font-bold p-1 outline outline-1 outline-offset-2 outline-neutral-500/50">
                        {move || { lb_pair.get().map(|x| shorten_address(x.contract.address)) }}
                        " ↗"
                    </div>
                </a>
            </div>
        </div>

        <div class="grid auto-rows-min grid-cols-1 sm:grid-cols-2 gap-8">

            <details class="text-neutral-300 font-bold">
                <summary class="text-lg cursor-pointer">Pool Details</summary>
                <ul class="my-1 font-normal text-base text-neutral-200 ">
                    <Suspense fallback=|| view! { <div>"Loading Total Reserves..."</div> }>
                        <li>
                            "Total Reserves: "
                            <span tabindex="0" class="cursor-pointer text-white peer">
                                "🛈"
                            </span>
                            <li class="list-none text-sm font-bold text-violet-400 peer-focus:block hidden">
                                "🛈 Reserves may be in reverse order"
                            </li>
                            {move || Suspend::new(async move {
                                let reserves = total_reserves.await.unwrap();
                                view! {
                                    <li class="pl-4 list-none">
                                        "reserve_x: "{reserves.reserve_x.to_string()}
                                    </li>
                                    <li class="pl-4 list-none">
                                        "reserve_y: "{reserves.reserve_y.to_string()}
                                    </li>
                                }
                            })}
                        </li>
                    </Suspense>
                    <Suspense fallback=|| view! { <div>"Loading Active ID..."</div> }>
                        <li>
                            "Active Bin ID: " {move || Suspend::new(async move { active_id.await })}
                        </li>
                    </Suspense>
                    <Suspense fallback=|| view! { <div>"Loading Bin Reserves..."</div> }>
                        <li>
                            "Active Bin Reserves: "
                            {move || Suspend::new(async move {
                                let reserves = bin_reserves.await.unwrap();
                                view! {
                                    <li class="pl-4 list-none">
                                        "bin_reserve_x: "{reserves.bin_reserve_x.to_string()}
                                    </li>
                                    <li class="pl-4 list-none">
                                        "bin_reserve_y: "{reserves.bin_reserve_y.to_string()}
                                    </li>
                                }
                            })}
                        </li>
                    </Suspense>
                    // a bit crazy innit. but it works.
                    <Suspense fallback=|| view! { <div>"Loading Nearby Bins..."</div> }>
                        <li>
                            "Nearby Bins: "
                            {move || Suspend::new(async move {
                                nearby_bins
                                    .await
                                    .and_then(|bins| {
                                        Ok(
                                            bins
                                                .into_iter()
                                                .map(|bin| {
                                                    view! {
                                                        <li class="pl-4 list-none">
                                                            {bin.bin_id} " " {bin.bin_reserve_x.to_string()} " "
                                                            {bin.bin_reserve_y.to_string()}
                                                        </li>
                                                    }
                                                })
                                                .collect::<Vec<_>>(),
                                        )
                                    })
                            })}
                        </li>
                    </Suspense>
                // <SecretQuery query=bin_total_supply />
                </ul>
            </details>

            <div class="block px-5 py-4 max-w-md w-full box-border space-y-5 mx-auto outline outline-2 outline-neutral-700 rounded max-h-max">

                <div class="flex gap-4 w-full max-w-md">
                    // This preserves the query params when navigating to nested routes.
                    // TODO: this is terribly complicated. it works, but there must be a simpler way
                    <button
                        class="w-full"
                        on:click={
                            let navigate_ = navigate.clone();
                            move |_| {
                                let mut pathname = location.pathname.get();
                                let query_params = location.search.get();
                                if pathname.ends_with('/') {
                                    pathname.pop();
                                }
                                if pathname.ends_with("/add") || pathname.ends_with("/remove") {
                                    pathname = pathname
                                        .rsplit_once('/')
                                        .map(|(base, _)| base)
                                        .unwrap_or(&pathname)
                                        .to_string();
                                }
                                let new_route = format!("{pathname}/add/?{query_params}");
                                navigate_(&new_route, Default::default());
                            }
                        }
                    >
                        "Add Liquidity"
                    </button>

                    <button
                        class="w-full"
                        on:click={
                            let navigate_ = navigate.clone();
                            move |_| {
                                let mut pathname = location.pathname.get();
                                let query_params = location.search.get();
                                if pathname.ends_with('/') {
                                    pathname.pop();
                                }
                                if pathname.ends_with("/add") || pathname.ends_with("/remove") {
                                    pathname = pathname
                                        .rsplit_once('/')
                                        .map(|(base, _)| base)
                                        .unwrap_or(&pathname)
                                        .to_string();
                                }
                                let new_route = format!("{pathname}/remove/?{query_params}");
                                navigate_(&new_route, Default::default());
                            }
                        }
                    >
                        "Remove Liquidity"
                    </button>

                // <button on:click={
                // let navigate_ = navigate.clone();
                // move |_| {
                // let mut pathname = location.pathname.get();
                // let query_params = location.search.get();
                // if pathname.ends_with('/') {
                // pathname.pop();
                // }
                // if pathname.ends_with("/add") || pathname.ends_with("/remove") {
                // pathname = pathname
                // .rsplit_once('/')
                // .map(|(base, _)| base)
                // .unwrap_or(&pathname)
                // .to_string();
                // }
                // let new_route = format!("{pathname}/?{query_params}");
                // navigate_(&new_route, Default::default());
                // }
                // }>"Nevermind"</button>
                </div>

                <Outlet />

            </div>

        </div>
    }
}

// <Suspense fallback=|| view! { <p>"Loading..."</p> }>
// // you can `.await` resources to avoid dealing with the `None` state
// <p>
// "User ID: "
// {move || Suspend::new(async move {
// match resource.await {
// Ok(response) => response,
// Err(_) => "error".to_string(),
// }
// })}
// </p>
// or you can still use .get() to access resources in things like component props
// <For
// each=move || resource.get().and_then(Result::ok).unwrap_or_default()
// key=|resource| resource.id
// let:post
// >
// // ...
// </For>
// </Suspense>
