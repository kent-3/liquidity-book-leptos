use crate::state::{PoolState, PoolStateStoreFields};
use ammber_charts::{PoolDistributionChart, ReserveData};
use ammber_core::{
    utils::{display_token_amount, shorten_address},
    Error,
};
use ammber_sdk::contract_interfaces::lb_pair::BinResponse;
use leptos::prelude::*;
use leptos_use::{use_clipboard, UseClipboardReturn};
use lucide_leptos::{Copy, Link};
use reactive_stores::Store;
use tracing::{debug, error, info};

#[component]
pub fn PoolAnalytics() -> impl IntoView {
    info!("rendering <PoolAnalytics/>");

    on_cleanup(move || {
        info!("cleaning up <PoolAnalytics/>");
    });

    let UseClipboardReturn { copy, .. } = use_clipboard();

    // benefit of using Store. no Option or Result types to deal with
    let pool = use_context::<Store<PoolState>>().expect("missing the Store<PoolState> context");

    // TODO:
    // - Liquidity
    // - Volume
    // - Fees
    // - APR
    // - +/- 2% depth

    let reserve_x = move || {
        let token_x = pool.token_x().get();
        let reserve_x = pool.total_reserves().get().reserve_x.u128();

        let amount_x = if token_x.decimals > 0 {
            reserve_x / 10u128.pow(token_x.decimals as u32)
        } else {
            reserve_x
        };

        format!("{} {}", amount_x, token_x.symbol)
    };
    let reserve_y = move || {
        let token_y = pool.token_y().get();
        let reserve_y = pool.total_reserves().get().reserve_y.u128();

        let amount_y = if token_y.decimals > 0 {
            reserve_y / 10u128.pow(token_y.decimals as u32)
        } else {
            reserve_y
        };

        format!("{} {}", amount_y, token_y.symbol)
    };

    let token_x_symbol = move || {
        let token = pool.token_x().get();
        token.display_name.unwrap_or(token.symbol)
    };
    let token_y_symbol = move || {
        let token = pool.token_y().get();
        token.display_name.unwrap_or(token.symbol)
    };

    let pool_address = move || pool.lb_pair().get().contract.address.to_string();
    let token_x_address = move || pool.lb_pair().get().token_x.address().to_string();
    let token_y_address = move || pool.lb_pair().get().token_y.address().to_string();

    let bin_step = move || pool.lb_pair().get().bin_step;
    let base_fee = move || {
        let base_factor = pool.static_fee_parameters().get().base_factor as u64;
        let fee_bps = base_factor * bin_step() as u64 / 10_000;

        format!("{}.{}%", fee_bps / 100, fee_bps % 100)
    };
    let max_fee = move || {
        // TODO: this seems right, but double check the maths
        let static_fee_parameters = pool.static_fee_parameters().get();
        let base_factor = static_fee_parameters.base_factor as u64;
        let variable_fee_control = static_fee_parameters.variable_fee_control as u64;
        let max_volatility_accumulator = static_fee_parameters.max_volatility_accumulator as u64;

        let base_fee = base_factor * bin_step() as u64 * 10_000_000_000;

        let prod = (max_volatility_accumulator as u64) * 100;
        let max_variable_fee = (prod * prod * variable_fee_control + 99) / 100;

        let max_fee = base_fee + max_variable_fee;
        let max_fee_bps = max_fee / 100_000_000_000_000;

        format!("{}.{}%", max_fee_bps / 100, max_fee_bps % 100)
    };
    let protocol_fee = move || {
        let protocol_fee = pool.static_fee_parameters().get().protocol_share;

        format!("{}.{}%", protocol_fee / 100, protocol_fee % 100)
    };

    // TODO: should this be a resource instead? so we can show something while loading
    let nearby_bins = use_context::<RwSignal<Result<Vec<BinResponse>, Error>>>()
        .expect("missing nearby_bins context");

    let debug = RwSignal::new(false);

    // TODO: doesn't need to be async
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
            // .inspect(|ok| debug!("{ok:?}"))
            .inspect_err(|err| debug!("{err:?}"))
    });

    let token_labels = Signal::derive(move || {
        let token_x = token_x_symbol();
        let token_y = token_y_symbol();

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
                        let token_labels = token_labels;
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
                            {token_x_symbol} " Reserves"
                        </dt>
                        <dd class="pt-0.5 text-2xl font-semibold align-baseline proportional-nums">
                            {reserve_x}
                        </dd>
                    </dl>
                </div>
                <div class="bg-card px-4 sm:px-8 py-4 rounded-lg">
                    <dl class="m-0">
                        <dt class="text-sm text-muted-foreground font-medium">
                            {token_y_symbol} " Reserves"
                        </dt>
                        <dd class="pt-0.5 text-2xl font-semibold align-baseline proportional-nums">
                            {reserve_y}
                        </dd>
                    </dl>
                </div>
                <div class="bg-card px-4 sm:px-8 py-4 rounded-lg">
                    <dl class="m-0">
                        <dt class="text-sm text-muted-foreground font-medium">"+2% Depth"</dt>
                        <dd class="pt-0.5 text-2xl font-semibold align-baseline proportional-nums">
                            "TBD " {token_x_symbol}
                        </dd>
                    </dl>
                </div>
                <div class="bg-card px-4 sm:px-8 py-4 rounded-lg">
                    <dl class="m-0">
                        <dt class="text-sm text-muted-foreground font-medium">"-2% Depth"</dt>
                        <dd class="pt-0.5 text-2xl font-semibold align-baseline proportional-nums">
                            "TBD " {token_y_symbol}
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
                                    {pool_address}
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
                                {token_x_symbol}
                            </p>
                            <div class="flex flex-row items-center gap-2">
                                <p class="hidden lg:block text-base font-semibold m-0">
                                    {token_x_address}
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
                                {token_y_symbol}
                            </p>
                            <div class="flex flex-row items-center gap-2">
                                <p class="hidden lg:block text-base font-semibold m-0">
                                    {token_y_address}
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
                            <p class="text-base font-semibold m-0">{bin_step}"bps"</p>
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
                            <p class="text-base font-semibold m-0">{base_fee}</p>
                        </div>
                        <div class="flex flex-col items-start">
                            <p class="text-sm text-muted-foreground font-semibold m-0">"Max fee"</p>
                            <p class="text-base font-semibold m-0">{max_fee}</p>
                        </div>
                        <div class="flex flex-col items-start">
                            <p class="text-sm text-muted-foreground font-semibold m-0">
                                "Protocol fee"
                            </p>
                            <p class="text-base font-semibold m-0">{protocol_fee}</p>
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
                            <p class="m-0">{token_y_symbol}</p>
                        </div>
                        <div class="flex flex-row gap-1 items-center">
                            <div class="w-2 h-2 rounded-full bg-pine"></div>
                            <p class="m-0">{token_x_symbol}</p>
                        </div>
                    </div>
                </div>
                <div class="flex justify-center w-full">{chart_element}</div>
            </div>
        </div>
    }
}
