use crate::components::AnimatedShow;
use crate::state::*;
use crate::CHAIN_ID;
use rsecret::query::compute::ComputeQuerier;
use rsecret::secret_network_client::CreateQuerierOptions;
use send_wrapper::SendWrapper;
use std::time::Duration;

use leptos::prelude::*;
use leptos_router::components::{ParentRoute, Route, A};
use leptos_router::hooks::use_params_map;
use leptos_router::nested_router::Outlet;
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

    use crate::liquidity_book::lb_factory::QueryMsg;
    use crate::liquidity_book::Querier;

    let resource = LocalResource::new(move || {
        SendWrapper::new(async move { QueryMsg::GetNumberOfLbPairs {}.do_query().await })
    });

    provide_context(resource);

    let show = RwSignal::new(false);

    view! {
        <div
            class="hover-me fadeIn"
            on:mouseenter=move |_| show.set(true)
            on:mouseleave=move |_| show.set(false)
        >
            "Hover Me"
        </div>

        <AnimatedShow
            when=show
            show_class="fadeIn"
            hide_class="fadeOut"
            hide_delay=Duration::from_millis(1000)
        >
            <div class="here-i-am">
                "Here I Am!"
            </div>
        </AnimatedShow>

        <div class="p-2 fadeIn">
            <Outlet/>
        </div>

        // alternate layout with button to the right
        // <div class="p-2 flex flex-row items-center gap-12">
        //     <div class="space-y-0">
        //         <div class="text-3xl font-bold">Pool</div>
        //         <div class="text-sm text-neutral-400">Provide liquidity and earn fees.</div>
        //     </div>
        //     <A href="/pool/create">
        //         <button>"Create New Pool"</button>
        //     </A>
        // </div>

    }
}

#[component]
pub fn PoolBrowser() -> impl IntoView {
    info!("rendering <PoolBrowser/>");

    on_cleanup(move || {
        info!("cleaning up <PoolBrowser/>");
    });

    // TODO: query for the pools
    let pools = vec!["foo", "bar"];

    let resource = use_context::<LocalResource<String>>().expect("Context missing!");

    view! {
        <div class="text-3xl font-bold">"Pool"</div>
        <div class="text-sm text-neutral-400">
            "Provide liquidity and earn fees."
        </div>

        <h3 class="mb-1">"Existing Pools"</h3>
        <ul>{pools.into_iter().map(|n| view! { <li>{n}</li> }).collect_view()}</ul>

        // <h3>{move || resource.get()}</h3>

        <A href="/pool/create">
            <button>"Create New Pool"</button>
        </A>
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

    let resource = Resource::new(
        move || (token_a(), token_b(), basis_points()),
        move |(token_a, token_b, basis_points)| {
            SendWrapper::new(async move {
                let encryption_utils = secretrs::EncryptionUtils::new(None, CHAIN_ID).unwrap();
                // TODO: revisit this. url is not needed, EncryptionUtils should be a trait
                let options = CreateQuerierOptions {
                    url: "https://grpc.mainnet.secretsaturn.net",
                    chain_id: CHAIN_ID,
                    encryption_utils,
                };
                let compute = ComputeQuerier::new(wasm_client.get(), options);
                // TODO:
                let query = format!("{}, {}, {}", token_a, token_b, basis_points);
                debug!("{query}");

                let result = compute.address_by_label("amber-24").await;
                result.map_err(Into::<crate::Error>::into)
            })
        },
    );
    let (pending, set_pending) = signal(false);

    view! {
        <a
            href="/pool"
            class="block text-neutral-200/50 text-sm font-bold cursor-pointer no-underline"
        >
            "🡨 Back to pools list"
        </a>
        <div class="flex p-2 items-center gap-4">
            <div class="text-3xl font-bold">{token_a}" / "{token_b}</div>
            <div class="text-md font-bold p-1 outline outline-1 outline-offset-2 outline-neutral-500/50">
                {basis_points}" bps"
            </div>
            <a href="about:blank" target="_blank" rel="noopener">
                <div class="text-md font-bold p-1 outline outline-1 outline-offset-2 outline-neutral-500/50">
                    "secret123...xyz ↗"
                </div>
            </a>
        </div>

        <Suspense fallback=|| view! { <p>"Loading..."</p> }>
            // you can `.await` resources to avoid dealing with the `None` state
            <p>
                "User ID: "
                {move || Suspend::new(async move {
                    match resource.await {
                        Ok(response) => response,
                        Err(_) => "error".to_string(),
                    }
                })}
            </p>
        // or you can still use .get() to access resources in things like component props
        // <For
        // each=move || resource.get().and_then(Result::ok).unwrap_or_default()
        // key=|resource| resource.id
        // let:post
        // >
        // // ...
        // </For>
        </Suspense>
    }
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
