use ammber_charts::{PoolDistributionChart, ReserveData};
use ammber_core::support::chain_query;
use ammber_core::utils::{display_token_amount, get_token_decimals, shorten_address};
use ammber_core::Error;
use ammber_sdk::contract_interfaces::lb_pair::{
    self, BinResponse, LbPair, ReservesResponse, StaticFeeParametersResponse,
};
use batch_query::{
    msg_batch_query, parse_batch_query, BatchItemResponseStatus, BatchQuery, BatchQueryParams,
    BatchQueryParsedResponse, BatchQueryResponse, BATCH_QUERY_ROUTER,
};
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_use::use_clipboard;
use liquidity_book::interfaces::lb_pair::BinsResponse;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info};

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
    let lb_pair = use_context::<LocalResource<Result<LbPair, Error>>>()
        .expect("missing the LbPair resource context");
    let active_id = use_context::<LocalResource<Result<u32, Error>>>()
        .expect("missing the active_id resource context");
    let static_fee_parameters =
        use_context::<LocalResource<Result<StaticFeeParametersResponse, Error>>>()
            .expect("missing the static fee parameters resource context");

    // TODO: decide on making these signals sync or async
    let bin_step = move || {
        lb_pair
            .get()
            .as_deref()
            .and_then(|result| result.clone().ok())
            .map(|pair| pair.bin_step)
            .unwrap_or_default()
    };
    let pool_address = move || {
        lb_pair
            .get()
            .as_deref()
            .and_then(|result| result.clone().ok())
            .map(|pair| pair.contract.address.to_string())
            .unwrap_or_default()
    };
    let token_x_address = move || {
        lb_pair
            .get()
            .as_deref()
            .and_then(|result| result.clone().ok())
            .map(|pair| pair.token_x.address().to_string())
            .unwrap_or_default()
    };
    let token_y_address = move || {
        lb_pair
            .get()
            .as_deref()
            .and_then(|result| result.clone().ok())
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

        // debug!("{base_fee}");

        let prod = (max_volatility_accumulator as u128) * (100 as u128);
        let max_variable_fee = (prod * prod * variable_fee_control + 99) / 100;

        // debug!("{max_variable_fee}");

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

    let total_reserves = use_context::<LocalResource<Result<ReservesResponse, Error>>>()
        .expect("missing the total reserves resource context");

    let reserve_x = AsyncDerived::new(move || async move {
        let decimals_x = get_token_decimals(token_x_address().as_str());

        let amount = total_reserves
            .await
            .map(|r| {
                if let Ok(decimals) = decimals_x {
                    let full_amount = display_token_amount(r.reserve_x.u128(), decimals);
                    full_amount
                        .splitn(2, '.')
                        .next()
                        .unwrap_or(&full_amount)
                        .to_string()
                } else {
                    r.reserve_x.to_string()
                }
            })
            .unwrap_or(0.to_string());
        let denom = token_x_symbol.await;

        format!("{} {}", amount, denom)
    });
    let reserve_y = AsyncDerived::new(move || async move {
        let decimals_y = get_token_decimals(token_y_address().as_str());

        let amount = total_reserves
            .await
            .map(|r| {
                if let Ok(decimals) = decimals_y {
                    let full_amount = display_token_amount(r.reserve_y.u128(), decimals);
                    full_amount
                        .splitn(2, '.')
                        .next()
                        .unwrap_or(&full_amount)
                        .to_string()
                } else {
                    r.reserve_y.to_string()
                }
            })
            .unwrap_or(0.to_string());
        let denom = token_y_symbol.await;

        format!("{} {}", amount, denom)
    });

    let nearby_bins = RwSignal::<Result<Vec<BinResponse>, Error>>::new(Ok(vec![]));

    spawn_local(async move {
        let lb_pair_result = lb_pair.await;
        let id_result = active_id.await;

        let lb_pair_contract = match lb_pair_result {
            Ok(pair) => pair.contract,
            Err(err) => {
                error!("Failed to get LB pair: {:?}", err);
                nearby_bins.set(Err(err.into())); // Convert error and set in state
                return;
            }
        };

        let id = match id_result {
            Ok(id) => id,
            Err(err) => {
                error!("Failed to get active ID: {:?}", err);
                nearby_bins.set(Err(err.into()));
                return;
            }
        };

        let mut ids = Vec::new();
        let radius = 49;
        for i in 0..(radius * 2 + 1) {
            let offset_id = if i < radius {
                id - (radius - i) as u32
            } else {
                id + (i - radius) as u32
            };
            ids.push(offset_id);
        }

        debug!("getting nearby bins");

        match chain_query::<BinsResponse>(
            lb_pair_contract.code_hash.clone(),
            lb_pair_contract.address.to_string(),
            lb_pair::QueryMsg::GetBins { ids },
        )
        .await
        {
            Ok(response) => nearby_bins.set(Ok(response.0)),
            Err(err) => {
                error!("Failed to get bins: {:?}", err);
                nearby_bins.set(Err(err.into()));
            }
        }
    });

    // 8.7 kb
    // TODO: decide if this should be a Resource or not
    // let nearby_bins = LocalResource::new(move || {
    //     let active_id = active_id.get();
    //     async move {
    //         let Some(Ok(id)) = active_id.as_deref() else {
    //             return Err(Error::generic("active_id is not ready yet"));
    //         };
    //
    //         let lb_pair_contract = lb_pair.await?.contract;
    //         let mut ids = Vec::new();
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
    //             ids.push(offset_id);
    //         }
    //
    //         debug!("getting nearby bins");
    //
    //         let bins = chain_query::<BinsResponse>(
    //             lb_pair_contract.code_hash.clone(),
    //             lb_pair_contract.address.to_string(),
    //             lb_pair::QueryMsg::GetBins { ids },
    //         )
    //         .await
    //         .map(|response| response.0);
    //
    //         bins
    //     }
    // });

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

    let debug = RwSignal::new(false);

    let chart_data = LocalResource::new(move || async move {
        debug!("processing chart data");

        nearby_bins
            .get()
            // .as_deref()
            // .unwrap()
            // .clone()
            .map(|bins| {
                bins.iter()
                    .map(|bin_response| {
                        ReserveData::from_bin(
                            bin_response.bin_id,
                            bin_response.bin_reserve_y.u128(),
                            bin_response.bin_reserve_x.u128(),
                        )
                    })
                    .collect::<Vec<ReserveData>>()
            })
            .inspect(|ok| debug!("{ok:?}"))
            .inspect_err(|err| debug!("{err:?}"))
    });

    let token_labels = AsyncDerived::new(move || async move {
        let token_x = token_x_symbol.await;
        let token_y = token_y_symbol.await;

        (token_x, token_y)
    });

    #[cfg(feature = "charts")]
    let chart_element = view! {
        <div class="flex justify-center w-full">
            <Suspense fallback=|| {
                view! { "Loading..." }
            }>
                {move || {
                    Suspend::new(async move {
                        let data = chart_data.await.unwrap_or_default();
                        let token_labels = token_labels.await;
                        view! {
                            <PoolDistributionChart
                                debug=debug.into()
                                data=data.into()
                                token_labels=token_labels.into()
                            />
                        }
                    })
                }}
            </Suspense>
        </div>
    };

    #[cfg(not(feature = "charts"))]
    let chart_element = view! {
        <div class="flex items-center justify-center w-full h-[160px]">
            <code>"Charts disabled"</code>
        </div>
    };

    // TODO: make signals for delta liquidity, volume, and fees. toggle class by value.

    view! {
        <div class="flex flex-col gap-4">
            <div class="grid grid-cols-2 sm:grid-cols-3 gap-4">
                <div class="bg-card px-4 sm:px-8 py-4 rounded-lg">
                    <dl class="m-0">
                        <dt class="text-sm text-muted-foreground font-medium">"Liquidity"</dt>
                        <div class="flex items-start flex-col sm:flex-row sm:items-center sm:gap-2">
                            <dd class="py-0.5 sm:pb-0 text-2xl font-semibold align-baseline proportional-nums">
                                "$0.00"
                            </dd>
                            <dd class="sm:pt-0.5 text-foam text-sm font-semibold">"0%"</dd>
                        </div>
                    </dl>
                </div>
                <div class="bg-card px-4 sm:px-8 py-4 rounded-lg">
                    <dl class="m-0">
                        <dt class="text-sm text-muted-foreground font-medium">"Volume (24H)"</dt>
                        <div class="flex items-start flex-col sm:flex-row sm:items-center sm:gap-2">
                            <dd class="py-0.5 sm:pb-0 text-2xl font-semibold align-baseline proportional-nums">
                                "$0.00"
                            </dd>
                            <dd class="sm:pt-0.5 text-rose text-sm font-semibold">"0%"</dd>
                        </div>
                    </dl>
                </div>
                <div class="bg-card px-4 sm:px-8 py-4 rounded-lg">
                    <dl class="m-0">
                        <dt class="text-sm text-muted-foreground font-medium">"Fees (24H)"</dt>
                        <div class="flex items-start flex-col sm:flex-row sm:items-center sm:gap-2">
                            <dd class="py-0.5 sm:pb-0 text-2xl font-semibold align-baseline proportional-nums">
                                "$0.00"
                            </dd>
                            <dd class="sm:pt-0.5 text-rose text-sm font-semibold">"0%"</dd>
                        </div>
                    </dl>
                </div>
                <div class="bg-card px-4 sm:px-8 py-4 rounded-lg">
                    <dl class="m-0">
                        <dt class="text-sm text-muted-foreground font-medium">"APR (7D)"</dt>
                        <dd class="pt-0.5 text-2xl font-semibold align-baseline proportional-nums">
                            "0.00%"
                        </dd>
                    </dl>
                </div>
                <div class="bg-card px-4 sm:px-8 py-4 rounded-lg">
                    <dl class="m-0">
                        <dt class="text-sm text-muted-foreground font-medium">
                            {move || token_x_symbol.get()} " Reserves"
                        </dt>
                        <dd class="pt-0.5 text-2xl font-semibold align-baseline proportional-nums">
                            <Suspense fallback=|| {
                                view! { "Loading..." }
                            }>{move || Suspend::new(async move { reserve_x.await })}</Suspense>
                        </dd>
                    </dl>
                </div>
                <div class="bg-card px-4 sm:px-8 py-4 rounded-lg">
                    <dl class="m-0">
                        <dt class="text-sm text-muted-foreground font-medium">
                            {move || token_y_symbol.get()} " Reserves"
                        </dt>
                        <dd class="pt-0.5 text-2xl font-semibold align-baseline proportional-nums">
                            <Suspense fallback=|| {
                                view! { "Loading..." }
                            }>{move || Suspend::new(async move { reserve_y.await })}</Suspense>
                        </dd>
                    </dl>
                </div>
                <div class="bg-card px-4 sm:px-8 py-4 rounded-lg">
                    <dl class="m-0">
                        <dt class="text-sm text-muted-foreground font-medium">"+2% Depth"</dt>
                        <dd class="pt-0.5 text-2xl font-semibold align-baseline proportional-nums">
                            "TBD " {move || token_x_symbol.get()}
                        </dd>
                    </dl>
                </div>
                <div class="bg-card px-4 sm:px-8 py-4 rounded-lg">
                    <dl class="m-0">
                        <dt class="text-sm text-muted-foreground font-medium">"-2% Depth"</dt>
                        <dd class="pt-0.5 text-2xl font-semibold align-baseline proportional-nums">
                            "TBD " {move || token_y_symbol.get()}
                        </dd>
                    </dl>
                </div>
            </div>
            <div class="p-4 sm:p-7 bg-card rounded-lg border border-solid border-border">
                <div class="grid grid-cols-[minmax(0px,_1fr)_80px_80px] sm:grid-cols-[minmax(0px,_3fr)_minmax(0px,_1fr)_minmax(0px,_1fr)]">
                    <div class="flex flex-col gap-4">
                        <div class="flex flex-col items-start">
                            <p class="text-sm text-muted-foreground font-semibold m-0">"Pool"</p>
                            <div class="flex flex-row items-center gap-2">
                                <p class="hidden lg:block text-base font-semibold m-0">
                                    {move || pool_address()}
                                </p>
                                <p class="block lg:hidden text-base font-semibold m-0">
                                    {move || shorten_address(pool_address())}
                                </p>
                                <div
                                    on:click={
                                        let copy = copy.clone();
                                        move |_| copy(pool_address().as_str())
                                    }
                                    class="hidden md:inline text-muted-foreground hover:brightness-75 active:brightness-125"
                                >
                                    <Copy size=20 stroke_width=3 />
                                </div>
                                <a
                                    href=move || {
                                        format!(
                                            "https://testnet.ping.pub/secret/account/{}",
                                            pool_address(),
                                        )
                                    }
                                    target="_blank"
                                    rel="noopener"
                                    class="text-muted-foreground hover:brightness-75"
                                >
                                    <Link size=20 stroke_width=3 />
                                </a>
                            </div>
                        </div>
                        <div class="flex flex-col items-start">
                            <p class="text-sm text-muted-foreground font-semibold m-0">
                                {move || token_x_symbol.get()}
                            </p>
                            <div class="flex flex-row items-center gap-2">
                                <p class="hidden lg:block text-base font-semibold m-0">
                                    {move || token_x_address()}
                                </p>
                                <p class="block lg:hidden text-base font-semibold m-0">
                                    {move || shorten_address(token_x_address())}
                                </p>
                                <div
                                    on:click={
                                        let copy = copy.clone();
                                        move |_| copy(token_x_address().as_str())
                                    }
                                    class="hidden md:inline text-muted-foreground hover:brightness-75 active:brightness-125"
                                >
                                    <Copy size=20 stroke_width=3 />
                                </div>
                                <a
                                    href=move || {
                                        format!(
                                            "https://testnet.ping.pub/secret/account/{}",
                                            token_x_address(),
                                        )
                                    }
                                    target="_blank"
                                    rel="noopener"
                                    class="text-muted-foreground hover:brightness-75"
                                >
                                    <Link size=20 stroke_width=3 />
                                </a>
                            </div>
                        </div>
                        <div class="flex flex-col items-start">
                            <p class="text-sm text-muted-foreground font-semibold m-0">
                                {move || token_y_symbol.get()}
                            </p>
                            <div class="flex flex-row items-center gap-2">
                                <p class="hidden lg:block text-base font-semibold m-0">
                                    {move || token_y_address()}
                                </p>
                                <p class="block lg:hidden text-base font-semibold m-0">
                                    {move || shorten_address(token_y_address())}
                                </p>
                                <div
                                    on:click={
                                        let copy = copy.clone();
                                        move |_| copy(token_y_address().as_str())
                                    }
                                    class="hidden md:inline text-muted-foreground hover:brightness-75 active:brightness-125"
                                >
                                    <Copy size=20 stroke_width=3 />
                                </div>
                                <a
                                    href=move || {
                                        format!(
                                            "https://testnet.ping.pub/secret/account/{}",
                                            token_y_address(),
                                        )
                                    }
                                    target="_blank"
                                    rel="noopener"
                                    class="text-muted-foreground hover:brightness-75"
                                >
                                    <Link size=20 stroke_width=3 />
                                </a>
                            </div>
                        </div>
                    </div>
                    <div class="flex flex-col gap-4">
                        <div class="flex flex-col items-start">
                            <p class="text-sm text-muted-foreground font-semibold m-0">
                                "Bin steps"
                            </p>
                            <p class="text-base font-semibold m-0">{move || bin_step()}"bps"</p>
                        </div>
                        // TODO: is the Version stored anywhere?
                        <div class="flex flex-col items-start">
                            <p class="text-sm text-muted-foreground font-semibold m-0">"Version"</p>
                            <p class="text-base font-semibold m-0">v2.2</p>
                        </div>
                        // TODO: I'm not sure what this means.
                        <div class="flex flex-col items-start">
                            <p class="text-sm text-muted-foreground font-semibold m-0">"Status"</p>
                            <p class="text-base font-semibold m-0">Main</p>
                        </div>
                    </div>
                    <div class="flex flex-col gap-4">
                        <div class="flex flex-col items-start">
                            <p class="text-sm text-muted-foreground font-semibold m-0">
                                "Base fee"
                            </p>
                            <p class="text-base font-semibold m-0">
                                <Suspense fallback=|| {
                                    view! { "Loading..." }
                                }>{move || Suspend::new(async move { base_fee.await })}</Suspense>
                            </p>
                        </div>
                        <div class="flex flex-col items-start">
                            <p class="text-sm text-muted-foreground font-semibold m-0">"Max fee"</p>
                            <p class="text-base font-semibold m-0">
                                <Suspense fallback=|| {
                                    view! { "Loading..." }
                                }>{move || Suspend::new(async move { max_fee.await })}</Suspense>
                            </p>
                        </div>
                        <div class="flex flex-col items-start">
                            <p class="text-sm text-muted-foreground font-semibold m-0">
                                "Protocol fee"
                            </p>
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
            <div class="p-4 sm:p-8 bg-card rounded-lg border border-solid border-border">
                <div class="flex justify-between w-full">
                    <div class="text-xl leading-tight font-semibold text-white">
                        "Pool Distribution"
                    </div>
                    <div class="flex flex-row gap-2 items-center">
                        <div class="flex flex-row gap-1 items-center">
                            <div class="w-2 h-2 rounded-full bg-gold"></div>
                            <p class="m-0">{move || token_y_symbol.get()}</p>
                        </div>
                        <div class="flex flex-row gap-1 items-center">
                            <div class="w-2 h-2 rounded-full bg-pine"></div>
                            <p class="m-0">{move || token_x_symbol.get()}</p>
                        </div>
                    </div>
                </div>
                <div class="flex justify-center w-full">{chart_element}</div>
            </div>
        </div>
    }
}
