use leptos::{ev, html, logging::*, prelude::*, tachys::dom::window};
use lucide_leptos::{Info, X};
use tracing::{debug, info};

#[component]
fn SwapSettings(
    dialog_ref: NodeRef<html::Dialog>,
    toggle_menu: impl Fn(ev::MouseEvent) + 'static,
    slippage: (Signal<u16>, WriteSignal<u16>),
    deadline: (Signal<u64>, WriteSignal<u64>),
) -> impl IntoView {
    info!("rendering <SettingsMenu/>");

    view! {
        <div class="floating-menu">
            <dialog
                node_ref=dialog_ref
                class="z-40 mt-1.5 -mr-0 md:-mr-[124px] w-80 h-52 p-0 shadow-md bg-background text-foreground rounded-md border border-solid border-border"
            >
                <div class="relative flex flex-col z-auto">
                    // <div class="absolute right-1.5 top-1.5 flex shrink-0 items-center justify-center w-6 h-6 p-1 box-border rounded-md hover:bg-neutral-700">
                    // <X size=16 />
                    // </div>
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
                                            on:click=move |_| slippage.1.set(10)
                                            class="h-8 min-w-8 w-16 text-sm font-semibold bg-secondary text-secondary-foreground rounded-md"
                                        >
                                            "0.1%"
                                        </button>
                                        <button
                                            on:click=move |_| slippage.1.set(50)
                                            class="h-8 min-w-8 w-16 text-sm font-semibold bg-secondary text-secondary-foreground rounded-md"
                                        >
                                            "0.5%"
                                        </button>
                                        <button
                                            on:click=move |_| slippage.1.set(100)
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
                                            prop:value=move || { slippage.0.get() as f64 / 100.0 }
                                            on:change=move |ev| {
                                                let value = event_target_value(&ev)
                                                    .parse::<f64>()
                                                    .unwrap_or(0.5);
                                                let value = (value * 100.0).round() as u16;
                                                slippage.1.set(value)
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
                                        prop:value=move || { deadline.0.get() }
                                        on:change=move |ev| {
                                            let value = event_target_value(&ev)
                                                .parse::<u64>()
                                                .unwrap_or(5u64);
                                            deadline.1.set(value)
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
