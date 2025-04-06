use crate::BASE_URL;
use ammber_sdk::utils::u128_to_string_with_precision;
use liquidity_book::libraries::PriceHelper;
use lucide_leptos::{Info, Settings2, X};

mod pool_analytics;
mod pool_browser;
mod pool_creator;
mod pool_manager;

pub use pool_analytics::PoolAnalytics;
pub use pool_browser::PoolBrowser;
pub use pool_creator::PoolCreator;
pub use pool_manager::{AddLiquidity, PoolManager, RemoveLiquidity};

use leptos::ev;
use leptos::html;
use leptos::prelude::*;
use leptos_router::nested_router::Outlet;
use tracing::info;

#[component]
pub fn Pools() -> impl IntoView {
    info!("rendering <Pools/>");

    on_cleanup(move || {
        info!("cleaning up <Pools/>");
    });

    // Resources in this component can be shared with all children, so they only run once and are
    // persistent. This is just an example:
    // let resource = LocalResource::new(move || {
    //     SendWrapper::new(async move {
    //         QueryMsg::GetNumberOfLbPairs {}
    //             .do_query(&LB_FACTORY)
    //             .await
    //     })
    // });

    // provide_context(resource);

    view! {
        <div class="pools-group">
            <Outlet />
        </div>
    }
}

// NOTE: If the Router gets complicated enough, it's possible to split it up like this:

// use leptos_router::{
//     components::{ParentRoute, Route},
//     MatchNestedRoutes,
// };
// use leptos_router_macro::path;
//
// #[component]
// pub fn PoolRoutes() -> impl MatchNestedRoutes<Dom> + Clone {
//     view! {
//         <ParentRoute path=path!("/pool") view=Pool>
//             <Route path=path!("/") view=PoolBrowser />
//             <Route path=path!("/create") view=PoolCreator />
//             <ParentRoute path=path!("/:token_a/:token_b/:bps") view=PoolManager>
//                 <Route path=path!("/") view=|| () />
//                 <Route path=path!("/add") view=AddLiquidity />
//                 <Route path=path!("/remove") view=RemoveLiquidity />
//             </ParentRoute>
//         </ParentRoute>
//     }
//     .into_inner()
// }

#[component]
pub fn Pool() -> impl IntoView {
    info!("rendering <Pool/>");

    use crate::{
        error::Error,
        prelude::*,
        support::{chain_query, ILbPair, Querier, COMPUTE_QUERIER},
    };
    use ammber_sdk::contract_interfaces::lb_pair::{BinResponse, LbPair};
    use batch_query::{
        msg_batch_query, parse_batch_query, BatchItemResponseStatus, BatchQueryParams,
        BatchQueryParsedResponse, BatchQueryResponse, BATCH_QUERY_ROUTER,
    };
    use codee::string::FromToStringCodec;
    use cosmwasm_std::{Addr, ContractInfo};
    use leptos::prelude::*;
    use leptos_router::{components::A, hooks::use_params_map, nested_router::Outlet};
    use leptos_use::storage::use_local_storage;
    use lucide_leptos::{ArrowLeft, ExternalLink};
    use secret_toolkit_snip20::TokenInfoResponse;
    use send_wrapper::SendWrapper;
    use serde::Serialize;
    use tracing::{debug, info, trace};

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

    // slippage is in basis points. smallest supported slippage = 0.01%
    let (amount_slippage, set_amount_slippage, _) =
        use_local_storage::<u16, FromToStringCodec>("amount_slippage");
    let (price_slippage, set_price_slippage, _) =
        use_local_storage::<u16, FromToStringCodec>("price_slippage");

    if amount_slippage.get() == 0 {
        set_amount_slippage.set(50u16);
    }
    if price_slippage.get() == 0 {
        set_price_slippage.set(5u16);
    }

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
            if let Some(ref display_name) = token.display_name {
                return display_name.clone();
            } else {
                return token.symbol.clone();
            }
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

    provide_context((token_a_symbol, token_b_symbol));

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
    let lb_pair: Resource<Result<LbPair, Error>> = Resource::new(
        move || (token_a(), token_b(), basis_points()),
        |(token_a, token_b, basis_points)| {
            debug!("run lb_pair resource");
            SendWrapper::new(async move {
                let token_x = addr_2_contract(token_a).await.unwrap();
                let token_y = addr_2_contract(token_b).await.unwrap();
                let bin_step = basis_points;

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
                                _ = storage.set_item(
                                    &storage_key,
                                    &serde_json::to_string(&lb_pair).unwrap(),
                                )
                            });

                        // let _ = storage
                        //     .set_item(&storage_key, &serde_json::to_string(&lb_pair).unwrap());

                        lb_pair
                    }
                    Ok(Some(lb_pair)) => Ok(serde_json::from_str(&lb_pair).unwrap()),
                    _ => todo!(),
                }
            })
        },
    );

    // NOTE: For some reason, this always runs twice. To avoid double querying, do nothing if
    //       lb_pair is None, instead of await-ing it. I wonder if it has something to do with all
    //       the SendWrappers involved...
    let active_id = Resource::new(
        move || lb_pair.get(),
        |lb_pair| {
            debug!("run active_id resource");
            async move {
                if let Some(Ok(lb_pair)) = lb_pair {
                    ILbPair(lb_pair.contract).get_active_id().await
                } else {
                    Err(Error::generic("lb_pair resource is not available yet"))
                }
            }
        },
    );

    let target_price = AsyncDerived::new(move || async move {
        active_id
            .await
            .ok()
            .and_then(|id| PriceHelper::get_price_from_id(id, basis_points()).ok())
            .and_then(|price| PriceHelper::convert128x128_price_to_decimal(price).ok())
            .map(|price| u128_to_string_with_precision(price.as_u128()))
    });

    provide_context(active_id);
    provide_context(lb_pair);

    // TODO: decide if these queries should go here or in the analytics component
    let total_reserves = Resource::new(
        move || lb_pair.get(),
        move |_| async move { ILbPair(lb_pair.await?.contract).get_reserves().await },
    );
    let static_fee_parameters = Resource::new(
        move || lb_pair.get(),
        move |_| async move {
            ILbPair(lb_pair.await?.contract)
                .get_static_fee_parameters()
                .await
        },
    );
    provide_context(total_reserves);
    provide_context(static_fee_parameters);

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

    view! {
        <a
            href="/liquidity-book-leptos/pool"
            class="inline-flex gap-x-2 mb-3 items-center text-muted-foreground text-sm font-bold cursor-pointer no-underline"
        >
            <ArrowLeft size=14 />
            "Back to pools list"
        </a>

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
                <a
                    href="about:blank"
                    target="_blank"
                    rel="noopener"
                    class="
                    inline-flex px-2.5 py-0.5 rounded-md border border-solid border-border
                    text-sm text-foreground font-semibold no-underline
                    "
                >
                    <div class="flex gap-1 items-center [&_svg]:-translate-y-[1px] [&_svg]:text-muted-foreground">
                        <div>
                            {move || {
                                lb_pair
                                    .get()
                                    .and_then(Result::ok)
                                    .map(|x| shorten_address(x.contract.address))
                            }}
                        </div>
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
