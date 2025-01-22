use crate::chain_query;
use crate::error::Error;
use ammber_sdk::contract_interfaces::lb_pair::{
    self, BinResponse, LbPair, ReservesResponse, StaticFeeParametersResponse,
};
use batch_query::{
    msg_batch_query, parse_batch_query, BatchItemResponseStatus, BatchQuery, BatchQueryParams,
    BatchQueryParsedResponse, BatchQueryResponse, BATCH_QUERY_ROUTER,
};
use leptos::prelude::*;
use leptos_use::use_clipboard;
use liquidity_book::interfaces::lb_pair::BinsResponse;
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

#[component]
pub fn PoolAnalytics() -> impl IntoView {
    info!("rendering <PoolAnalytics/>");

    use leptos_use::UseClipboardReturn;
    use lucide_leptos::{Copy, Link};

    let UseClipboardReturn { copy, .. } = use_clipboard();

    let (token_x_symbol, token_y_symbol) = use_context::<(
        AsyncDerived<String, LocalStorage>,
        AsyncDerived<String, LocalStorage>,
    )>()
    .expect("missing token symbols context");
    let lb_pair = use_context::<Resource<Result<LbPair, Error>>>()
        .expect("missing the LbPair resource context");
    let active_id = use_context::<Resource<Result<u32, Error>>>()
        .expect("missing the active_id resource context");
    let static_fee_parameters =
        use_context::<Resource<Result<StaticFeeParametersResponse, Error>>>()
            .expect("missing the static fee parameters resource context");

    let bin_step = move || {
        lb_pair
            .get_untracked()
            .and_then(Result::ok)
            .map(|pair| pair.bin_step)
            .unwrap_or_default()
    };
    let pool_address = move || {
        lb_pair
            .get_untracked()
            .and_then(Result::ok)
            .map(|pair| pair.contract.address.to_string())
            .unwrap_or_default()
    };
    let token_x_address = move || {
        lb_pair
            .get_untracked()
            .and_then(Result::ok)
            .map(|pair| pair.token_x.address().to_string())
            .unwrap_or_default()
    };
    let token_y_address = move || {
        lb_pair
            .get_untracked()
            .and_then(Result::ok)
            .map(|pair| pair.token_y.address().to_string())
            .unwrap_or_default()
    };

    let base_fee = AsyncDerived::new(move || async move {
        let base_factor = static_fee_parameters
            .await
            .map(|r| r.base_factor as u128)
            .unwrap_or_default();

        let fee_bps = base_factor * bin_step() as u128 / 10_000;
        format!("{}.{}%", fee_bps / 100, fee_bps % 100)
    });

    // TODO: this seems right, but double check the maths
    let max_fee = AsyncDerived::new(move || async move {
        let static_fee_parameters = static_fee_parameters.await.unwrap();
        let base_factor = static_fee_parameters.base_factor as u128;
        let variable_fee_control = static_fee_parameters.variable_fee_control as u128;
        let max_volatility_accumulator = static_fee_parameters.max_volatility_accumulator as u128;

        let base_fee = base_factor * bin_step() as u128 * 10_000_000_000;

        debug!("{base_fee}");

        let prod = (max_volatility_accumulator as u128) * (100 as u128);
        let max_variable_fee = (prod * prod * variable_fee_control + 99) / 100;

        debug!("{max_variable_fee}");

        let max_fee = base_fee + max_variable_fee;
        let max_fee_bps = max_fee / 100_000_000_000_000;
        format!("{}.{}%", max_fee_bps / 100, max_fee_bps % 100)
    });

    // TODO: test this when the protocol_share isn't set to 0 lol
    let protocol_fee = AsyncDerived::new(move || async move {
        let protocol_fee = static_fee_parameters
            .await
            .map(|r| r.protocol_share)
            .unwrap_or_default();

        format!("{}.{}%", protocol_fee / 100, protocol_fee % 100)
    });

    let total_reserves = use_context::<Resource<Result<ReservesResponse, Error>>>()
        .expect("missing the total reserves resource context");

    let reserve_x = AsyncDerived::new(move || async move {
        let amount = total_reserves
            .await
            .map(|r| r.reserve_x.to_string())
            .unwrap_or(0.to_string());
        let denom = token_x_symbol.await;

        format!("{} {}", amount, denom)
    });
    let reserve_y = AsyncDerived::new(move || async move {
        let amount = total_reserves
            .await
            .map(|r| r.reserve_y.to_string())
            .unwrap_or(0.to_string());
        let denom = token_y_symbol.await;

        format!("{} {}", amount, denom)
    });

    // 8.7 kb
    let nearby_bins = LocalResource::new(move || {
        debug!("getting nearby bins");
        async move {
            let lb_pair_contract = lb_pair.await?.contract;
            let id = active_id.await?;
            let mut ids = Vec::new();

            let radius = 50;

            for i in 0..(radius * 2 + 1) {
                let offset_id = if i < radius {
                    id - (radius - i) as u32 // Subtract for the first half
                } else {
                    id + (i - radius) as u32 // Add for the second half
                };

                ids.push(offset_id);
            }

            let bins = chain_query::<BinsResponse>(
                lb_pair_contract.code_hash.clone(),
                lb_pair_contract.address.to_string(),
                lb_pair::QueryMsg::GetBins { ids },
            )
            .await
            .map(|response| response.0);

            bins
        }
    });

    // 38.4 kb
    // let nearby_bins = LocalResource::new(move || {
    //     async move {
    //         let lb_pair_contract = lb_pair.await?.contract;
    //         let id = active_id.await?;
    //         let mut queries = Vec::new();
    //
    //         let radius = 50;
    //
    //         for i in 0..(radius * 2 + 1) {
    //             let offset_id = if i < radius {
    //                 id - (radius - i) as u32 // Subtract for the first half
    //             } else {
    //                 id + (i - radius) as u32 // Add for the second half
    //             };
    //
    //             queries.push(BatchQueryParams {
    //                 id: offset_id.to_string(),
    //                 contract: lb_pair_contract.clone(),
    //                 query_msg: lb_pair::QueryMsg::GetBin { id: offset_id },
    //             });
    //         }
    //
    //         let batch_query_message = msg_batch_query(queries);
    //
    //         let bins = chain_query::<BatchQueryResponse>(
    //             BATCH_QUERY_ROUTER.pulsar.code_hash.clone(),
    //             BATCH_QUERY_ROUTER.pulsar.address.to_string(),
    //             batch_query_message,
    //         )
    //         .await
    //         .map(parse_batch_query)
    //         .map(extract_bins_from_batch);
    //
    //         bins
    //     }
    // });

    fn extract_bins_from_batch(batch_response: BatchQueryParsedResponse) -> Vec<BinResponse> {
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

    #[derive(Clone, Debug)]
    pub struct MyData {
        id: f64,
        x: f64,
        y: f64,
    }

    impl MyData {
        fn new(id: u32, x: u128, y: u128) -> Self {
            Self {
                id: id as f64,
                x: x as f64,
                y: y as f64,
            }
        }
    }

    // FIXME: prevent this from running twice
    let chart_data = AsyncDerived::new(move || async move {
        debug!("gathering chart data");
        let bins = nearby_bins.await.unwrap_or_default();

        let data: Vec<MyData> = bins
            .iter()
            .map(|bin_response| {
                MyData::new(
                    bin_response.bin_id,
                    bin_response.bin_reserve_x.u128(),
                    bin_response.bin_reserve_y.u128(),
                )
            })
            .collect();

        format!("{data:?}")
    });

    view! {
        <div class="flex flex-col gap-4">
            <div class="grid grid-cols-2 sm:grid-cols-3 gap-4">
                <div class="bg-neutral-800 px-4 sm:px-8 py-4 rounded-md">
                    <dl class="m-0">
                        <dt class="text-sm text-neutral-400 font-medium">"Liquidity"</dt>
                        <div class="flex flex-row items-center gap-2">
                            <dd class="m-0 text-2xl font-semibold align-baseline proportional-nums">
                                "$0.00"
                            </dd>
                            <dd class="m-0 text-foam text-sm font-semibold">"0%"</dd>
                        </div>
                    </dl>
                </div>
                <div class="bg-neutral-800 px-4 sm:px-8 py-4 rounded-md">
                    <dl class="m-0">
                        <dt class="text-sm text-neutral-400 font-medium">"Volume (24H)"</dt>
                        <div class="flex flex-row items-center gap-2">
                            <dd class="m-0 text-2xl font-semibold align-baseline proportional-nums">
                                "$0.00"
                            </dd>
                            <dd class="m-0 text-rose text-sm font-semibold">"0%"</dd>
                        </div>
                    </dl>
                </div>
                <div class="bg-neutral-800 px-4 sm:px-8 py-4 rounded-md">
                    <dl class="m-0">
                        <dt class="text-sm text-neutral-400 font-medium">"Fees (24H)"</dt>
                        <div class="flex flex-row items-center gap-2">
                            <dd class="m-0 text-2xl font-semibold align-baseline proportional-nums">
                                "$0.00"
                            </dd>
                            <dd class="m-0 text-rose text-sm font-semibold">"0%"</dd>
                        </div>
                    </dl>
                </div>
                <div class="bg-neutral-800 px-4 sm:px-8 py-4 rounded-md">
                    <dl class="m-0">
                        <dt class="text-sm text-neutral-400 font-medium">"APR (7D)"</dt>
                        <dd class="m-0 text-2xl font-semibold align-baseline proportional-nums">
                            "0.00%"
                        </dd>
                    </dl>
                </div>
                <div class="bg-neutral-800 px-4 sm:px-8 py-4 rounded-md">
                    <dl class="m-0">
                        <dt class="text-sm text-neutral-400 font-medium">
                            {move || token_x_symbol.get()} " Reserves"
                        </dt>
                        <dd class="m-0 text-2xl font-semibold align-baseline proportional-nums">
                            <Suspense fallback=|| {
                                view! { "Loading..." }
                            }>{move || Suspend::new(async move { reserve_x.await })}</Suspense>
                        </dd>
                    </dl>
                </div>
                <div class="bg-neutral-800 px-4 sm:px-8 py-4 rounded-md">
                    <dl class="m-0">
                        <dt class="text-sm text-neutral-400 font-medium">
                            {move || token_y_symbol.get()} " Reserves"
                        </dt>
                        <dd class="m-0 text-2xl font-semibold align-baseline proportional-nums">
                            <Suspense fallback=|| {
                                view! { "Loading..." }
                            }>{move || Suspend::new(async move { reserve_y.await })}</Suspense>
                        </dd>
                    </dl>
                </div>
                <div class="bg-neutral-800 px-4 sm:px-8 py-4 rounded-md">
                    <dl class="m-0">
                        <dt class="text-sm text-neutral-400 font-medium">"+2% Depth"</dt>
                        <dd class="m-0 text-2xl font-semibold align-baseline proportional-nums">
                            "TBD " {move || token_x_symbol.get()}
                        </dd>
                    </dl>
                </div>
                <div class="bg-neutral-800 px-4 sm:px-8 py-4 rounded-md">
                    <dl class="m-0">
                        <dt class="text-sm text-neutral-400 font-medium">"-2% Depth"</dt>
                        <dd class="m-0 text-2xl font-semibold align-baseline proportional-nums">
                            "TBD " {move || token_y_symbol.get()}
                        </dd>
                    </dl>
                </div>
            </div>
            <div class="p-4 sm:p-7 bg-neutral-800 rounded-md border border-solid border-neutral-700">
                <div class="grid grid-cols-[minmax(0px,_1fr)_80px_80px] sm:grid-cols-[minmax(0px,_3fr)_minmax(0px,_1fr)_minmax(0px,_1fr)]">
                    <div class="flex flex-col gap-4">
                        <div class="flex flex-col items-start">
                            <p class="text-sm text-neutral-400 font-semibold m-0">"Pool"</p>
                            <div class="flex flex-row items-center gap-2">
                                <p class="text-base font-semibold m-0">{pool_address()}</p>
                                <div
                                    on:click={
                                        let copy = copy.clone();
                                        move |_| copy(pool_address().as_str())
                                    }
                                    class="hover:brightness-75 transition-all active:brightness-150"
                                >
                                    <Copy size=20 stroke_width=3 color="#a3a3a3" />
                                </div>
                                <a
                                    href=format!(
                                        "https://testnet.ping.pub/secret/account/{}",
                                        pool_address(),
                                    )
                                    target="_blank"
                                    rel="noopener"
                                    class="hover:brightness-75 transition-all"
                                >
                                    <Link size=20 stroke_width=3 color="#a3a3a3" />
                                </a>
                            </div>
                        </div>
                        <div class="flex flex-col items-start">
                            <p class="text-sm text-neutral-400 font-semibold m-0">
                                {move || token_x_symbol.get()}
                            </p>
                            <div class="flex flex-row items-center gap-2">
                                <p class="text-base font-semibold m-0">{token_x_address()}</p>
                                <div
                                    on:click={
                                        let copy = copy.clone();
                                        move |_| copy(token_x_address().as_str())
                                    }
                                    class="hover:brightness-75 transition-all active:brightness-150"
                                >
                                    <Copy size=20 stroke_width=3 color="#a3a3a3" />
                                </div>
                                <a
                                    href=format!(
                                        "https://testnet.ping.pub/secret/account/{}",
                                        token_x_address(),
                                    )
                                    target="_blank"
                                    rel="noopener"
                                    class="hover:brightness-75 transition-all"
                                >
                                    <Link size=20 stroke_width=3 color="#a3a3a3" />
                                </a>
                            </div>
                        </div>
                        <div class="flex flex-col items-start">
                            <p class="text-sm text-neutral-400 font-semibold m-0">
                                {move || token_y_symbol.get()}
                            </p>
                            <div class="flex flex-row items-center gap-2">
                                <p class="text-base font-semibold m-0">{token_y_address()}</p>
                                <div
                                    on:click={
                                        let copy = copy.clone();
                                        move |_| copy(token_y_address().as_str())
                                    }
                                    class="hover:brightness-75 transition-all active:brightness-150"
                                >
                                    <Copy size=20 stroke_width=3 color="#a3a3a3" />
                                </div>
                                <a
                                    href=format!(
                                        "https://testnet.ping.pub/secret/account/{}",
                                        token_y_address(),
                                    )
                                    target="_blank"
                                    rel="noopener"
                                    class="hover:brightness-75 transition-all"
                                >
                                    <Link size=20 stroke_width=3 color="#a3a3a3" />
                                </a>
                            </div>
                        </div>
                    </div>
                    <div class="flex flex-col gap-4">
                        <div class="flex flex-col items-start">
                            <p class="text-sm text-neutral-400 font-semibold m-0">"Bin steps"</p>
                            <p class="text-base font-semibold m-0">{bin_step()}"bps"</p>
                        </div>
                        // TODO: is the Version stored anywhere?
                        <div class="flex flex-col items-start">
                            <p class="text-sm text-neutral-400 font-semibold m-0">"Pool Version"</p>
                            <p class="text-base font-semibold m-0">v2.2</p>
                        </div>
                        // TODO: I'm not sure what this means.
                        <div class="flex flex-col items-start">
                            <p class="text-sm text-neutral-400 font-semibold m-0">"Pool Status"</p>
                            <p class="text-base font-semibold m-0">Main</p>
                        </div>
                    </div>
                    <div class="flex flex-col gap-4">
                        <div class="flex flex-col items-start">
                            <p class="text-sm text-neutral-400 font-semibold m-0">"Base fee"</p>
                            <p class="text-base font-semibold m-0">
                                <Suspense fallback=|| {
                                    view! { "Loading..." }
                                }>{move || Suspend::new(async move { base_fee.await })}</Suspense>
                            </p>
                        </div>
                        <div class="flex flex-col items-start">
                            <p class="text-sm text-neutral-400 font-semibold m-0">"Max fee"</p>
                            <p class="text-base font-semibold m-0">
                                <Suspense fallback=|| {
                                    view! { "Loading..." }
                                }>{move || Suspend::new(async move { max_fee.await })}</Suspense>
                            </p>
                        </div>
                        <div class="flex flex-col items-start">
                            <p class="text-sm text-neutral-400 font-semibold m-0">"Protocol fee"</p>
                            <p class="text-base font-semibold m-0">
                                <Suspense fallback=|| {
                                    view! { "Loading..." }
                                }>
                                    {move || Suspend::new(async move { protocol_fee.await })}
                                </Suspense>
                            </p>
                        </div>
                    </div>
                </div>
            </div>
            <div class="p-4 sm:p-8 bg-neutral-800 rounded-md border border-solid border-neutral-700">
                <div class="relative">
                    <div class="flex justify-between w-full">
                        <div class="text-xl leading-tight font-semibold text-white">
                            "Pool Distribution"
                        </div>
                        <div class="flex flex-row gap-2 items-center">
                            <div class="flex flex-row gap-1 items-center">
                                <div class="w-2 h-2 rounded-full bg-gold"></div>
                                <p class="m-0">{move || token_x_symbol.get()}</p>
                            </div>
                            <div class="flex flex-row gap-1 items-center">
                                <div class="w-2 h-2 rounded-full bg-pine"></div>
                                <p class="m-0">{move || token_y_symbol.get()}</p>
                            </div>
                        </div>
                    </div>
                    <Suspense fallback=|| {
                        view! { "Loading..." }
                    }>
                        <div>{move || Suspend::new(async move { chart_data.await })}</div>
                    </Suspense>
                </div>
            </div>
        </div>
    }
}
