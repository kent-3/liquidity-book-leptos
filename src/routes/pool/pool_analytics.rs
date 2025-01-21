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
use leptos_use::use_clipboard;
use lucide_leptos::{ArrowLeft, ExternalLink, Plus};
use secret_toolkit_snip20::TokenInfoResponse;
use send_wrapper::SendWrapper;
use serde::Serialize;
use tracing::{debug, info, trace};

#[component]
pub fn PoolAnalytics() -> impl IntoView {
    info!("rendering <PoolAnalytics/>");

    use leptos_use::UseClipboardReturn;
    use lucide_leptos::{Copy, Link};

    let UseClipboardReturn {
        is_supported,
        text,
        copied,
        copy,
    } = use_clipboard();

    let lb_pair = use_context::<Resource<LbPair>>().expect("missing the LbPair resource context");
    let active_id = use_context::<Resource<Result<u32, Error>>>()
        .expect("missing the active_id resource context");
    // TODO: give this context a named type? not really necessary for a one-off
    let (token_a_symbol, token_b_symbol) = use_context::<(
        AsyncDerived<String, LocalStorage>,
        AsyncDerived<String, LocalStorage>,
    )>()
    .expect("missing token symbols context");

    // TODO: process all the signals and resources before using them in the view?

    let pool_address = move || {
        lb_pair
            .get_untracked()
            .map(|pair| pair.contract.address.to_string())
            .unwrap_or_default()
    };
    let token_x_address = move || {
        lb_pair
            .get_untracked()
            .map(|pair| pair.token_x.address().to_string())
            .unwrap_or_default()
    };
    let token_y_address = move || {
        lb_pair
            .get_untracked()
            .map(|pair| pair.token_y.address().to_string())
            .unwrap_or_default()
    };

    let total_reserves = Resource::new(
        move || (),
        move |_| async move {
            ILbPair(lb_pair.await.contract).get_reserves().await
            // .map(|response| {
            //     (
            //         response.reserve_x.to_string(),
            //         response.reserve_y.to_string(),
            //     )
            // })
            // .unwrap_or_default()
        },
    );

    // TODO: I think there's a different way to write this, like a a regular derived signal since I
    // don't really need to await it (I probably should though, to show a skeleton or "loading...")
    let reserve_x = move || {
        total_reserves
            .get()
            .and_then(Result::ok)
            .map(|r| r.reserve_x.to_string())
            .unwrap_or_default()
    };
    // let reserve_x = AsyncDerived::new(move || async move {
    //     total_reserves
    //         .await
    //         .map(|r| r.reserve_x.to_string())
    //         .unwrap_or_default()
    // });
    let reserve_y = AsyncDerived::new(move || async move {
        total_reserves
            .await
            .map(|r| r.reserve_y.to_string())
            .unwrap_or_default()
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
                            <dd class="m-0 text-foam text-sm font-semibold">"50%"</dd>
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
                            <dd class="m-0 text-rose text-sm font-semibold">"-10%"</dd>
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
                            <dd class="m-0 text-rose text-sm font-semibold">"-10%"</dd>
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
                            {move || token_a_symbol.get()} " Reserves"
                        </dt>
                        <dd class="m-0 text-2xl font-semibold align-baseline proportional-nums">
                            {reserve_x} " " {move || token_a_symbol.get()}
                        </dd>
                    </dl>
                </div>
                <div class="bg-neutral-800 px-4 sm:px-8 py-4 rounded-md">
                    <dl class="m-0">
                        <dt class="text-sm text-neutral-400 font-medium">
                            {move || token_b_symbol.get()} " Reserves"
                        </dt>
                        <dd class="m-0 text-2xl font-semibold align-baseline proportional-nums">
                            {move || reserve_y.get()} " " {move || token_b_symbol.get()}
                        </dd>
                    </dl>
                </div>
                <div class="bg-neutral-800 px-4 sm:px-8 py-4 rounded-md">
                    <dl class="m-0">
                        <dt class="text-sm text-neutral-400 font-medium">"+2% Depth"</dt>
                        <dd class="m-0 text-2xl font-semibold align-baseline proportional-nums">
                            "000 " {move || token_a_symbol.get()}
                        </dd>
                    </dl>
                </div>
                <div class="bg-neutral-800 px-4 sm:px-8 py-4 rounded-md">
                    <dl class="m-0">
                        <dt class="text-sm text-neutral-400 font-medium">"-2% Depth"</dt>
                        <dd class="m-0 text-2xl font-semibold align-baseline proportional-nums">
                            "000 " {move || token_b_symbol.get()}
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
                                    class="hover:brightness-75 transition-all"
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
                                    class="appearance-none"
                                >
                                    <Link size=20 stroke_width=3 color="#a3a3a3" />
                                </a>
                            </div>
                        </div>
                        <div class="flex flex-col items-start">
                            <p class="text-sm text-neutral-400 font-semibold m-0">
                                {move || token_a_symbol.get()}
                            </p>
                            <div class="flex flex-row items-center gap-2">
                                <p class="text-base font-semibold m-0">{token_x_address()}</p>
                                <div
                                    on:click={
                                        let copy = copy.clone();
                                        move |_| copy(token_x_address().as_str())
                                    }
                                    class="hover:brightness-75 transition-all"
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
                                    class="appearance-none"
                                >
                                    <Link size=20 stroke_width=3 color="#a3a3a3" />
                                </a>
                            </div>
                        </div>
                        <div class="flex flex-col items-start">
                            <p class="text-sm text-neutral-400 font-semibold m-0">
                                {move || token_b_symbol.get()}
                            </p>
                            <div class="flex flex-row items-center gap-2">
                                <p class="text-base font-semibold m-0">{token_y_address()}</p>
                                <div
                                    on:click={
                                        let copy = copy.clone();
                                        move |_| copy(token_y_address().as_str())
                                    }
                                    class="hover:brightness-75 transition-all"
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
                                    class="appearance-none"
                                >
                                    <Link size=20 stroke_width=3 color="#a3a3a3" />
                                </a>
                            </div>
                        </div>
                    </div>
                    <div class="flex flex-col gap-4">
                        <div class="flex flex-col items-start">
                            <p class="text-sm text-neutral-400 font-semibold m-0">"Bin steps"</p>
                            <p class="text-base font-semibold m-0">100bps</p>
                        </div>
                        <div class="flex flex-col items-start">
                            <p class="text-sm text-neutral-400 font-semibold m-0">"Pool Version"</p>
                            <p class="text-base font-semibold m-0">v2.2</p>
                        </div>
                        <div class="flex flex-col items-start">
                            <p class="text-sm text-neutral-400 font-semibold m-0">"Pool Status"</p>
                            <p class="text-base font-semibold m-0">Main</p>
                        </div>
                    </div>
                    <div class="flex flex-col gap-4">
                        <div class="flex flex-col items-start">
                            <p class="text-sm text-neutral-400 font-semibold m-0">"Base fee"</p>
                            <p class="text-base font-semibold m-0">"0.01%"</p>
                        </div>
                        <div class="flex flex-col items-start">
                            <p class="text-sm text-neutral-400 font-semibold m-0">"Max fee"</p>
                            <p class="text-base font-semibold m-0">"0.03%"</p>
                        </div>
                        <div class="flex flex-col items-start">
                            <p class="text-sm text-neutral-400 font-semibold m-0">"Protocol fee"</p>
                            <p class="text-base font-semibold m-0">"5%"</p>
                        </div>
                    </div>
                </div>
            </div>
            <div class="p-4 sm:p-8 bg-neutral-800 rounded-md border border-solid border-neutral-700">
                <div class="relative h-52">
                    <div class="flex justify-between w-full">
                        <div class="text-xl leading-tight font-semibold text-white">
                            "Pool Distribution"
                        </div>
                        <div class="flex flex-row gap-2 items-center">
                            <div class="flex flex-row gap-1 items-center">
                                <div class="w-2 h-2 rounded-full bg-gold"></div>
                                <p class="m-0">{move || token_a_symbol.get()}</p>
                            </div>
                            <div class="flex flex-row gap-1 items-center">
                                <div class="w-2 h-2 rounded-full bg-pine"></div>
                                <p class="m-0">{move || token_b_symbol.get()}</p>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}
