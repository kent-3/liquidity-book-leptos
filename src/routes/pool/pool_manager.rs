use crate::state::*;
use leptos::prelude::*;
use leptos_router::{
    components::A,
    hooks::{use_params, use_params_map, use_query_map},
    nested_router::Outlet,
};
use send_wrapper::SendWrapper;
use shade_protocol::liquidity_book::lb_pair::{
    ActiveIdResponse, BinResponse, NextNonEmptyBinResponse, TotalSupplyResponse,
};
use tracing::{debug, info};

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
        debug!("{:?}", response);
        serde_json::from_str::<ReservesResponse>(&response)
            .expect("Failed to deserialize ReservesResponse!")
    }
    async fn query_active_id() -> u32 {
        let response = QueryMsg::GetActiveId {}.do_query().await;
        debug!("{:?}", response);
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
        || (),
        move |_| SendWrapper::new(async move { query_reserves().await }),
    );
    let active_id = Resource::new(
        || (),
        move |_| SendWrapper::new(async move { query_active_id().await }),
    );
    // TODO: Figure out how to prevent these from running twice
    let bin_reserves = Resource::new(
        move || active_id.track(),
        move |_| {
            debug!("bin_reserves");
            SendWrapper::new(async move {
                let id = active_id.await;
                query_bin_reserves(id).await
            })
        },
    );
    // let bin_reserves = Action::new(move |input: &u32| {
    //     debug!("bin_reserves");
    //     let id = input.clone();
    //     SendWrapper::new(async move {
    //         // let id = active_id.await;
    //         query_bin_reserves(id).await
    //     })
    // });
    let next_non_empty_bin = Resource::new(
        move || active_id.track(),
        move |_| {
            debug!("next_non_empty_bin");
            SendWrapper::new(async move {
                let id = active_id.await;
                query_next_non_empty_bin(id).await
            })
        },
    );
    let bin_total_supply = Resource::new(
        move || active_id.track(),
        move |_| {
            debug!("bin_total_supply");
            SendWrapper::new(async move {
                let id = active_id.await;
                query_total_supply(id).await
            })
        },
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
                        "Total Reserves: "<span tabindex="0" class="cursor-pointer text-white peer">
                            "🛈"
                        </span>
                        <li class="list-none text-sm font-bold text-violet-400 peer-focus:block hidden">
                            "🛈 Reserves may be in reverse order"
                        </li>
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
                                <li class="pl-4 list-none">
                                    "bin_reserve_x: "{reserves.bin_reserve_x}
                                </li>
                                <li class="pl-4 list-none">
                                    "bin_reserve_y: "{reserves.bin_reserve_y}
                                </li>
                            }
                        })}
                    </li>
                </Suspense>
                <Suspense fallback=|| view! { <div>"Loading Next Non-Empty Bin..."</div> }>
                    <li>
                        "Next Non-Empty Bin ID: "
                        {move || Suspend::new(async move { next_non_empty_bin.await })}
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
