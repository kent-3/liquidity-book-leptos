#![allow(unused)]

use crate::{
    constants::{CHAIN_ID, NODE, TOKEN_MAP},
    state::{ChainId, Endpoint, KeplrSignals, TokenMap},
    types::Coin,
    utils::*,
    Error, BASE_URL,
};
use keplr::Keplr;
use leptos::{either::Either, html, logging::*, prelude::*};
use lucide_leptos::{ArrowLeft, ChevronRight, History, Power, Settings, Wallet, WavesLadder};
use rsecret::query::{bank::BankQuerier, compute::ComputeQuerier};
use send_wrapper::SendWrapper;
use tonic_web_wasm_client::Client;
use tracing::{debug, info};
use web_sys::MouseEvent;

#[component]
pub fn WalletMenu(
    dialog_ref: NodeRef<html::Dialog>,
    toggle_menu: impl Fn(MouseEvent) + 'static,
) -> impl IntoView {
    info!("rendering <WalletMenu/>");

    let (contents, set_contents) = signal("main");

    let endpoint = use_context::<Endpoint>().expect("endpoint context missing!");
    let chain_id = use_context::<ChainId>().expect("chain id context missing!");
    let keplr = use_context::<KeplrSignals>().expect("keplr signals context missing!");
    let token_map = use_context::<TokenMap>().expect("tokens context missing!");

    let disable_keplr = move |_: MouseEvent| {
        Keplr::disable(CHAIN_ID);
        keplr.enabled.set(false);
    };

    let key_address = move || {
        keplr
            .key
            .get()
            .and_then(Result::ok)
            .map(|key| key.bech32_address)
    };

    // Note: this resource is running twice for some reason! at least here I can imagine it's due
    // to the AsyncDerived signal of keplr.key
    let user_balance = Resource::new(
        move || keplr.key.get(),
        move |key| {
            let client = Client::new(endpoint.get().to_string());
            SendWrapper::new(async move {
                if let Some(Ok(key)) = key {
                    let bank = BankQuerier::new(client);

                    bank.balance(key.bech32_address, "uscrt")
                        .await
                        .map(|balance| Coin::from(balance.balance.unwrap()))
                        .map_err(Error::from)
                        .inspect(|coin| debug!("{coin:?}"))
                        .inspect_err(|err| error!("{err:?}"))
                        .map(|coin| coin.amount.parse::<u128>().unwrap_or_default())
                        .map(|amount| display_token_amount(amount, 6u8))
                } else {
                    Ok(0.to_string())
                }
            })
        },
    );

    // move || user_balance.get().and_then(Result::ok).map(|coin| display_token_amount(coin.amount, 6))

    // let toaster = ToasterInjection::expect_context();
    //
    // let on_click = move |_| {
    //     toaster.dispatch_toast(
    //         move || {
    //             view! {
    //                <Toast>
    //                    <ToastTitle>"Email sent"</ToastTitle>
    //                    <ToastBody>
    //                        "This is a toast body"
    //                        <ToastBodySubtitle slot>
    //                            "Subtitle"
    //                        </ToastBodySubtitle>
    //                    </ToastBody>
    //                    <ToastFooter>
    //                        "Footer"
    //                        // <Link>Action</Link>
    //                        // <Link>Action</Link>
    //                    </ToastFooter>
    //                </Toast>
    //             }
    //         },
    //         ToastOptions::default().with_timeout(Duration::from_secs(5)),
    //     );
    // };

    view! {
        <dialog
            node_ref=dialog_ref
            class="z-50 mr-0 mt-2 px-0 py-3 w-80 shadow-md bg-popover text-popover-foreground rounded-lg border border-solid border-border"
        >
            <Show when=move || contents.get() == "activity">
                <div class="flex items-center px-2 pb-3">
                    <div
                        on:click=move |_| set_contents.set("main")
                        class="
                        inline-flex gap-x-3 items-center box-border w-full h-10 px-3 py-1 rounded
                        text-lg font-semibold hover:bg-secondary transition-colors ease-linear
                        "
                    >
                        <ArrowLeft size=22 absolute_stroke_width=true />
                        "Activity"
                    </div>
                </div>
                <hr class="m-0 border-border" />
                <p class="m-0 py-4 px-5 text-sm">"Most recent transactions will appear here..."</p>
                <hr class="m-0 border-border" />
                <div class="px-2 pt-3">
                    <a
                        href=move || {
                            key_address()
                                .map(|address| {
                                    format!("https://www.mintscan.io/secret/address/{}", address)
                                })
                        }
                        target="_blank"
                        rel="noopener"
                    >
                        <div class="menu-button">
                            <span class="text-sm text-muted-foreground">
                                "View more on explorer"
                            </span>
                            <ChevronRight size=20 absolute_stroke_width=true />
                        </div>
                    </a>
                </div>
            </Show>
            <Show when=move || contents.get() == "settings">
                <div class="flex items-center px-2 pb-3">
                    <div
                        on:click=move |_| set_contents.set("main")
                        class="
                        inline-flex gap-x-3 items-center box-border w-full h-10 px-3 py-1 rounded
                        text-lg text-foreground font-semibold hover:bg-secondary transition-colors ease-linear
                        "
                    >
                        <ArrowLeft size=22 absolute_stroke_width=true />
                        "Settings"
                    </div>
                </div>
                <hr class="m-0 border-border" />
            </Show>
            <Show when=move || contents.get() == "main">
                // <!-- Header -->
                <div class="flex items-center justify-between px-6 pb-3">
                    <div class="flex items-center gap-3">
                        <div class="w-8 h-8 flex items-center justify-center bg-transparent outline outline-[1.5px] outline-foam shadow-foam-glow rounded-full">
                            <img
                                class="w-5 h-5"
                                src=format!("{BASE_URL}{}", "/icons/SECRET_FOAM-ICON_RGB.svg")
                            />
                        </div>
                        <div>
                            <div class="text-xs text-muted-foreground font-light">
                                "Connected Account:"
                            </div>
                            <div class="text-base font-semibold">
                                {move || key_address().map(shorten_address)}
                            </div>
                        </div>
                    </div>
                    <button
                        title="Disconnect wallet"
                        on:click=disable_keplr
                        class="w-10 h-10 p-0 inline-flex items-center justify-center rounded-full bg-transparent text-gold active:bg-background
                        hover:bg-secondary hover:border-secondary hover:text-[#ffbf5a] hover:outline-[#ffbf5a] hover:shadow-gold-glow
                        outline outline-1 outline-offset-0 outline-transparent
                        border border-solid border-border
                        transition-all ease-standard duration-200"
                    >
                        <Power size=16 />
                    </button>
                </div>
                <hr class="m-0 border-border" />
                // <!-- Menu Items -->
                <ul class="space-y-1 px-1 py-2 list-none">
                    <li>
                        <a href="#">
                            <div class="menu-button">
                                <WavesLadder size=22 />
                                "My Pools"
                                <ChevronRight size=20 absolute_stroke_width=true />
                            </div>
                        </a>
                    </li>
                    <li>
                        <div on:click=move |_| set_contents.set("activity") class="menu-button">
                            <History size=22 />
                            "Activity"
                            <ChevronRight size=20 absolute_stroke_width=true />
                        </div>
                    </li>
                    <li>
                        <div on:click=move |_| set_contents.set("settings") class="menu-button">
                            <Settings size=22 />
                            "Settings"
                            <ChevronRight size=20 absolute_stroke_width=true />
                        </div>
                    </li>
                </ul>
                <hr class="m-0 border-border" />
                // <!-- Token List -->
                <div class="px-1 pt-2">
                    // <!-- Wallet Header -->
                    <div class="flex items-center gap-3 px-3 py-2 text-popover-foreground font-semibold [&_svg]:stroke-muted-foreground">
                        <Wallet size=22 />
                        "Wallet"
                    </div>
                    // <!-- Token Item -->
                    <div class="flex items-center justify-between px-3 py-2 rounded hover:bg-secondary">
                        <div class="flex items-center gap-3">
                            <img src=format!("{BASE_URL}{}", "/icons/uscrt.png") class="w-6 h-6" />
                            <div>
                                <div class="text-sm font-semibold">SCRT</div>
                                <div class="text-xs text-muted-foreground">Secret</div>
                            </div>
                        </div>
                        <div class="text-right">
                            <div class="text-sm font-semibold">
                                {move || user_balance.get().and_then(Result::ok)}
                            </div>
                            <div class="text-xs text-muted-foreground">$0</div>
                        </div>
                    </div>

                    // <!-- Token Item -->
                    <div class="flex items-center justify-between px-3 py-2 rounded hover:bg-secondary">
                        <div class="flex items-center gap-3">
                            <img
                                src="https://raw.githubusercontent.com/traderjoe-xyz/joe-tokenlists/main/logos/0xB97EF9Ef8734C71904D8002F8b6Bc66Dd9c48a6E/logo.png"
                                alt="USDC logo"
                                class="w-6 h-6"
                            />
                            <div>
                                <div class="text-sm font-semibold">USDC</div>
                                <div class="text-xs text-muted-foreground">USD Coin</div>
                            </div>
                        </div>
                        <div class="text-right">
                            <div class="text-sm font-semibold">0</div>
                            <div class="text-xs text-muted-foreground">$0</div>
                        </div>
                    </div>

                    // <!-- Token Item -->
                    <div class="flex items-center justify-between px-3 py-2 rounded hover:bg-secondary">
                        <div class="flex items-center gap-3">
                            <img src=format!("{BASE_URL}{}", "/icons/amber.svg") class="w-6 h-6" />
                            <div>
                                <div class="text-sm font-semibold">AMBER</div>
                                <div class="text-xs text-muted-foreground">Amber</div>
                            </div>
                        </div>
                        <div class="text-right">
                            <div class="text-sm font-semibold">0</div>
                            <div class="text-xs text-muted-foreground">$0</div>
                        </div>
                    </div>
                </div>
            </Show>
        </dialog>
    }
}
