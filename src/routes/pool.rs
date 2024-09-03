use leptos::prelude::*;
use leptos_router::components::{ParentRoute, Route, A};
use leptos_router::nested_router::Outlet;
use leptos_router::{MatchNestedRoutes, ParamSegment, StaticSegment};
use leptos_router_macro::path;
use tracing::debug;

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
    // TODO: query for the pools
    let pools = vec!["foo", "bar"];

    view! {
        <A href="/pool/create">
            <button>"Create New Pool"</button>
        </A>
        <p>"Existing Pools"</p>
        <ul>{pools.into_iter().map(|n| view! { <li>{n}</li> }).collect_view()}</ul>
    }
}

#[component]
pub fn PoolCreator() -> impl IntoView {
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
        // Now you can use token_x, token_y, bin_step, and active_price as needed
        // ...
    };

    view! {
        <h2 class="text-center sm:text-left">"Create New Pool"</h2>
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
                    "Select Bin Step" <div class="block box-border pt-1 font-semibold w-full max-w-xs space-x-4">
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
                <button class="w-full p-1 !mt-6" type="submit">Create Pool</button>
        </form>
    }
}
