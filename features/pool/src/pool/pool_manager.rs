use ammber_core::{prelude::*, state::*, support::ILbPair, Error, BASE_URL};
use ammber_sdk::contract_interfaces::lb_pair::LbPair;
use cosmwasm_std::Uint256;
use keplr::Keplr;
use leptos::prelude::*;
use leptos_router::{
    hooks::{use_location, use_navigate, use_params_map},
    nested_router::Outlet,
    NavigateOptions,
};
use lucide_leptos::Plus;
use tracing::{debug, info};

mod add_liquidity;
mod remove_liquidity;

pub use add_liquidity::AddLiquidity;
pub use remove_liquidity::RemoveLiquidity;

#[component]
pub fn PoolManager() -> impl IntoView {
    info!("rendering <PoolManager/>");

    let navigate = use_navigate();
    let location = use_location();
    let nav_options = NavigateOptions {
        // prevents scrolling to the top of the page each time a query param changes
        scroll: false,
        ..Default::default()
    };

    let endpoint = use_context::<Endpoint>().expect("endpoint context missing!");
    let keplr = use_context::<KeplrSignals>().expect("keplr signals context missing!");
    let token_map = use_context::<TokenMap>().expect("tokens context missing!");

    let lb_pair = use_context::<Resource<Result<LbPair, Error>>>()
        .expect("missing the LbPair resource context");
    let active_id = use_context::<Resource<Result<u32, Error>>>()
        .expect("missing the active_id resource context");

    let params = use_params_map();
    // TODO: decide on calling these a/b or x/y
    let token_a = move || {
        params
            .read()
            .get("token_a")
            .and_then(|token_address| TOKEN_MAP.get(&token_address))
            .expect("Missing token_a URL param")
    };
    let token_b = move || {
        params
            .read()
            .get("token_b")
            .and_then(|token_address| TOKEN_MAP.get(&token_address))
            .expect("Missing token_b URL param")
    };
    let basis_points = move || {
        params
            .read()
            .get("bps")
            .and_then(|value| value.parse::<u16>().ok())
            .expect("Missing bps URL param")
    };

    let token_a_symbol = Signal::derive(move || token_a().symbol.clone());
    let token_b_symbol = Signal::derive(move || token_b().symbol.clone());

    let my_liquidity = LocalResource::new(move || {
        let url = endpoint;
        let chain_id = CHAIN_ID;

        async move {
            if !keplr.enabled.get() {
                return Err(Error::generic("keplr is disabled"));
            }

            let Some(Ok(lb_pair)) = lb_pair.get() else {
                return Err(Error::generic("lb pair information is missing!"));
            };
            let Some(Ok(id)) = active_id.get() else {
                return Err(Error::generic("active id is missing!"));
            };

            let mut ids = vec![];
            let radius = 49;

            for i in 0..(radius * 2 + 1) {
                let offset_id = if i < radius {
                    id - (radius - i) as u32 // Subtract for the first half
                } else {
                    id + (i - radius) as u32 // Add for the second half
                };

                ids.push(offset_id);
            }

            debug!("{:?}", ids);

            let key = Keplr::get_key(&chain_id).await?;
            let account = key.bech32_address;

            let accounts = vec![account.clone(); ids.len()];

            let balances: Vec<Uint256> = ILbPair(lb_pair.contract.clone())
                .balance_of_batch(accounts, ids.clone())
                .await?;

            debug!("{:?}", balances);

            Ok((ids, balances))
        }
    });

    // TODO: Convert user liquidity from Uint256 to f64
    // let chart_data = Signal::derive(move || {
    //     if let Some(Ok((ids, amounts))) = my_liquidity.get().as_deref() {
    //         ids.iter()
    //             .cloned()
    //             .zip(amounts)
    //             .map(|(id, amount)| ReserveData::new(id.into(), amount.into(), 0.0))
    //             .collect()
    //     } else {
    //         vec![ReserveData::new(0.0, 0.0, 0.0)]
    //     }
    // });

    // TODO: all this chart stuff
    // use ammber_charts::{load_data, LiquidityChart, ReserveData};
    //
    // let debug = RwSignal::new(false);
    // let token_labels = (token_a_symbol, token_b_symbol);
    // let chart_data = load_data();
    //
    // #[cfg(feature = "charts")]
    // let chart_element = view! {
    //     <div class="flex justify-center w-full">
    //         <Suspense fallback=|| {
    //             view! { "Loading..." }
    //         }>
    //             {move || {
    //                 Suspend::new(async move {
    //                     let data = chart_data.await;
    //                     let token_labels = token_labels;
    //                     view! {
    //                         <LiquidityChart
    //                             debug=debug.into()
    //                             data=chart_data.into()
    //                             token_labels=token_labels.into()
    //                         />
    //                     }
    //                 })
    //             }}
    //         </Suspense>
    //     </div>
    // };
    //
    // #[cfg(not(feature = "charts"))]
    // let chart_element = view! {
    //     <div class="flex items-center justify-center w-full h-[160px]">
    //         <code>"Charts disabled"</code>
    //     </div>
    // };

    view! {
        <div class="grid auto-rows-min grid-cols-1 md:grid-cols-2 gap-4">
            // left side of the screen
            <div class="flex flex-col items-center gap-4">
                // my liquidity box
                <div class="block w-full bg-card border-solid border rounded-lg">
                    <div class="px-6 py-4">
                        <div class="w-full">
                            <h2 class="m-0 mb-2 text-base font-semibold">My Liquidity</h2>
                            <div class="flex justify-center items-center h-48">
                                <p class="text-muted-foreground text-sm">
                                    "You have no liquidity in this pool"
                                </p>
                            </div>
                        </div>
                    </div>
                    <hr class="m-0 border-b-1 border-border" />
                    <div class="px-6 py-4">
                        <h3 class="m-0 mb-2 text-sm font-medium">Deposit Balance</h3>
                        <div class="flex flex-col gap-2 items-center">
                            <div class="grid grid-cols-1 gap-4 w-full">
                                <div class="grid grid-cols-1 md:grid-cols-[1fr_14px_1fr] gap-4 w-full items-center">
                                    // token x deposit balance
                                    <div class="flex items-center bg-muted px-4 py-3 h-16 border border-none rounded-md">
                                        <div class="flex items-center flex-row flex-1 gap-2">
                                            <img
                                                // src=move || token_a()
                                                src=format!("{BASE_URL}{}", "/icons/amber.svg")
                                                class="w-8 h-8 rounded-full"
                                            />
                                            <div class="flex flex-col items-start gap-0">
                                                <p class="m-0 text-sm text-muted-foreground">
                                                    <b class="text-white">0</b>
                                                    " "
                                                    {move || token_a_symbol.get()}
                                                </p>
                                                <p class="m-0 text-sm text-muted-foreground">"$0"</p>
                                            </div>
                                        </div>
                                    </div>
                                    <div class="hidden md:block">
                                        <Plus size=14 color="white" />
                                    </div>
                                    // token y deposit balance
                                    <div class="flex items-center bg-muted px-4 py-3 h-16 border border-none rounded-md">
                                        <div class="flex items-center flex-row flex-1 gap-2">
                                            <img
                                                src=format!("{BASE_URL}{}", "/icons/uscrt.png")
                                                class="w-8 h-8 rounded-full"
                                            />
                                            <div class="flex flex-col items-start gap-0">
                                                <p class="m-0 text-sm text-muted-foreground">
                                                    <b class="text-white">0</b>
                                                    " "
                                                    {move || token_b_symbol.get()}
                                                </p>
                                                <p class="m-0 text-sm text-muted-foreground">"$0"</p>
                                            </div>
                                        </div>
                                    </div>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
                <div class="block w-full bg-card border-solid border rounded-lg">
                    <div class="px-6 py-4">
                        <div class="w-full">
                            <h2 class="m-0 mb-2 text-xl">"Fees Earned"</h2>
                            <div class="flex justify-center items-center h-48">
                                <p class="text-sm text-muted-foreground">
                                    "You have no fees earned"
                                </p>
                            </div>
                        </div>
                    </div>
                </div>

            </div>

            // right side of screen, moves to bottom on medium screens
            <div class="block px-5 py-4 w-full box-border space-y-5 mx-auto bg-card border-solid border rounded-lg max-h-max">
                // "Tab Group"
                <div class="flex gap-0.5 w-full p-[5px] bg-muted rounded-md">
                    // This preserves the query params when navigating to nested routes.
                    // TODO: this is terribly complicated. it works, but there must be a simpler way
                    <button
                        class="w-full h-8 py-1.5 bg-background rounded-sm"
                        on:click={
                            let navigate_ = navigate.clone();
                            let nav_options_ = nav_options.clone();
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
                                navigate_(&new_route, nav_options_.clone());
                            }
                        }
                    >
                        "Add Liquidity"
                    </button>
                    <button
                        class="w-full h-8 py-1.5 bg-muted text-muted-foreground rounded-sm"
                        on:click={
                            let navigate_ = navigate.clone();
                            let nav_options_ = nav_options.clone();
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
                                navigate_(&new_route, nav_options_.clone());
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
