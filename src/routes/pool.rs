use crate::components::AnimatedShow;
use crate::state::*;
use crate::CHAIN_ID;
use leptos_router::NavigateOptions;
use rsecret::query::compute::ComputeQuerier;
use rsecret::secret_network_client::CreateQuerierOptions;
use send_wrapper::SendWrapper;
use shade_protocol::liquidity_book::lb_pair::ActiveIdResponse;
use shade_protocol::liquidity_book::lb_pair::BinResponse;
use shade_protocol::liquidity_book::lb_pair::NextNonEmptyBinResponse;
use shade_protocol::liquidity_book::lb_pair::TotalSupplyResponse;
use shade_protocol::{
    c_std::{Addr, ContractInfo},
    contract_interfaces::liquidity_book::lb_pair::LBPair,
    // liquidity_book::lb_pair::ReservesResponse,
    swap::core::{TokenAmount, TokenType},
};
use std::time::Duration;

use leptos::prelude::*;
use leptos_router::components::{ParentRoute, Route, A};
use leptos_router::hooks::use_params;
use leptos_router::hooks::use_params_map;
use leptos_router::hooks::use_query_map;
use leptos_router::location::State;
use leptos_router::nested_router::Outlet;
use leptos_router::params::Params;
use leptos_router::{MatchNestedRoutes, ParamSegment, StaticSegment};
use leptos_router_macro::path;
use tracing::{debug, info};

// /pool/0x49d5c2bdffac6ce2bfdb6640f4f80f226bc10bab/AVAX/10
// /pool/<tokenX_address>/<tokenY_address>/<basis_points>

// path=(StaticSegment("posts"), ParamSegment("id"))
// or
// path=path!("posts/:id")

// #[component]
// pub fn PoolRoutes() -> impl MatchNestedRoutes<Dom> + Clone {
//     view! {
//       <ParentRoute path=StaticSegment("pool") view=Pool>
//           <Route path=StaticSegment("") view=|| "empty Outlet"/>
//           <Route path=StaticSegment("create") view=PoolCreator/>
//           // <Route path=path!("manage") view=PoolManager/>
//           // <Route path=path!("stats") view=PoolStats/>
//       </ParentRoute>
//     }
//     .into_inner()
// }

#[component]
pub fn Pool() -> impl IntoView {
    info!("rendering <Pool/>");

    on_cleanup(move || {
        info!("cleaning up <Pool/>");
    });

    use crate::liquidity_book::constants::addrs::{LB_FACTORY_CONTRACT, LB_PAIR_CONTRACT};
    use crate::liquidity_book::Querier;
    use shade_protocol::contract_interfaces::liquidity_book::lb_factory::QueryMsg;

    let resource = LocalResource::new(move || {
        SendWrapper::new(async move { QueryMsg::GetNumberOfLbPairs {}.do_query().await })
    });

    provide_context(resource);

    view! {
        <div class="p-2">
            <Outlet />
        </div>
    }
}

#[component]
pub fn PoolBrowser() -> impl IntoView {
    info!("rendering <PoolBrowser/>");

    on_cleanup(move || {
        info!("cleaning up <PoolBrowser/>");
    });

    let pair_1 = LBPair {
        token_x: TokenType::CustomToken {
            contract_addr: Addr::unchecked("foo"),
            token_code_hash: "code_hash".to_string(),
        },
        token_y: TokenType::CustomToken {
            contract_addr: Addr::unchecked("bar"),
            token_code_hash: "code_hash".to_string(),
        },
        bin_step: 100,
        contract: ContractInfo {
            address: Addr::unchecked("secret1pt5nd3fuevamy5lqcv53jqsvytspmknanf5c28"),
            code_hash: "9768cfd5753a7fa2b51b30a3fc41632df2b3bc31801dece2d6111f321a3e4252"
                .to_string(),
        },
    };

    let pair_2 = LBPair {
        token_x: TokenType::CustomToken {
            contract_addr: Addr::unchecked("bar"),
            token_code_hash: "code_hash".to_string(),
        },
        token_y: TokenType::CustomToken {
            contract_addr: Addr::unchecked("baz"),
            token_code_hash: "code_hash".to_string(),
        },
        bin_step: 100,
        contract: ContractInfo {
            address: Addr::unchecked("a second LB Pair"),
            code_hash: "9768cfd5753a7fa2b51b30a3fc41632df2b3bc31801dece2d6111f321a3e4252"
                .to_string(),
        },
    };

    // TODO: query for the pools
    let pools = vec![pair_1, pair_2];

    let resource = use_context::<LocalResource<String>>().expect("Context missing!");

    view! {
        <div class="text-3xl font-bold">"Pool"</div>
        <div class="text-sm text-neutral-400">"Provide liquidity and earn fees."</div>

        <h3 class="mb-1">"Existing Pools"</h3>
        <ul>
            {pools
                .into_iter()
                .map(|n| {
                    view! {
                        <li>
                            <a href=format!(
                                "/pool/{}/{}/{}",
                                match n.token_x {
                                    TokenType::CustomToken { contract_addr, .. } => {
                                        contract_addr.to_string()
                                    }
                                    TokenType::NativeToken { denom } => denom,
                                },
                                match n.token_y {
                                    TokenType::CustomToken { contract_addr, .. } => {
                                        contract_addr.to_string()
                                    }
                                    TokenType::NativeToken { denom } => denom,
                                },
                                n.bin_step,
                            )>{n.contract.address.to_string()}</a>
                        </li>
                    }
                })
                .collect_view()}
        </ul>

        // <h3>{move || resource.get()}</h3>

        <div class="mt-4">
            <A href="/pool/create">
                <button class="p-1">"Create New Pool"</button>
            </A>
        </div>
    }
}

#[component]
pub fn PoolManager() -> impl IntoView {
    info!("rendering <PoolManager/>");

    let keplr = use_context::<KeplrSignals>().expect("keplr signals context missing!");
    let wasm_client = use_context::<WasmClient>().expect("wasm client context missing!");
    let token_map = use_context::<TokenMap>().expect("tokens context missing!");

    // whenever the key store changes, this will re-set 'is_keplr_enabled' to true, triggering a
    // reload of everything subscribed to that signal
    let keplr_keystorechange_handle =
        window_event_listener_untyped("keplr_keystorechange", move |_| {
            keplr.enabled.set(true);
        });

    on_cleanup(move || {
        info!("cleaning up <PoolManager/>");
        keplr_keystorechange_handle.remove()
    });

    let params = use_params_map();
    let token_a = move || {
        params
            .read()
            .get("token_a")
            .unwrap_or_else(|| "foo".to_string())
    };
    let token_b = move || {
        params
            .read()
            .get("token_b")
            .unwrap_or_else(|| "bar".to_string())
    };
    let basis_points = move || {
        params
            .read()
            .get("basis_points")
            .unwrap_or_else(|| "100".to_string())
    };

    // let resource = Resource::new(
    //     move || (token_a(), token_b(), basis_points()),
    //     move |(token_a, token_b, basis_points)| {
    //         SendWrapper::new(async move {
    //             let encryption_utils = secretrs::EncryptionUtils::new(None, CHAIN_ID).unwrap();
    //             // TODO: revisit this. url is not needed, EncryptionUtils should be a trait
    //             let options = CreateQuerierOptions {
    //                 url: "https://grpc.mainnet.secretsaturn.net",
    //                 chain_id: CHAIN_ID,
    //                 encryption_utils,
    //             };
    //             let compute = ComputeQuerier::new(wasm_client.get(), options);
    //             // TODO:
    //             let query = format!("{}, {}, {}", token_a, token_b, basis_points);
    //             debug!("{query}");
    //
    //             let result = compute.address_by_label("amber-24").await;
    //             result.map_err(Into::<crate::Error>::into)
    //         })
    //     },
    // );
    // let (pending, set_pending) = signal(false);

    use crate::liquidity_book::contract_interfaces::lb_pair::QueryMsg;
    use crate::liquidity_book::Querier;

    // TODO: change contract_interfaces to not use u128 in response types.

    #[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
    pub struct ReservesResponse {
        pub reserve_x: String,
        pub reserve_y: String,
    }

    #[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
    pub struct BinResponse {
        pub bin_id: u32,
        pub bin_reserve_x: String,
        pub bin_reserve_y: String,
    }

    // TODO: Move these to a separate module
    async fn query_reserves() -> ReservesResponse {
        let response = QueryMsg::GetReserves {}.do_query().await;
        serde_json::from_str::<ReservesResponse>(&response)
            .expect("Failed to deserialize ReservesResponse!")
    }
    async fn query_active_id() -> u32 {
        let response = QueryMsg::GetActiveId {}.do_query().await;
        serde_json::from_str::<ActiveIdResponse>(&response)
            .expect("Failed to deserialize ActiveIdResponse!")
            .active_id
    }
    async fn query_bin_reserves(id: u32) -> BinResponse {
        let response = QueryMsg::GetBinReserves { id }.do_query().await;
        debug!("{:?}", response);
        serde_json::from_str::<BinResponse>(&response).expect("Failed to deserialize BinResponse!")
    }
    async fn query_next_non_empty_bin(id: u32) -> u32 {
        let response = QueryMsg::GetNextNonEmptyBin {
            swap_for_y: true,
            id,
        }
        .do_query()
        .await;
        debug!("{:?}", response);
        serde_json::from_str::<NextNonEmptyBinResponse>(&response)
            .expect("Failed to deserialize BinResponse!")
            .next_id
    }
    // TODO: Figure out why this number is so huge, for example:
    //       12243017097593870802128434755484640756287535340
    async fn query_total_supply(id: u32) -> String {
        let response = QueryMsg::TotalSupply { id }.do_query().await;
        debug!("{:?}", response);
        serde_json::from_str::<TotalSupplyResponse>(&response)
            .expect("Failed to deserialize TotalSupplyResponse!")
            .total_supply
            .to_string()
    }

    let total_reserves = Resource::new(
        // move || (token_a(), token_b(), basis_points()),
        move || (),
        move |_| SendWrapper::new(async move { query_reserves().await }),
    );
    let active_id = Resource::new(
        move || (),
        move |_| SendWrapper::new(async move { query_active_id().await }),
    );
    // TODO: Figure out how to prevent these from running twice
    let bin_reserves = Resource::new(
        move || active_id.track(),
        move |_| SendWrapper::new(async move { query_bin_reserves(active_id.await).await }),
    );
    let next_non_empty_bin = Resource::new(
        move || active_id.track(),
        move |_| SendWrapper::new(async move { query_next_non_empty_bin(active_id.await).await }),
    );
    let bin_total_supply = Resource::new(
        move || active_id.track(),
        move |_| SendWrapper::new(async move { query_total_supply(active_id.await).await }),
    );

    view! {
        <a
            href="/pool"
            class="block text-neutral-200/50 text-sm font-bold cursor-pointer no-underline"
        >
            "🡨 Back to pools list"
        </a>
        <div class="flex flex-wrap py-2 items-center gap-x-4 gap-y-2">
            <div class="text-3xl font-bold">{token_a}" / "{token_b}</div>
            <div class="flex items-center gap-x-4">
                <div class="text-md font-bold p-1 outline outline-1 outline-offset-2 outline-neutral-500/50">
                    {basis_points}" bps"
                </div>
                <a href="about:blank" target="_blank" rel="noopener">
                    <div class="text-md font-bold p-1 outline outline-1 outline-offset-2 outline-neutral-500/50">
                        "secret123...xyz ↗"
                    </div>
                </a>
            </div>
        </div>

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

        <details class="text-neutral-300 font-bold">
            <summary class="text-lg cursor-pointer">Pool Details</summary>
            <ul class="my-1 font-normal text-base text-neutral-200 ">
                <Suspense fallback=|| view! { <div>"Loading Total Reserves..."</div> }>
                    <li>
                        "Total Reserves: "<span  tabindex="0" class="cursor-pointer text-white peer">"🛈"</span>
                        <li class="list-none text-sm font-bold text-violet-400 peer-focus:block hidden"> "🛈 Reserves may be in reverse order" </li>
                        {move || Suspend::new(async move {
                            let reserves = total_reserves.await;
                            view! {
                                <li class="pl-4 list-none">"reserve_x: "{reserves.reserve_x}</li>
                                <li class="pl-4 list-none">"reserve_y: "{reserves.reserve_y}</li>
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
                            let reserves = bin_reserves.await;
                            view! {
                                <li class="pl-4 list-none">"bin_reserve_x: "{reserves.bin_reserve_x}</li>
                                <li class="pl-4 list-none">"bin_reserve_y: "{reserves.bin_reserve_y}</li>
                            }
                        })}
                    </li>
                </Suspense>
                <Suspense fallback=|| view! { <div>"Loading Next Non-Empty Bin..."</div> }>
                    <li>
                        "Next Non-Empty Bin ID: " {move || Suspend::new(async move { next_non_empty_bin.await })}
                    </li>
                </Suspense>
            </ul>
        </details>

        <div class="flex gap-4 py-2">
            <A href="add">
                <button>Add Liquidity</button>
            </A>

            <A href="remove">
                <button>Remove Liquidity</button>
            </A>

            <A href="">
                <button>Nevermind</button>
            </A>
        </div>

        <Outlet />
    }
}

#[component]
pub fn AddLiquidity() -> impl IntoView {
    info!("rendering <AddLiquidity/>");

    let params = use_params_map();
    let token_a = move || params.read().get("token_a").unwrap_or("foo".to_string());
    let token_b = move || params.read().get("token_b").unwrap_or("bar".to_string());
    let basis_points = move || params.read().get("bps").unwrap_or("100".to_string());

    let query = use_query_map();
    let price = move || query.read().get("price").unwrap_or("radius".to_string());

    let (token_x_amount, set_token_x_amount) = signal("0".to_string());
    let (token_y_amount, set_token_y_amount) = signal("0".to_string());
    let (liquidity_shape, set_liquidity_shape) = signal("curve".to_string());

    let (target_price, set_target_price) = signal("1.00".to_string());
    let (radius, set_radius) = signal("5".to_string());

    // Effect::new(move || debug!("{:?}", token_x_amount.get()));
    // Effect::new(move || debug!("{:?}", token_y_amount.get()));
    // Effect::new(move || debug!("{:?}", liquidity_shape.get()));

    // let navigate = leptos_router::hooks::use_navigate();
    // let nav_options = NavigateOptions {
    //     resolve: true,
    //     replace: true,
    //     scroll: false,
    //     state: leptos_router::location::State::new(None),
    // };

    view! {
        <div class="container max-w-xs py-2 space-y-2">
            <div class="text-xl font-semibold">Deposit Liquidity</div>
            <div class="flex items-center gap-2">
                <div class="basis-1/3 text-md text-ellipsis overflow-hidden">{token_a}</div>
                <input
                    class="p-1 basis-2/3"
                    type="number"
                    placeholder="Enter Amount"
                    on:change=move |ev| set_token_x_amount.set(event_target_value(&ev))
                />
            </div>
            <div class="flex items-center gap-2">
                <div class="basis-1/3 text-md text-ellipsis line-clamp-1">{token_b}</div>
                <input
                    class="p-1 basis-2/3"
                    type="number"
                    placeholder="Enter Amount"
                    on:change=move |ev| set_token_y_amount.set(event_target_value(&ev))
                />
            </div>
            <div class="text-xl font-semibold !mt-6">Choose Liquidity Shape</div>
            <select
                class="block p-1"
                on:change=move |ev| set_liquidity_shape.set(event_target_value(&ev))
            >
                // on:input=move |_ev| navigate("&shape=curve", nav_options.clone())
                <option value="curve">"Curve"</option>
                <option value="uniform">Spot/Uniform</option>
                <option value="bid-ask">Bid-Ask</option>
            </select>
            <div class="flex items-center gap-2 !mt-6">
                <div class="text-xl font-semibold mr-auto">Price</div>
                <A href="?price=range">
                    <button>By Range</button>
                </A>
                <A href="?price=radius">
                    <button>By Radius</button>
                </A>
            </div>
            <Show when=move || price() == "range">
                <div class="font-mono">"todo!()"</div>
            </Show>
            <Show when=move || price() == "radius">
                <div class="flex items-center gap-2">
                    <div class="basis-1/3">"Target Price:"</div>
                    <input
                        class="p-1 basis-2/3"
                        type="number"
                        placeholder="Enter Target Price"
                        value="1.00"
                        min="0"
                        // prop:value=move|| target_price.get()
                        on:change=move |ev| set_target_price.set(event_target_value(&ev))
                    />
                </div>
                <div class="flex items-center gap-2">
                    <div class="basis-1/3">"Radius:"</div>
                    <input
                        class="p-1 basis-2/3"
                        type="number"
                        placeholder="Enter Bin Radius"
                        value="5"
                        min="0"
                        // prop:value=move|| radius.get()
                        on:change=move |ev| set_radius.set(event_target_value(&ev))
                    />
                </div>
                // <div class="grid grid-cols-2 items-center gap-2" >
                // <div class="">"Target Price:"</div>
                // <input
                // class="p-1"
                // type="number"
                // placeholder="Enter Amount"
                // on:change=move |ev| set_target_price.set(event_target_value(&ev))
                // />
                // <div class="">"Radius:"</div>
                // <input
                // class="p-1"
                // type="number"
                // placeholder="Enter Amount"
                // on:change=move |ev| set_radius.set(event_target_value(&ev))
                // />
                // </div>

                <button class="w-full p-1 !mt-6">"Add Liquidity"</button>
            </Show>
        </div>
    }
}

#[component]
pub fn RemoveLiquidity() -> impl IntoView {
    info!("rendering <RemoveLiquidity/>");

    view! { <div>TODO</div> }
}

#[component]
pub fn PoolCreator() -> impl IntoView {
    info!("rendering <PoolCreator/>");

    on_cleanup(move || {
        info!("cleaning up <PoolCreator/>");
    });

    let (token_x, set_token_x) = signal("TOKENX".to_string());
    let (token_y, set_token_y) = signal("TOKENY".to_string());
    let (bin_step, set_bin_step) = signal("100".to_string());
    let (active_price, set_active_price) = signal("1".to_string());

    let create_pool = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let token_x = token_x.get();
        let token_y = token_y.get();
        let bin_step = bin_step.get();
        let active_price = active_price.get();

        debug!("{}", token_x);
        debug!("{}", token_y);
        debug!("{}", bin_step);
        debug!("{}", active_price);

        // ...
    };

    view! {
        <a
            href="/pool"
            class="block text-neutral-200/50 text-sm font-bold cursor-pointer no-underline"
        >
            "🡨 Back to pools list"
        </a>
        <div class="py-3 text-2xl font-bold text-center sm:text-left">"Create New Pool"</div>
        <form class="container max-w-xs space-y-4 py-1 mx-auto sm:mx-0" on:submit=create_pool>
            <label class="block">
                "Select Token"
                <select
                    class="block p-1 font-bold w-full max-w-xs"
                    name="token_x"
                    title="Select Token"
                    on:input=move |ev| set_token_x.set(event_target_value(&ev))
                >
                    <option value="TOKENX">"TOKEN X"</option>
                    <option value="sSCRT">sSCRT</option>
                    <option value="SHD">SHD</option>
                    <option value="AMBER">AMBER</option>
                    <option value="SILK">SILK</option>
                </select>
            </label>
            <label class="block">
                "Select Quote Asset"
                <select
                    class="block p-1 font-bold w-full max-w-xs"
                    name="token_y"
                    title="Select Quote Asset"
                    on:input=move |ev| set_token_y.set(event_target_value(&ev))
                >
                    <option value="TOKENY">"TOKEN Y"</option>
                    <option value="sSCRT">sSCRT</option>
                    <option value="stkd-SCRT">stkd-SCRT</option>
                    <option value="SILK">SILK</option>
                </select>
            </label>
            <label class="block">
                "Select Bin Step"
                <div class="block box-border pt-1 font-semibold w-full max-w-xs space-x-4">
                    <label class="cursor-pointer">
                        <input
                            class=""
                            type="radio"
                            name="binStep"
                            value="25"
                            on:input=move |ev| set_bin_step.set(event_target_value(&ev))
                        />
                        "0.25%"
                    </label>
                    <label class="cursor-pointer">
                        <input
                            class=""
                            type="radio"
                            name="binStep"
                            value="50"
                            on:input=move |ev| set_bin_step.set(event_target_value(&ev))
                        />
                        "0.5%"
                    </label>
                    <label class="cursor-pointer">
                        <input
                            class=""
                            type="radio"
                            name="binStep"
                            value="100"
                            on:input=move |ev| set_bin_step.set(event_target_value(&ev))
                        />
                        "1%"
                    </label>
                </div>
            </label>
            <label class="block">
                "Enter Active Price"
                <input
                    name="active_price"
                    class="block p-1 font-bold w-full max-w-xs box-border"
                    type="number"
                    inputmode="decimal"
                    min="0"
                    placeholder="0.0"
                    title="Enter Active Price"
                    on:input=move |ev| set_active_price.set(event_target_value(&ev))
                />
            </label>
            <button class="w-full p-1 !mt-6" type="submit">
                Create Pool
            </button>
        </form>
    }
}
