use ammber_core::support::chain_query;
use ammber_core::{prelude::*, state::*, support::ILbPair, utils::addr_2_symbol, Error, BASE_URL};
use ammber_sdk::{
    contract_interfaces::lb_pair::{
        self, BinResponse, BinsResponse, LbPair, ReservesResponse, StaticFeeParametersResponse,
    },
    utils::u128_to_string_with_precision,
};
use codee::string::FromToStringCodec;
use cosmwasm_std::Uint256;
use keplr::Keplr;
use leptos::{ev, html, prelude::*, task::spawn_local};
use leptos_router::{components::A, hooks::use_params_map, nested_router::Outlet};
use leptos_use::storage::use_local_storage;
use liquidity_book::libraries::PriceHelper;
use lucide_leptos::{ArrowLeft, ExternalLink, Info, Settings2, X};
use tracing::{debug, error, info};

mod pool_analytics;
mod pool_browser;
mod pool_creator;
mod pool_manager;

pub use pool_analytics::PoolAnalytics;
pub use pool_browser::PoolBrowser;
pub use pool_creator::PoolCreator;
pub use pool_manager::{AddLiquidity, PoolManager, RemoveLiquidity};

use crate::state::PoolStateStoreFields;

#[component]
pub fn Pools() -> impl IntoView {
    info!("rendering <Pools/>");

    on_cleanup(move || {
        info!("cleaning up <Pools/>");
    });

    view! {
        <div class="pools-group">
            <Outlet />
        </div>
    }
}

// TODO: structure the pool data and provide_context for child components
#[component]
pub fn Pool() -> impl IntoView {
    info!("rendering <Pool/>");

    on_cleanup(move || {
        info!("cleaning up <Pool/>");
    });

    let endpoint = use_context::<Endpoint>().expect("endpoint context missing!");
    let keplr = use_context::<KeplrSignals>().expect("keplr signals context missing!");

    let params = use_params_map();

    // TODO: decide on calling these a/b or x/y
    let token_a = Signal::derive(move || {
        params
            .read_untracked()
            .get("token_a")
            .expect("Missing token_a URL param")
    });
    let token_b = Signal::derive(move || {
        params
            .read_untracked()
            .get("token_b")
            .expect("Missing token_b URL param")
    });
    let basis_points = Signal::derive(move || {
        params
            .read_untracked()
            .get("bps")
            .and_then(|value| value.parse::<u16>().ok())
            .expect("Missing bps URL param")
    });

    // slippage is in basis points. smallest supported slippage = 0.01%
    let (amount_slippage, set_amount_slippage, _) =
        use_local_storage::<u16, FromToStringCodec>("amount_slippage");
    let (price_slippage, set_price_slippage, _) =
        use_local_storage::<u16, FromToStringCodec>("price_slippage");

    // initialize local storage to default values
    if amount_slippage.get_untracked() == 0 {
        set_amount_slippage.set(50u16);
    }
    if price_slippage.get_untracked() == 0 {
        set_price_slippage.set(5u16);
    }

    // TODO: fine for now but I should convert the address to a Token, not only the symbol

    let token_a_symbol =
        AsyncDerived::new_unsync(move || async move { addr_2_symbol(token_a.get()).await });
    let token_b_symbol =
        AsyncDerived::new_unsync(move || async move { addr_2_symbol(token_b.get()).await });

    provide_context((token_a_symbol, token_b_symbol));

    let lb_pair: LocalResource<Result<LbPair, Error>> = LocalResource::new(move || async move {
        debug!("run lb_pair resource");

        let token_x = addr_2_contract(token_a.get()).await.unwrap();
        let token_y = addr_2_contract(token_b.get()).await.unwrap();
        let bin_step = basis_points.get();

        let storage = window()
            .local_storage()
            .expect("local storage not available?")
            .expect("local storage returned none?");

        let storage_key = format!("{}_{}_{}", token_x.address, token_y.address, bin_step);

        match storage.get_item(&storage_key) {
            Ok(None) => {
                let lb_pair = LB_FACTORY
                    .get_lb_pair_information(token_x, token_y, bin_step)
                    .await
                    .map(|lb_pair_information| lb_pair_information.lb_pair)
                    .inspect(|lb_pair| {
                        _ = storage
                            .set_item(&storage_key, &serde_json::to_string(&lb_pair).unwrap())
                    });

                lb_pair
            }
            Ok(Some(lb_pair)) => Ok(serde_json::from_str(&lb_pair).unwrap()),
            _ => todo!(),
        }
    });

    let active_id = LocalResource::new(move || {
        debug!("run active_id resource");
        async move {
            match lb_pair.await {
                Ok(pair) => ILbPair(pair.contract).get_active_id().await,
                Err(_) => Err(Error::generic("lb_pair resource is not available")),
            }
        }
    });

    let target_price = LocalResource::new(move || async move {
        active_id
            .await
            .ok()
            .and_then(|id| PriceHelper::get_price_from_id(id, basis_points.get()).ok())
            .and_then(|price| PriceHelper::convert128x128_price_to_decimal(price).ok())
            .map(|price| u128_to_string_with_precision(price.as_u128()))
    });

    provide_context(lb_pair);
    provide_context(active_id);

    // --- Store demonstration

    // A Store can be initialized with whatever starting values, and then a series of spawn_local
    // tasks can update the values of the stores asynchronously. These would all run once, before
    // the component mounts. See here for a great explanation:
    // https://book.leptos.dev/appendix_life_cycle.html?highlight=mount#component-life-cycle

    use crate::PoolState;

    let store = reactive_stores::Store::new(PoolState::default());

    spawn_local(async move {
        let token_x = addr_2_contract(token_a.get()).await.unwrap();
        let token_y = addr_2_contract(token_b.get()).await.unwrap();
        let bin_step = basis_points.get();

        let storage = window()
            .local_storage()
            .expect("local storage not available?")
            .expect("local storage returned none?");

        let storage_key = format!("{}_{}_{}", token_x.address, token_y.address, bin_step);

        let lb_pair = match storage.get_item(&storage_key) {
            Ok(None) => {
                let lb_pair = LB_FACTORY
                    .get_lb_pair_information(token_x, token_y, bin_step)
                    .await
                    .map(|lb_pair_information| lb_pair_information.lb_pair)
                    .inspect(|lb_pair| {
                        _ = storage
                            .set_item(&storage_key, &serde_json::to_string(&lb_pair).unwrap())
                    });

                lb_pair
            }
            Ok(Some(lb_pair)) => Ok(serde_json::from_str(&lb_pair).unwrap()),
            _ => todo!(),
        };

        store.lb_pair().set(lb_pair.unwrap())
    });

    spawn_local(async move {
        let active_id = match lb_pair.await {
            Ok(pair) => ILbPair(pair.contract).get_active_id().await,
            Err(_) => Err(Error::generic("lb_pair resource is not available")),
        };

        store.active_id().set(active_id.unwrap_or_default())
    });

    // --- end Store demonstration

    // TODO: (maybe) batch query
    let total_reserves =
        LocalResource::new(
            move || async move { ILbPair(lb_pair.await?.contract).get_reserves().await },
        );
    let static_fee_parameters = LocalResource::new(move || async move {
        ILbPair(lb_pair.await?.contract)
            .get_static_fee_parameters()
            .await
    });

    provide_context(total_reserves);
    provide_context(static_fee_parameters);

    // TODO: decide on LocalResource vs regular Signal
    let nearby_bins = RwSignal::<Result<Vec<BinResponse>, Error>>::new(Ok(vec![]));
    provide_context(nearby_bins);

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

        debug!("getting nearby bins reserves");

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

    let my_liquidity = LocalResource::new(move || {
        let url = endpoint;
        let chain_id = CHAIN_ID;

        // we have to access this signal sychronously to prevent the query from happening twice
        let active_id = active_id.get();

        async move {
            debug!("getting my_liquidity");

            if !keplr.enabled.get() {
                return Err(Error::KeplrDisabled);
            }

            if active_id.is_none() {
                return Err(Error::generic("active_id is missing"));
            }

            let lb_pair = lb_pair.await.map(|lb_pair| lb_pair.contract)?;

            // If the signal is None, the resource is still loading and we return early.
            // If we were to await it instead, this resource would spawn a second future when
            // active_id changes. Both futures would complete independently,
            // resulting in duplicate queries.
            let id = match active_id.as_deref() {
                Some(Ok(active_id)) => active_id,
                Some(Err(err)) => {
                    return Err(err.clone());
                }
                None => {
                    return Err(Error::generic("active_id is missing"));
                }
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

            let account = Keplr::get_key(&chain_id)
                .await
                .map(|key| key.bech32_address)?;

            let accounts = vec![account; ids.len()];

            let balances: Vec<Uint256> = ILbPair(lb_pair)
                .balance_of_batch(accounts, ids.clone())
                .await?;

            let combined: Vec<(u32, String)> = ids
                .iter()
                .zip(balances.iter())
                .map(|(&a, &b)| (a, b.to_string()))
                .collect();

            debug!("{:?}", combined);

            Ok((ids, balances))
        }
    });

    provide_context(my_liquidity);

    let settings_dialog_ref = NodeRef::<html::Dialog>::new();

    let toggle_settings = move |_: ev::MouseEvent| match settings_dialog_ref.get() {
        Some(dialog) => match dialog.open() {
            false => {
                _ = dialog.show();
            }
            true => {
                dialog.close();
            }
        },
        None => {
            _ = window().alert_with_message("Something is wrong!");
        }
    };

    let pool_address = move || {
        lb_pair
            .get()
            .map_or(String::new(), |wrapper| match wrapper.as_ref() {
                Ok(pair) => pair.contract.address.to_string(),
                Err(_) => String::new(),
            })
    };

    view! {
        <a
            href="/liquidity-book-leptos/pool"
            class="inline-flex gap-x-2 mb-3 items-center text-muted-foreground text-sm font-bold cursor-pointer no-underline"
        >
            <ArrowLeft size=14 />
            "Back to pools list"
        </a>

        // {move || store.lb_pair().get().contract.address.to_string()}
        // {move || store.active_id().get()}

        // page title with the token symbols
        <div class="md:h-10 flex flex-col md:flex-row items-start md:items-center gap-x-4 gap-y-3">
            <Suspense fallback=move || {
                view! { <div class="text-3xl font-bold">{token_a}" / "{token_b}</div> }
            }>
                // TODO: use accurate token icons here
                <div class="flex flex-row gap-x-2 items-center">
                    <div class="flex flex-row gap-x-2 items-center">
                        <img
                            src=format!("{BASE_URL}{}", "/icons/amber.svg")
                            class="w-9 h-9 rounded-full"
                        />
                        <h2 class="m-0 text-white text-3xl font-bold">
                            {move || Suspend::new(async move { token_a_symbol.await })}
                        </h2>
                    </div>

                    <h2 class="m-0 text-white text-3xl font-bold">"/"</h2>

                    <div class="flex flex-row gap-x-2 items-center">
                        <img
                            src=format!("{BASE_URL}{}", "/icons/uscrt.png")
                            class="w-9 h-9 rounded-full"
                        />
                        <h2 class="m-0 text-white text-3xl font-bold">
                            {move || Suspend::new(async move { token_b_symbol.await })}
                        </h2>
                    </div>
                </div>
            </Suspense>

            <div class="flex items-center gap-x-2 md:pl-4">
                <span class="text-sm text-foreground inline-flex font-semibold px-2.5 py-0.5 rounded-md border border-solid border-border">
                    {basis_points}" bps"
                </span>

                // Not bothering with Suspend. Accessing signal synchronously.
                <a
                    href=move || {
                        format!("https://testnet.ping.pub/secret/account/{}", pool_address())
                    }
                    target="_blank"
                    rel="noopener"
                    class="
                    inline-flex px-2.5 py-0.5 rounded-md border border-solid border-border
                    text-sm text-foreground font-semibold no-underline
                    "
                >
                    <div class="flex gap-1 items-center [&_svg]:-translate-y-[1px] [&_svg]:text-muted-foreground">
                        <div>{move || shorten_address(pool_address())}</div>
                        <ExternalLink size=14 />
                    </div>
                </a>
            </div>
        </div>
        <div class="my-4 flex flex-col-reverse md:flex-row justify-between gap-4">
            // NOTE: button style changes based on aria-current
            <div class="flex items-center gap-0.5 max-w-fit p-[5px] bg-muted rounded-md">
                <A href="manage">
                    <button
                        tabindex="-1"
                        class="py-1.5 px-3 rounded-sm bg-muted text-muted-foreground border-none h-8 w-[95px]"
                    >
                        "Manage"
                    </button>
                </A>
                <A href="analytics">
                    <button
                        tabindex="-1"
                        class="py-1.5 px-3 rounded-sm bg-muted text-muted-foreground border-none h-8 w-[95px]"
                    >
                        "Analytics"
                    </button>
                </A>
            </div>
            <div class="flex gap-2 md:gap-4 items-center justify-between sm:justify-normal">
                <div>
                    <p class="text-sm text-muted-foreground">"Current Price:"</p>
                    <Suspense fallback=|| {
                        view! {
                            <p class="text-base text-foreground font-semibold">"Loading..."</p>
                        }
                    }>
                        <p class="text-base text-foreground font-semibold">
                            {move || Suspend::new(async move {
                                format!(
                                    "1 {} = {} {}",
                                    token_a_symbol.await,
                                    target_price.await.unwrap_or("?".to_string()),
                                    token_b_symbol.await,
                                )
                            })}
                        </p>
                    </Suspense>
                </div>
                <div class="flex flex-row items-center gap-2 md:gap-4">
                    <div class="inline-flex items-center gap-0.5 p-[3px] bg-muted rounded-md">
                        <button class="py-2 px-3 rounded-md bg-background text-foreground text-sm border-none h-10 basis-1/2">
                            {move || token_a_symbol.get()}
                        </button>
                        <button
                            on:click=|_| alert("TODO: invert price")
                            class="py-2 px-3 rounded-md bg-muted text-muted-foreground text-sm border-none h-10 basis-1/2"
                        >
                            {move || token_b_symbol.get()}
                        </button>
                    </div>

                    <div class="relative">
                        <button
                            on:click=toggle_settings
                            class="inline-flex items-center justify-center
                            ml-auto w-[46px] h-[46px] text-foreground
                            rounded-md border border-solid border-border"
                        >
                            <Settings2 size=18 />
                        </button>
                        <AddLiquiditySettings
                            dialog_ref=settings_dialog_ref
                            toggle_menu=toggle_settings
                            amount_slippage=(amount_slippage, set_amount_slippage)
                            price_slippage=(price_slippage, set_price_slippage)
                        />
                    </div>
                </div>
            </div>
        </div>

        // container for the View Transition when navigating within the 'Pool' ParentRoute
        <div class="pool-tab-group">
            <Outlet />
        </div>
    }
}

#[component]
fn AddLiquiditySettings(
    dialog_ref: NodeRef<html::Dialog>,
    toggle_menu: impl Fn(ev::MouseEvent) + 'static,
    amount_slippage: (Signal<u16>, WriteSignal<u16>),
    price_slippage: (Signal<u16>, WriteSignal<u16>),
) -> impl IntoView {
    info!("rendering <SettingsMenu/>");

    view! {
        <div class="floating-menu">
            <dialog
                node_ref=dialog_ref
                class="z-40 mt-1.5 -mr-0 md:-mr-[124px] w-80 h-52 p-0 shadow-md bg-background text-foreground rounded-md border border-solid border-border"
            >
                <div class="relative flex flex-col z-auto">
                    <div class="flex justify-between items-center p-2 pl-3 text-popover-foreground border-0 border-b border-solid border-border">
                        <p class="m-0">"Settings"</p>
                        <button
                            autofocus
                            on:click=toggle_menu
                            class="appearance-none border-0
                            flex shrink-0 items-center justify-center w-6 h-6 p-1 box-border rounded-md
                            bg-transparent hover:bg-muted transition-colors duration-200 ease-standard
                            "
                        >
                            <X size=16 />
                        </button>
                    </div>
                    <div class="px-3 py-4 box-border">
                        <div class="flex flex-col items-start gap-4 w-full">
                            <div class="flex flex-col items-start gap-2 w-full">
                                <div class="flex flex-row items-center justify-between gap-2 w-full">
                                    <p class="text-muted-foreground text-sm m-0">
                                        "Slippage tolerance"
                                    </p>
                                    <div class="relative group focus-within:group">
                                        <div
                                            tabindex="0"
                                            class="text-foreground focus:outline-none"
                                        >
                                            <Info size=16 />
                                        </div>
                                        <div class="absolute w-[200px] z-50 bottom-full right-0 lg:right-1/2 translate-x-0 lg:translate-x-1/2
                                        bg-popover text-popover-foreground text-xs font-normal rounded-md border border-solid
                                        mb-1 p-2 invisible opacity-0 transition-opacity duration-100 ease-in
                                        group-hover:visible group-hover:opacity-100 group-focus-within:visible group-focus-within:opacity-100">
                                            "Your transaction will revert if the price changes unfavorably by more than this percentage."
                                        </div>
                                    </div>
                                </div>
                                <div class="flex flex-row items-center gap-2">
                                    <div class="flex flex-row items-center gap-1">
                                        <button
                                            on:click=move |_| amount_slippage.1.set(10)
                                            class="h-8 min-w-8 w-16 text-sm font-semibold bg-secondary text-secondary-foreground rounded-md"
                                        >
                                            "0.1%"
                                        </button>
                                        <button
                                            on:click=move |_| amount_slippage.1.set(50)
                                            class="h-8 min-w-8 w-16 text-sm font-semibold bg-secondary text-secondary-foreground rounded-md"
                                        >
                                            "0.5%"
                                        </button>
                                        <button
                                            on:click=move |_| amount_slippage.1.set(100)
                                            class="h-8 min-w-8 w-16 text-sm font-semibold bg-secondary text-secondary-foreground rounded-md"
                                        >
                                            "1%"
                                        </button>
                                    </div>
                                    <div class="w-full relative flex items-center isolate box-border">
                                        <input
                                            class="w-full box-border px-3 h-8 text-sm font-semibold bg-transparent text-popover-foreground rounded-md"
                                            inputmode="decimal"
                                            minlength="1"
                                            maxlength="79"
                                            type="text"
                                            pattern="^[0-9]*[.,]?[0-9]*$"
                                            prop:value=move || {
                                                amount_slippage.0.get() as f64 / 100.0
                                            }
                                            on:change=move |ev| {
                                                let value = event_target_value(&ev)
                                                    .parse::<f64>()
                                                    .unwrap_or(0.5);
                                                let value = (value * 100.0).round() as u16;
                                                amount_slippage.1.set(value)
                                            }
                                        />
                                        <div class="absolute right-0 top-0 w-8 h-8 z-[2] flex items-center justify-center text-popover-foreground">
                                            "%"
                                        </div>
                                    </div>
                                </div>
                            </div>
                            <div class="flex flex-col items-start gap-2">
                                <p class="text-muted-foreground text-sm m-0">
                                    "Transaction deadline"
                                </p>
                                <div class="w-full relative flex items-center isolate box-border">
                                    <input
                                        class="w-full box-border px-3 h-8 text-sm font-semibold bg-transparent text-popover-foreground rounded-md"
                                        inputmode="decimal"
                                        minlength="1"
                                        maxlength="79"
                                        type="text"
                                        pattern="^[0-9]*[.,]?[0-9]*$"
                                        prop:value=move || { price_slippage.0.get() }
                                        on:change=move |ev| {
                                            let value = event_target_value(&ev)
                                                .parse::<u16>()
                                                .unwrap_or(5u16);
                                            price_slippage.1.set(value)
                                        }
                                    />
                                    <div class="absolute right-0 top-0 min-w-fit h-8 mr-4 z-[2] flex items-center justify-center text-sm text-popover-foreground">
                                        "minutes"
                                    </div>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
            </dialog>
        </div>
    }
}
