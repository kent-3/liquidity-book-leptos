use cosmwasm_std::Uint128;
use leptos::{ev, html, prelude::*};
use lucide_leptos::{ChevronDown, TriangleAlert};
use tracing::info;

#[component]
pub fn SwapDetails(
    #[prop(into)] price_ratio: Signal<Option<f64>>,
    #[prop(into)] expected_output: Signal<Option<Uint128>>,
    #[prop(into)] minimum_received: Signal<Option<Uint128>>,
    #[prop(into)] price_impact: Signal<f64>,
) -> impl IntoView {
    info!("rendering <SwapDetails/>");

    on_cleanup(move || {
        info!("cleaning up <SwapDetails/>");
    });

    let (expanded, set_expanded) = signal(false);

    let content_ref = NodeRef::<html::Div>::new();

    let toggle_expand = move |_: ev::MouseEvent| {
        if let Some(content) = content_ref.get() {
            // let full_height = content.get_bounding_client_rect().height();
            let full_height = content.scroll_height();

            if expanded.get() {
                // Ensure the content has an explicit height before collapsing
                content.style(("height", format!("{}px", full_height)));
                request_animation_frame(move || {
                    content.style(("height", "0px"));
                });
            } else {
                // First, set the height explicitly (this fixes the first animation issue)
                content.style(("height", "0px"));
                request_animation_frame(move || {
                    content.style(("height", format!("{}px", full_height)));
                });

                // Reset height to `auto` after transition ends to allow dynamic resizing
                let expanded_signal = expanded.clone();
                window_event_listener_untyped("transitionend", move |_| {
                    if expanded_signal.get() {
                        if let Some(content) = content_ref.get() {
                            content.style(("height", "auto"));
                        }
                    }
                });
            }
        }
        set_expanded.update(|e| *e = !*e);
    };

    view! {
        <div class="flex flex-col w-full rounded-md box-border border border-solid border-border">
            // Header (Click to Toggle)
            <div
                class="min-h-[40px] px-4 flex items-center justify-between cursor-pointer"
                on:click=toggle_expand
            >
                // TODO: toggle between price ratio on click. somehow make this take precedence
                // over the toggle_expand for the whole header.
                // NOTE: This price ratio is based on the expected output (amount_out).
                // TODO: add token symbols to this string.
                <p on:click=move |_| () class="m-0 text-sm text-white font-semibold">
                    {move || price_ratio.get().map(|uint128| uint128.to_string())}
                // "1 AVAX = 35.37513945 USDC"
                </p>
                <div
                    class="flex items-center justify-center transition-transform"
                    class=("rotate-180", move || expanded.get())
                >
                    <ChevronDown size=20 />
                </div>
            </div>

            // Expandable Content
            <div
                node_ref=content_ref
                class="transition-all ease-standard box-border overflow-hidden"
                class=(["opacity-0", "invisible", "h-0"], move || !expanded.get())
                class=(["opacity-100", "visible"], move || expanded.get())
            >
                <div class="w-full box-border p-4 pt-2 flex flex-col gap-2 items-center">
                    <div class="w-full flex flex-row justify-between text-sm">
                        <p class="m-0 text-muted-foreground">"Expected Output:"</p>
                        <p class="m-0 text-foreground font-semibold">
                            {move || expected_output.get().map(|uint128| uint128.to_string())}
                        </p>
                    </div>
                    <div class="w-full flex flex-row justify-between text-sm">
                        <p class="m-0 text-muted-foreground">"Minimum Received:"</p>
                        <p class="m-0 text-foreground font-semibold">
                            {move || minimum_received.get().map(|uint128| uint128.to_string())}
                        </p>
                    </div>
                    <div class="w-full flex flex-row justify-between text-sm">
                        <p class="m-0 text-muted-foreground">"Price Impact:"</p>
                        <p class="m-0 text-foreground font-semibold">
                            {move || price_impact.get()}
                        </p>
                    </div>
                </div>
            </div>

            // Warning (Price Impact, etc)
            <Show when=move || price_impact.get().gt(&2.0)>
                <div class="flex flex-col items-center gap-2 m-2 mt-0">
                    <div class="flex items-center justify-between box-border w-full px-4 py-2 text-sm text-white font-semibold bg-red-500/90 rounded-md">
                        // price impact icon and text
                        <div class="flex flex-row items-center gap-3">
                            <TriangleAlert size=20 />
                            <p class="m-0">"Price Impact Warning"</p>
                        </div>
                        // price impact percentage
                        <p class="m-0">{move || price_impact.get()}"%"</p>
                    </div>
                </div>
            </Show>
        </div>
    }
}
