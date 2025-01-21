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

mod add_liquidity;
mod remove_liquidity;

pub use add_liquidity::AddLiquidity;
pub use remove_liquidity::RemoveLiquidity;

// #[derive(Clone)]
// pub struct MyData {
//     x: f64,
//     y1: f64,
//     y2: f64,
// }
//
// impl MyData {
//     fn new(x: f64, y1: f64, y2: f64) -> Self {
//         Self { x, y1, y2 }
//     }
// }
//
// pub fn load_data() -> Signal<Vec<MyData>> {
//     Signal::derive(|| {
//         vec![
//             MyData::new(10.0, 1.5, 0.0),
//             MyData::new(11.0, 1.5, 0.0),
//             MyData::new(12.0, 1.5, 0.0),
//             MyData::new(13.0, 1.5, 0.0),
//             MyData::new(14.0, 1.5, 0.0),
//             MyData::new(15.0, 1.0, 0.0),
//             MyData::new(16.0, 1.0, 0.0),
//             MyData::new(17.0, 1.0, 0.0),
//             MyData::new(18.0, 1.0, 0.0),
//             MyData::new(19.0, 1.0, 0.0),
//             MyData::new(20.0, 1.0, 0.0),
//             MyData::new(21.0, 1.0, 0.0),
//             MyData::new(22.0, 1.0, 0.0),
//             MyData::new(23.0, 1.0, 0.0),
//             MyData::new(24.0, 1.0, 0.0),
//             MyData::new(25.0, 0.5, 0.0),
//             MyData::new(26.0, 0.5, 0.0),
//             MyData::new(27.0, 0.5, 0.0),
//             MyData::new(28.0, 0.5, 0.0),
//             MyData::new(29.0, 0.5, 0.0),
//             MyData::new(30.0, 0.5, 0.0),
//             MyData::new(31.0, 0.5, 0.0),
//             MyData::new(32.0, 0.5, 0.0),
//             MyData::new(33.0, 0.5, 0.0),
//             MyData::new(34.0, 0.5, 0.0),
//             MyData::new(35.0, 0.1, 0.0),
//             MyData::new(36.0, 0.1, 0.0),
//             MyData::new(37.0, 0.1, 0.0),
//             MyData::new(38.0, 0.1, 0.0),
//             MyData::new(39.0, 0.1, 0.0),
//             MyData::new(40.0, 0.5, 0.0),
//             MyData::new(41.0, 0.5, 0.0),
//             MyData::new(42.0, 0.5, 0.0),
//             MyData::new(43.0, 0.5, 0.0),
//             MyData::new(44.0, 0.5, 0.0),
//             MyData::new(45.0, 1.0, 0.0),
//             MyData::new(46.0, 2.0, 0.0),
//             MyData::new(47.0, 3.0, 0.0),
//             MyData::new(48.0, 4.0, 0.0),
//             MyData::new(49.0, 5.0, 0.0),
//             MyData::new(50.0, 3.0, 3.0),
//             MyData::new(51.0, 0.0, 5.0),
//             MyData::new(52.0, 0.0, 4.0),
//             MyData::new(53.0, 0.0, 3.0),
//             MyData::new(54.0, 0.0, 2.0),
//             MyData::new(55.0, 0.0, 1.0),
//             MyData::new(56.0, 0.0, 0.5),
//             MyData::new(57.0, 0.0, 0.5),
//             MyData::new(58.0, 0.0, 0.5),
//             MyData::new(59.0, 0.0, 0.5),
//             MyData::new(60.0, 0.0, 0.5),
//             MyData::new(61.0, 0.0, 0.1),
//             MyData::new(62.0, 0.0, 0.1),
//             MyData::new(63.0, 0.0, 0.1),
//             MyData::new(64.0, 0.0, 0.1),
//             MyData::new(65.0, 0.0, 0.1),
//             MyData::new(66.0, 0.0, 0.0),
//             MyData::new(67.0, 0.0, 0.0),
//             MyData::new(68.0, 0.0, 0.0),
//             MyData::new(69.0, 0.0, 0.0),
//             MyData::new(70.0, 0.0, 3.0),
//             MyData::new(71.0, 0.0, 3.0),
//             MyData::new(72.0, 0.0, 3.0),
//             MyData::new(73.0, 0.0, 3.0),
//             MyData::new(74.0, 0.0, 3.0),
//             MyData::new(75.0, 0.0, 3.0),
//             MyData::new(76.0, 0.0, 3.0),
//             MyData::new(77.0, 0.0, 3.0),
//             MyData::new(78.0, 0.0, 3.0),
//             MyData::new(79.0, 0.0, 3.0),
//             MyData::new(80.0, 0.0, 3.0),
//             MyData::new(81.0, 0.0, 3.1),
//             MyData::new(82.0, 0.0, 3.1),
//             MyData::new(83.0, 0.0, 3.1),
//             MyData::new(84.0, 0.0, 3.1),
//             MyData::new(85.0, 0.0, 3.1),
//             MyData::new(86.0, 0.0, 3.1),
//             MyData::new(87.0, 0.0, 3.1),
//             MyData::new(88.0, 0.0, 3.1),
//             MyData::new(89.0, 0.0, 3.1),
//             MyData::new(90.0, 0.0, 3.1),
//         ]
//     })
// }
//
// use leptos_chartistry::*;
//
// #[component]
// pub fn Example(debug: Signal<bool>, data: Signal<Vec<MyData>>) -> impl IntoView {
//     let series = Series::new(|data: &MyData| data.x)
//         .with_min_y(0.00)
//         // .with_x_range(Some(40.0), Some(60.0))
//         .with_colours([
//             Colour::from_rgb(246, 193, 119),
//             Colour::from_rgb(49, 116, 143),
//         ])
//         .bar(
//             Bar::new(|data: &MyData| data.y1).with_name("Token Y"), // .with_group_gap(0.05)
//                                                                     // .with_gap(0.1),
//         )
//         .bar(
//             Bar::new(|data: &MyData| data.y2).with_name("Token X"), // .with_group_gap(0.05)
//                                                                     // .with_gap(0.1),
//         );
//
//     view! {
//         <Chart
//             aspect_ratio=AspectRatio::from_outer_height(330.0, 1.73)
//             debug=debug
//             series=series
//             data=data
//             font_width=14.0
//             font_height=14.0
//
//             // left=TickLabels::aligned_floats()
//             inner=[
//                 // AxisMarker::bottom_edge().with_arrow(false).into_inner(),
//             ]
//             bottom=TickLabels::aligned_floats()
//             tooltip=Tooltip::left_cursor()
//         />
//     }
// }

#[component]
pub fn PoolManager() -> impl IntoView {
    info!("rendering <PoolManager/>");

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

    // // SendWrapper required due to addr_2_contract function
    // let lb_pair: Resource<LbPair> = Resource::new(
    //     move || (token_a(), token_b(), basis_points()),
    //     move |(token_a, token_b, basis_points)| {
    //         SendWrapper::new(async move {
    //             let token_x = addr_2_contract(token_a).await.unwrap();
    //             let token_y = addr_2_contract(token_b).await.unwrap();
    //             let bin_step = basis_points;
    //
    //             LB_FACTORY
    //                 .get_lb_pair_information(token_x, token_y, bin_step)
    //                 .await
    //                 .map(|lb_pair_information| lb_pair_information.lb_pair)
    //                 .unwrap()
    //         })
    //     },
    // );

    let lb_pair = use_context::<Resource<LbPair>>().expect("missing the LbPair resource context");
    let active_id = use_context::<Resource<Result<u32, Error>>>()
        .expect("missing the active_id resource context");

    let nav_options = NavigateOptions {
        // prevents scrolling to the top of the page each time a query param changes
        scroll: false,
        ..Default::default()
    };

    // // This component only needs to write to the signal
    // let (_, set_active_id) = query_signal_with_options::<String>("active_id", nav_options.clone());
    //
    // let active_id = Resource::new(
    //     move || lb_pair.track(),
    //     move |_| {
    //         async move {
    //             ILbPair(lb_pair.await.contract)
    //                 .get_active_id()
    //                 .await
    //                 // This will set a URL query param "active_id" for nested routes to use
    //                 .inspect(|id| set_active_id.set(Some(id.to_string())))
    //         }
    //     },
    // );

    // NOTE: We have a lot of Resources depending on other Resources.
    //       It works, but I wonder if there is a better way.

    // TODO: I don't think there's any need to track signals. These should all be
    //       LocalResources that only run once on page load.

    // let total_reserves = Resource::new(
    //     move || (),
    //     move |_| async move { ILbPair(lb_pair.await.contract).get_reserves().await },
    // );
    //
    // let bin_reserves = Resource::new(
    //     move || (),
    //     move |_| async move {
    //         let lb_pair = ILbPair(lb_pair.await.contract);
    //         let id = active_id.await?;
    //
    //         lb_pair.get_bin(id).await
    //     },
    // );
    //
    // let nearby_bins = LocalResource::new(move || {
    //     async move {
    //         let lb_pair_contract = lb_pair.await.contract;
    //         let id = active_id.await?;
    //         let mut batch = Vec::new();
    //
    //         let radius = 49;
    //
    //         for i in 0..(radius * 2 + 1) {
    //             let offset_id = if i < radius {
    //                 id - (radius - i) as u32 // Subtract for the first half
    //             } else {
    //                 id + (i - radius) as u32 // Add for the second half
    //             };
    //
    //             batch.push(BatchQueryParams {
    //                 id: offset_id.to_string(),
    //                 contract: lb_pair_contract.clone(),
    //                 query_msg: lb_pair::QueryMsg::GetBin { id: offset_id },
    //             });
    //         }
    //
    //         query_nearby_bins(batch).await
    //     }
    // });

    use ammber_charts::{load_data, LiquidityChart};
    let debug = RwSignal::new(false);
    let my_data = load_data();

    view! {
        <div class="grid auto-rows-min grid-cols-1 sm:grid-cols-2 gap-8">

            // left side of the screen
            <div class="flex flex-col items-center gap-6">
                // my liquidity box
                <div class="block w-full outline outline-2 outline-neutral-700 rounded">
                    <div class="px-6 py-4">
                        <div class="w-full">
                            <h2 class="m-0 mb-2 text-xl">My Liquidity</h2>
                            // <LiquidityChart debug=debug.into() data=my_data.into() />
                            // <p class="text-neutral-500">"You have no liquidity in this pool"</p>
                            <div class="flex justify-center items-center h-48"></div>
                        </div>
                    </div>
                    <hr class="m-0 border-2 border-neutral-700" />
                    <div class="px-6 py-4">
                        <h2 class="m-0 mb-2 text-xl">Deposit Balance</h2>
                        <div class="flex flex-col gap-2 items-center">
                            <div class="grid grid-cols-1 gap-4 w-full">
                                <div class="grid grid-cols-[1fr_14px_1fr] gap-4 w-full items-center">
                                    // token x deposit balance
                                    <div class="flex items-center box-border px-4 py-3 h-16 bg-neutral-800 rounded">
                                        <div class="flex items-center flex-row flex-1 gap-2">
                                            // <img class="w-8 h-8 rounded-full" src="" / >
                                            <div class="flex flex-col items-start gap-0">
                                                <p class="m-0 text-sm box-content">
                                                    <b class="text-white">0</b>
                                                    " "
                                                    {move || token_a_symbol.get()}
                                                </p>
                                                <p class="m-0 text-sm">"$0"</p>
                                            </div>
                                        </div>
                                    </div>
                                    <Plus size=14 color="white" />
                                    // token y deposit balance
                                    <div class="flex items-center box-border px-4 py-3 h-16 bg-neutral-800 rounded">
                                        <div class="flex items-center flex-row flex-1 gap-2">
                                            // <img class="w-8 h-8 rounded-full" src="" / >
                                            <div class="flex flex-col items-start gap-0">
                                                <p class="m-0 text-sm">
                                                    <b class="text-white">0</b>
                                                    " "
                                                    {move || token_b_symbol.get()}
                                                </p>
                                                <p class="m-0 text-sm">"$0"</p>
                                            </div>
                                        </div>
                                    </div>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
                <div class="block w-full outline outline-2 outline-neutral-700 rounded">
                    <div class="px-6 py-4">
                        <div class="w-full">
                            <h2 class="m-0 mb-2 text-xl">"Fees Earned"</h2>
                            <div class="flex justify-center items-center h-48">
                                <p class="text-neutral-500">"You have no fees earned"</p>
                            </div>
                        </div>
                    </div>
                </div>

            </div>

            // <details class="text-neutral-300 font-bold">
            // <summary class="text-lg cursor-pointer">Pool Details</summary>
            // <ul class="my-1 font-normal text-base text-neutral-200 ">
            // <Suspense fallback=|| view! { <div>"Loading Total Reserves..."</div> }>
            // <li>
            // "Total Reserves: "
            // <span tabindex="0" class="cursor-pointer text-white peer">
            // "🛈"
            // </span>
            // <li class="list-none text-sm font-bold text-violet-400 peer-focus:block hidden">
            // "🛈 Reserves may be in reverse order"
            // </li>
            // {move || Suspend::new(async move {
            // let reserves = total_reserves.await.unwrap();
            // view! {
            // <li class="pl-4 list-none">
            // "reserve_x: "{reserves.reserve_x.to_string()}
            // </li>
            // <li class="pl-4 list-none">
            // "reserve_y: "{reserves.reserve_y.to_string()}
            // </li>
            // }
            // })}
            // </li>
            // </Suspense>
            // <Suspense fallback=|| view! { <div>"Loading Active ID..."</div> }>
            // <li>
            // "Active Bin ID: " {move || Suspend::new(async move { active_id.await })}
            // </li>
            // </Suspense>
            // <Suspense fallback=|| view! { <div>"Loading Bin Reserves..."</div> }>
            // <li>
            // "Active Bin Reserves: "
            // {move || Suspend::new(async move {
            // let reserves = bin_reserves.await.unwrap();
            // view! {
            // <li class="pl-4 list-none">
            // "bin_reserve_x: "{reserves.bin_reserve_x.to_string()}
            // </li>
            // <li class="pl-4 list-none">
            // "bin_reserve_y: "{reserves.bin_reserve_y.to_string()}
            // </li>
            // }
            // })}
            // </li>
            // </Suspense>
            // // a bit crazy innit. but it works.
            // <Suspense fallback=|| view! { <div>"Loading Nearby Bins..."</div> }>
            // <li>
            // "Nearby Bins: "
            // {move || Suspend::new(async move {
            // nearby_bins
            // .await
            // .and_then(|bins| {
            // Ok(
            // bins
            // .into_iter()
            // .map(|bin| {
            // view! {
            // <li class="pl-4 list-none">
            // {bin.bin_id} " " {bin.bin_reserve_x.to_string()} " "
            // {bin.bin_reserve_y.to_string()}
            // </li>
            // }
            // })
            // .collect::<Vec<_>>(),
            // )
            // })
            // })}
            // </li>
            // </Suspense>
            // // <SecretQuery query=bin_total_supply />
            // </ul>
            // </details>

            // right side of screen
            <div class="block px-5 py-4 w-full box-border space-y-5 mx-auto outline outline-2 outline-neutral-700 rounded max-h-max">
                // "Tab Group"
                <div class="flex gap-4 w-full">
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
                </div>
                // TODO: I think add/remove liquidity should not be separate routes, and instead toggle
                // visibility with a tab-group-like thing
                <div class="liquidity-group">
                    <Outlet />
                </div>
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
