use leptos::prelude::*;
use tracing::{debug, info};

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

    // TODO:
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
