use ammber_charts::{load_data, LiquidityChart, ReserveData};
use ammber_core::{prelude::*, state::*, support::ILbPair, Error, BASE_URL};
use ammber_sdk::contract_interfaces::lb_pair::LbPair;
use cosmwasm_std::Uint256;
use ethnum::U256;
use keplr::Keplr;
use leptos::either::Either;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::components::A;
use leptos_router::{
    hooks::{use_location, use_navigate, use_params_map},
    nested_router::Outlet,
    NavigateOptions,
};
use liquidity_book::libraries::math::uint256_to_u256::ConvertUint256;
use liquidity_book::libraries::PriceHelper;
use lucide_leptos::Plus;
use tracing::{debug, error, info};

mod add_liquidity;
mod remove_liquidity;

pub use add_liquidity::AddLiquidity;
pub use remove_liquidity::RemoveLiquidity;

#[component]
pub fn PoolManager() -> impl IntoView {
    info!("rendering <PoolManager/>");

    let endpoint = use_context::<Endpoint>().expect("endpoint context missing!");
    let keplr = use_context::<KeplrSignals>().expect("keplr signals context missing!");
    let token_map = use_context::<TokenMap>().expect("tokens context missing!");

    let lb_pair = use_context::<LocalResource<Result<LbPair, Error>>>()
        .expect("missing the LbPair resource context");
    let active_id = use_context::<LocalResource<Result<u32, Error>>>()
        .expect("missing the active_id resource context");

    let params = use_params_map();
    // TODO: decide on calling these a/b or x/y
    let token_a = move || {
        params
            .read_untracked()
            .get("token_a")
            .and_then(|token_address| TOKEN_MAP.get(&token_address))
            .expect("Missing token_a URL param")
    };
    let token_b = move || {
        params
            .read_untracked()
            .get("token_b")
            .and_then(|token_address| TOKEN_MAP.get(&token_address))
            .expect("Missing token_b URL param")
    };
    let basis_points = move || {
        params
            .read_untracked()
            .get("bps")
            .and_then(|value| value.parse::<u16>().ok())
            .expect("Missing bps URL param")
    };

    let token_a_symbol = Signal::derive(move || token_a().symbol.clone());
    let token_b_symbol = Signal::derive(move || token_b().symbol.clone());

    let selected_tab = create_rw_signal("add");

    // let my_liquidity =
    //     RwSignal::<Result<(Vec<u32>, Vec<Uint256>), Error>>::new(Ok((vec![], vec![])));
    //
    // just realized this whole thing won't work because we need this to react to changes in
    // keplr.enabled
    // spawn_local(async move {
    //     if !keplr.enabled.get() {
    //         return;
    //     }
    //
    //     let lb_pair_result = lb_pair.await;
    //     let id_result = active_id.await;
    //
    //     let lb_pair = match lb_pair_result {
    //         Ok(lb_pair) => lb_pair.contract,
    //         Err(err) => {
    //             error!("Failed to get LB pair: {:?}", err);
    //             my_liquidity.set(Err(err.into()));
    //             return;
    //         }
    //     };
    //
    //     let id = match id_result {
    //         Ok(id) => id,
    //         Err(err) => {
    //             error!("Failed to get active ID: {:?}", err);
    //             my_liquidity.set(Err(err.into()));
    //             return;
    //         }
    //     };
    //
    //     let mut ids = vec![];
    //     let radius = 49;
    //
    //     for i in 0..(radius * 2 + 1) {
    //         let offset_id = if i < radius {
    //             id - (radius - i) as u32 // Subtract for the first half
    //         } else {
    //             id + (i - radius) as u32 // Add for the second half
    //         };
    //
    //         ids.push(offset_id);
    //     }
    //
    //     debug!("{:?}", ids);
    //
    //     // let account = Keplr::get_key(&chain_id)
    //     //     .await
    //     //     .map(|key| key.bech32_address)?;
    //     //
    //     // let accounts = vec![account; ids.len()];
    //     //
    //     // let balances: Vec<Uint256> = ILbPair(lb_pair)
    //     //     .balance_of_batch(accounts, ids.clone())
    //     //     .await?;
    //     //
    //     // debug!("{:?}", balances);
    //     //
    //     // Ok((ids, balances))
    // });

    let my_liquidity = use_context::<LocalResource<Result<(Vec<u32>, Vec<Uint256>), Error>>>()
        .expect("missing the my_liquidity context");

    fn fixed_point_to_f64(value: Uint256) -> f64 {
        let decimal_value = PriceHelper::convert128x128_price_to_decimal(value.uint256_to_u256())
            .unwrap_or_default();

        let decimal_factor = U256::from(10u64.pow(18));

        let integer_part = decimal_value / decimal_factor;
        let fractional_part = decimal_value % decimal_factor;

        let int_f64 = integer_part.as_f64();
        let frac_f64 = fractional_part.as_f64() / 1_000_000_000_000_000_000.0;

        int_f64 + frac_f64
    }

    // TODO: make this async?
    let chart_data = Signal::derive(move || {
        if let Some(Ok((ids, amounts))) = my_liquidity.get().as_deref() {
            ids.iter()
                .cloned()
                .zip(amounts)
                .map(|(id, amount)| ReserveData::new(id.into(), fixed_point_to_f64(*amount), 0.0))
                .collect()
        } else {
            vec![]
        }
    });

    #[cfg(feature = "charts")]
    {
        let debug = RwSignal::new(false);
        let token_labels = Signal::derive(move || {
            let token_x = token_a_symbol.get();
            let token_y = token_b_symbol.get();

            (token_x, token_y)
        });
        // let chart_data = load_data();

        let chart_element = view! {
            <div class="flex justify-center w-full">
                // TODO: Suspense is not necessary here. Use the Show component instead?
                // fallback view is the second one
                <Suspense fallback=|| {
                    view! { "Loading..." }
                }>
                    {move || {
                        Suspend::new(async move {
                            let data = chart_data;
                            let token_labels = token_labels;
                            if !data.get().is_empty() {
                                Either::Left(
                                    view! {
                                        <LiquidityChart
                                            debug=debug.into()
                                            data=chart_data.into()
                                            token_labels=token_labels.into()
                                        />
                                    },
                                )
                            } else {
                                Either::Right(
                                    view! {
                                        <p class="text-muted-foreground text-sm">
                                            "You have no liquidity in this pool"
                                        </p>
                                    },
                                )
                            }
                        })
                    }}
                </Suspense>
            </div>
        };
    }

    #[cfg(not(feature = "charts"))]
    let chart_element = view! {
        <div class="flex items-center justify-center w-full h-[160px]">
            <code>"Charts disabled"</code>
        </div>
    };

    view! {
        <div class="grid auto-rows-min grid-cols-1 md:grid-cols-2 gap-4">
            // left side of the screen
            <div class="flex flex-col items-center gap-4">
                // my liquidity box
                <div class="block w-full bg-card border-solid border rounded-lg">
                    <div class="px-6 py-4">
                        <div class="w-full">
                            <h2 class="m-0 mb-2 text-base font-semibold">My Liquidity</h2>
                            <div class="flex justify-center items-center h-48">{chart_element}</div>
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

            // Right side of screen, moves to bottom on medium screens
            <div class="block px-5 py-4 w-full box-border space-y-5 mx-auto bg-card border-solid border rounded-lg max-h-max">
                // Tab Group
                <div class="flex gap-0.5 w-full p-[5px] bg-muted rounded-md">
                    <button
                        class="py-1.5 px-3 rounded-sm border-none h-8 w-full"
                        class=(
                            ["bg-background", "text-foreground"],
                            move || selected_tab.get() == "add",
                        )
                        class=(
                            ["bg-muted", "text-muted-foreground"],
                            move || selected_tab.get() != "add",
                        )

                        on:click=move |_| selected_tab.set("add")
                    >
                        "Add Liquidity"
                    </button>
                    <button
                        class="py-1.5 px-3 rounded-sm border-none h-8 w-full"
                        class=(
                            ["bg-background", "text-foreground"],
                            move || selected_tab.get() == "remove",
                        )
                        class=(
                            ["bg-muted", "text-muted-foreground"],
                            move || selected_tab.get() != "remove",
                        )
                        on:click=move |_| selected_tab.set("remove")
                    >
                        "Remove Liquidity"
                    </button>
                </div>
                // Container for the component based on selected tab
                <div class="liquidity-group">
                    // Show component based on selected tab
                    {move || match selected_tab.get() {
                        "add" => view! { <AddLiquidity /> }.into_any(),
                        "remove" => view! { <RemoveLiquidity /> }.into_any(),
                        _ => {
                            view! {
                                // Default case
                                <AddLiquidity />
                            }
                                .into_any()
                        }
                    }}
                </div>
            </div>
        </div>
    }
}
