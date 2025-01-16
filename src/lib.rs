// #![allow(unused)]

// use codee::string::FromToStringCodec;
// use leptos_use::storage::use_local_storage;

use std::time::Duration;

use crate::prelude::*;
use crate::support::chain_query;
use ammber_sdk::contract_interfaces::{
    lb_factory::{self, LbPairAtIndexResponse},
    lb_pair::LbPair,
};
use batch_query::{
    msg_batch_query, parse_batch_query, BatchItemResponseStatus, BatchQuery, BatchQueryParams,
    BatchQueryParsedResponse, BatchQueryResponse, BATCH_QUERY_ROUTER,
};
use keplr::{Keplr, Key};
use leptos::{
    ev::MouseEvent,
    html::{Dialog, Input},
    logging::*,
    prelude::*,
    task::spawn_local,
};
use leptos_icons::Icon;
use leptos_meta::*;
use leptos_router::components::{ParentRoute, Route, Router, Routes, A};
use leptos_router_macro::path;
use rsecret::query::{bank::BankQuerier, compute::ComputeQuerier};
use send_wrapper::SendWrapper;
use serde::{Deserialize, Serialize};
use thaw::*;
use tonic_web_wasm_client::Client;
use tracing::{debug, error, info, trace};
use web_sys::{js_sys, wasm_bindgen::JsValue};

mod components;
mod constants;
mod error;
mod prelude;
mod routes;
mod state;
mod support;
mod types;
mod utils;

use components::{Spinner2, SuggestChains};
use constants::{CHAIN_ID, NODE, TOKEN_MAP};
use error::Error;
use routes::{home::Home, nav::Nav, pool::*, trade::*};
use state::{ChainId, Endpoint, KeplrSignals, TokenMap};
use types::Coin;

// TODO: configure this to be different in dev mode
pub static BASE_URL: &str = "/liquidity-book-leptos";

// TODO: If possible, use batch queries for resources. Combine the outputs in a struct
// and use that as the return type of the Resource.

#[derive(Clone)]
pub struct NumberOfLbPairs(pub LocalResource<u32>);

#[component]
pub fn App() -> impl IntoView {
    info!("rendering <App/>");

    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    // Global Contexts

    provide_context(Endpoint::new(NODE));
    provide_context(ChainId::new(CHAIN_ID));
    provide_context(KeplrSignals::new());
    provide_context(TokenMap::new(TOKEN_MAP.clone()));

    let endpoint = use_context::<Endpoint>().expect("endpoint context missing!");
    let chain_id = use_context::<ChainId>().expect("chain id context missing!");
    let keplr = use_context::<KeplrSignals>().expect("keplr signals context missing!");
    let token_map = use_context::<TokenMap>().expect("tokens context missing!");

    // debug!("{} Keplr tokens", token_map.len());
    // debug!(
    //     "{:#?}",
    //     token_map
    //         .iter()
    //         .map(|(_, token)| token.metadata.symbol.clone())
    //         .collect::<Vec<String>>()
    // );
    debug!("{} known tokens", token_map.len());
    debug!(
        "{:#?}",
        token_map
            .iter()
            .map(|(_, token)| token.symbol.clone())
            .collect::<Vec<String>>()
    );

    let sscrt_address = SYMBOL_TO_ADDR.get("SSCRT").expect("sSCRT is missing!");
    debug!("sSCRT address: {sscrt_address}");

    use codee::string::FromToStringCodec;
    use leptos_use::storage::*;

    // Bind string with LocalStorage stored in string format:
    let (id, set_id, _) = use_local_storage::<u32, FromToStringCodec>("my-id");

    Effect::new(move || debug!("id signal is {}", id.get()));

    // spawn_local(async move {
    //     if id.get() == 0 {
    //         let number = LB_FACTORY
    //             .get_number_of_lb_pairs()
    //             .await
    //             .unwrap_or_default();
    //         set_id.set(number);
    //     }
    // });

    // let storage = window()
    //     .local_storage()
    //     .expect("local storage not available?")
    //     .expect("local storage returned none?");

    // Effect::new(move || {
    //     let x = storage.get_item("my-id");
    //     debug!("{:?}", x);
    // });

    // spawn_local(async move {
    //     if Ok(None) == storage.get_item("my-id") {
    //         let number = LB_FACTORY
    //             .get_number_of_lb_pairs()
    //             .await
    //             .unwrap_or_default();
    //
    //         let _ = storage.set_item("lb_pairs", &number.to_string());
    //     }
    // });

    // TODO: How can we get these lb_pair queries to only happen when navigating to the "pool" route,
    // but not re-run every time that page loads? We need some kind of in-memory cache for this,
    // because we do want it to re-run if the user refreshes the page (to load any new pairs).

    let number_of_lb_pairs: LocalResource<u32> = LocalResource::new(move || async move {
        let storage = window()
            .local_storage()
            .expect("local storage not available?")
            .expect("local storage returned none?");

        match storage.get_item("number_of_lb_pairs") {
            Ok(None) => {
                let number = LB_FACTORY
                    .get_number_of_lb_pairs()
                    .await
                    .unwrap_or_default();

                let _ = storage.set_item("number_of_lb_pairs", &number.to_string());

                number
            }
            Ok(Some(number)) => number.parse::<u32>().unwrap(),
            _ => 0,
        }
    });

    // Effect::new(move || {
    //     if let Some(number) = number_of_lb_pairs.get().as_deref() {
    //         set_id.set(*number)
    //     }
    // });

    let all_lb_pairs: LocalResource<Vec<LbPair>> = LocalResource::new(move || async move {
        let storage = window()
            .local_storage()
            .expect("local storage not available?")
            .expect("local storage returned none?");

        let i = number_of_lb_pairs.await;
        let mut queries = Vec::new();

        for index in 0..i {
            queries.push(BatchQueryParams {
                id: index.to_string(),
                contract: LB_FACTORY.0.clone(),
                query_msg: lb_factory::QueryMsg::GetLbPairAtIndex { index },
            });
        }

        let batch_query_message = msg_batch_query(queries);

        match storage.get_item("all_lb_pairs") {
            // TODO: change BATCH_QUERY_ROUTER to automatically know the current chain_id
            Ok(None) => {
                let pairs = chain_query::<BatchQueryResponse>(
                    BATCH_QUERY_ROUTER.pulsar.code_hash.clone(),
                    BATCH_QUERY_ROUTER.pulsar.address.to_string(),
                    batch_query_message,
                )
                .await
                .map(parse_batch_query)
                .map(extract_pairs_from_batch)
                .unwrap();

                let _ = storage.set_item("all_lb_pairs", &serde_json::to_string(&pairs).unwrap());

                pairs
            }
            Ok(Some(pairs)) => serde_json::from_str(&pairs).unwrap(),
            _ => vec![],
        }
    });

    fn extract_pairs_from_batch(batch_response: BatchQueryParsedResponse) -> Vec<LbPair> {
        batch_response
            .items
            .into_iter()
            .filter(|item| item.status == BatchItemResponseStatus::SUCCESS) // Process only successful items
            .map(|item| {
                serde_json::from_str::<LbPairAtIndexResponse>(&item.response)
                    .expect("Invalid LbPairAtIndexResponse JSON")
            })
            .map(|item| item.lb_pair)
            .collect()
    }

    provide_context(NumberOfLbPairs(number_of_lb_pairs));
    provide_context(all_lb_pairs);

    Effect::new(move |_| {
        let enabled = keplr.enabled.get();
        let key = keplr.key.get();
        debug!("\nKeplr enabled?: {}\nKeplr Key: {:#?}", enabled, key)
    });
    Effect::new(move |_| info!("Endpoint set to {}", endpoint.get()));
    Effect::new(move |_| info!("Chain ID set to {:?}", chain_id.get()));

    // Event Listeners

    // Whenever the key store changes, this will re-set 'keplr.enabled' to true, triggering a
    // reload of everything subscribed to that signal
    let keplr_keystorechange_handle =
        window_event_listener_untyped("keplr_keystorechange", move |_| {
            warn!("Key store in Keplr is changed. You may need to refetch the account info.");
            keplr.enabled.set(true);
        });

    on_cleanup(move || {
        info!("cleaning up <App/>");
        keplr_keystorechange_handle.remove()
    });

    // Actions

    let enable_keplr_action: Action<(), bool, SyncStorage> =
        Action::new_unsync_with_value(Some(false), move |_: &()| async move {
            let keplr_extension = js_sys::Reflect::get(&window(), &JsValue::from_str("keplr"))
                .expect("unable to check for `keplr` property");

            if keplr_extension.is_undefined() || keplr_extension.is_null() {
                window()
                    .alert_with_message("keplr not found")
                    .expect("alert failed");
                keplr.enabled.set(false);
                false
            } else {
                debug!("Trying to enable Keplr...");
                match Keplr::enable(vec![CHAIN_ID.to_string()]).await {
                    Ok(_) => {
                        keplr.enabled.set(true);
                        debug!("Keplr is enabled");
                        true
                    }
                    Err(e) => {
                        keplr.enabled.set(false);
                        error!("{e}");
                        false
                    }
                }
            }
        });

    // on:click handlers

    let enable_keplr = move |_: MouseEvent| {
        enable_keplr_action.dispatch(());
    };

    let disable_keplr = move |_: MouseEvent| {
        Keplr::disable(CHAIN_ID);
        keplr.enabled.set(false);
    };

    // Node references

    let wallet_dialog_ref = NodeRef::<Dialog>::new();
    let options_dialog_ref = NodeRef::<Dialog>::new();

    // HTML Elements

    let toggle_wallet_menu = move |_| match wallet_dialog_ref.get() {
        Some(dialog) => match dialog.open() {
            false => {
                let _ = dialog.show();
            }
            true => dialog.close(),
        },
        None => {
            let _ = window().alert_with_message("Something is wrong!");
        }
    };
    let toggle_options_menu = move |_| match options_dialog_ref.get() {
        Some(dialog) => match dialog.open() {
            false => {
                let _ = dialog.show_modal();
            }
            true => dialog.close(),
        },
        None => {
            let _ = window().alert_with_message("Something is wrong!");
        }
    };

    let key_name = move || keplr.key.get().and_then(Result::ok).map(|key| key.name);
    let key_address = move || {
        keplr
            .key
            .get()
            .and_then(Result::ok)
            .map(|key| key.bech32_address)
    };

    let theme = RwSignal::new(Theme::dark());

    view! {
        <Router>
            // <div class="background-image"></div>
            <header>
                <div class="flex justify-between items-center">
                    <div
                        id="mainTitle"
                        class="my-2 font-bold text-3xl line-clamp-1 transition-transform duration-300"
                    >
                        "Liquidity Book"
                    </div>
                    <Show when=move || keplr.key.get().and_then(|key| key.ok()).is_some()>
                        <p class="hidden sm:block text-sm outline outline-2 outline-offset-8 outline-neutral-500">
                            "Connected as "<strong>{key_name}</strong>
                        </p>
                    </Show>
                    <div class="flex gap-1">
                        // This seems convoluted... is there a better way?
                        <Show
                            when=move || keplr.enabled.get()
                            fallback=move || {
                                view! {
                                    <button
                                        on:click=enable_keplr
                                        disabled=enable_keplr_action.pending()
                                        class="min-w-24 text-sm font-semibold leading-none py-[5px] px-[12px] inline-flex justify-center items-center align-middle"
                                    >
                                        "Connect Wallet"
                                    </button>
                                    <button
                                        on:click=toggle_options_menu
                                        class="text-xl font-semibold leading-none py-[2px] px-[8px] inline-flex justify-center items-center align-middle"
                                    >
                                        <svg
                                            xmlns="http://www.w3.org/2000/svg"
                                            class="stroke-white"
                                            height="24px"
                                            viewBox="0 0 24 24"
                                            width="24px"
                                            stroke-width="1.5"
                                            stroke="currentColor"
                                            fill="none"
                                        >
                                            <path
                                                stroke-linecap="round"
                                                stroke-linejoin="round"
                                                d="M10.343 3.94c.09-.542.56-.94 1.11-.94h1.093c.55 0 1.02.398 1.11.94l.149.894c.07.424.384.764.78.93.398.164.855.142 1.205-.108l.737-.527a1.125 1.125 0 0 1 1.45.12l.773.774c.39.389.44 1.002.12 1.45l-.527.737c-.25.35-.272.806-.107 1.204.165.397.505.71.93.78l.893.15c.543.09.94.559.94 1.109v1.094c0 .55-.397 1.02-.94 1.11l-.894.149c-.424.07-.764.383-.929.78-.165.398-.143.854.107 1.204l.527.738c.32.447.269 1.06-.12 1.45l-.774.773a1.125 1.125 0 0 1-1.449.12l-.738-.527c-.35-.25-.806-.272-1.203-.107-.398.165-.71.505-.781.929l-.149.894c-.09.542-.56.94-1.11.94h-1.094c-.55 0-1.019-.398-1.11-.94l-.148-.894c-.071-.424-.384-.764-.781-.93-.398-.164-.854-.142-1.204.108l-.738.527c-.447.32-1.06.269-1.45-.12l-.773-.774a1.125 1.125 0 0 1-.12-1.45l.527-.737c.25-.35.272-.806.108-1.204-.165-.397-.506-.71-.93-.78l-.894-.15c-.542-.09-.94-.56-.94-1.109v-1.094c0-.55.398-1.02.94-1.11l.894-.149c.424-.07.765-.383.93-.78.165-.398.143-.854-.108-1.204l-.526-.738a1.125 1.125 0 0 1 .12-1.45l.773-.773a1.125 1.125 0 0 1 1.45-.12l.737.527c.35.25.807.272 1.204.107.397-.165.71-.505.78-.929l.15-.894Z"
                                            />
                                            <path
                                                stroke-linecap="round"
                                                stroke-linejoin="round"
                                                d="M15 12a3 3 0 1 1-6 0 3 3 0 0 1 6 0Z"
                                            />
                                        </svg>
                                    </button>
                                }
                            }
                        >
                            <div class="relative inline-block">

                                <button
                                    on:click=toggle_wallet_menu
                                    class="min-w-24 text-sm font-semibold leading-none py-[5px] px-[12px] inline-flex justify-center items-center align-middle"
                                >
                                    // class="min-w-24 transition-shadow active:bg-neutral-900 active:border-neutral-600 hover:bg-neutral-700 hover:border-neutral-500 ease-standard duration-100 box-border font-semibold leading-5 inline-flex items-center justify-center rounded border border-solid border-neutral-600 bg-neutral-800 text-sm py-[5px] px-[12px]"
                                    "Wallet Menu"
                                // {move || key_address().map(shorten_address)}
                                </button>
                                <WalletMenu
                                    dialog_ref=wallet_dialog_ref
                                    toggle_menu=toggle_options_menu
                                />
                            </div>
                        </Show>
                    </div>
                </div>
                <hr />
                <Nav />
                <hr />
            </header>
            <main class="p-2 overflow-x-auto">
                <Routes transition=true fallback=|| "This page could not be found.">
                    <Route path=path!("/liquidity-book-leptos/") view=Home />
                    <ParentRoute path=path!("/liquidity-book-leptos/pool") view=Pool>
                        <Route path=path!("/") view=PoolBrowser />
                        <Route path=path!("/create") view=PoolCreator />
                        <ParentRoute path=path!("/:token_a/:token_b/:bps") view=PoolManager>
                            <Route path=path!("/") view=|| () />
                            <Route path=path!("/add") view=AddLiquidity />
                            <Route path=path!("/remove") view=RemoveLiquidity />
                        </ParentRoute>
                    </ParentRoute>
                    <Route path=path!("/liquidity-book-leptos/trade") view=Trade />
                </Routes>
            </main>
            <LoadingModal when=enable_keplr_action.pending() message="Requesting Connection" />
            <SettingsMenu dialog_ref=options_dialog_ref toggle_menu=toggle_options_menu />
        </Router>
    }
}

#[component]
pub fn LoadingModal(when: Memo<bool>, #[prop(into)] message: String) -> impl IntoView {
    let dialog_ref = NodeRef::<Dialog>::new();

    Effect::new(move |_| match dialog_ref.get() {
        Some(dialog) => match when.get() {
            true => {
                let _ = dialog.show_modal();
            }
            false => dialog.close(),
        },
        None => (),
    });

    view! {
        <dialog node_ref=dialog_ref class="block inset-0">
            // NOTE: when 'display: none' is toggled on/off, some of the animation gets lost,
            // so it's better to use 'visibility: hidden' instead of 'display: none'.
            // Tailwind's 'invisible' = 'visibility: hidden' and 'hidden' = 'display: none'
            // The svg will be spinning invisibly, but it's worth it for the nicer animation.
            // class=("invisible", move || !when.get())
            <div class="inline-flex items-center">
                <Spinner2 size="h-8 w-8" />
                <div class="font-bold">{message}</div>
            </div>
        </dialog>
    }
}

#[component]
pub fn WalletMenu(
    dialog_ref: NodeRef<Dialog>,
    toggle_menu: impl Fn(MouseEvent) + 'static,
) -> impl IntoView {
    info!("rendering <WalletMenu/>");

    let endpoint = use_context::<Endpoint>().expect("endpoint context missing!");
    let chain_id = use_context::<ChainId>().expect("chain id context missing!");
    let keplr = use_context::<KeplrSignals>().expect("keplr signals context missing!");
    let token_map = use_context::<TokenMap>().expect("tokens context missing!");

    let disable_keplr = move |_: MouseEvent| {
        Keplr::disable(CHAIN_ID);
        keplr.enabled.set(false);
        // keplr.key.set(None);
    };

    let key_address = move || {
        keplr
            .key
            .get()
            .and_then(Result::ok)
            .map(|key| key.bech32_address)
    };

    let user_balance = Resource::new(
        move || keplr.key.track(),
        move |_| {
            let client = Client::new(endpoint.get());
            SendWrapper::new(async move {
                let bank = BankQuerier::new(client);
                let key = keplr.key.await?;

                bank.balance(key.bech32_address, "uscrt")
                    .await
                    .map(|balance| Coin::from(balance.balance.unwrap()))
                    .map_err(Error::from)
                    .inspect(|coin| debug!("{coin:?}"))
                    .inspect_err(|err| error!("{err:?}"))
                    .map(|coin| coin.amount.parse::<u128>().unwrap_or_default())
                    .map(|amount| display_token_amount(amount, 6u8))
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
            class="mr-0 mt-2 px-0 py-3 shadow-lg bg-neutral-800 rounded border border-neutral-600"
        >
            // <!-- Header -->
            <div class="flex items-center justify-between w-72 px-6 pb-3">
                <div class="flex items-center gap-3">
                    <div class="w-8 h-8 flex items-center justify-center bg-transparent outline outline-[1.5px] outline-foam shadow-foam-glow rounded-full">
                        <img class="w-5 h-5" src=format!("{BASE_URL}{}", "/icons/SECRET_FOAM-ICON_RGB.svg") />
                    </div>
                    <div>
                        <div class="text-xs text-neutral-400 font-light">"Connected Account:"</div>
                        <div class="text-base font-semibold">
                            {move || key_address().map(shorten_address)}
                        </div>
                    </div>
                </div>
                <button
                    title="Disconnect wallet"
                    class="w-10 h-10 p-0 bg-transparent active:bg-neutral-900 hover:bg-neutral-700 hover:outline-gold hover:saturate-150 hover:shadow-gold-glow transition-all ease-standard duration-200 rounded-full inline-flex items-center justify-center outline outline-1 outline-offset-0 outline-transparent border border-solid border-neutral-500"
                >
                    <svg
                        xmlns="http://www.w3.org/2000/svg"
                        fill="none"
                        viewBox="0 0 24 24"
                        stroke-width="2.5"
                        stroke="currentColor"
                        class="w-4 h-4 stroke-gold"
                    >
                        <path
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            d="M5.636 5.636a9 9 0 1 0 12.728 0M12 3v9"
                        />
                    </svg>
                </button>
            </div>
            <hr class="m-0 border-neutral-600" />
            // <!-- Menu Items -->
            <ul class="space-y-1 px-1 py-2 list-none font-semibold text-base">
                <li>
                    <a
                        href="#"
                        class="hover:no-underline no-underline flex items-center gap-3 px-3 py-2 rounded text-neutral-200 hover:bg-neutral-700 ease-linear transition-all duration-200"
                    >
                        <svg
                            xmlns="http://www.w3.org/2000/svg"
                            fill="none"
                            viewBox="0 0 24 24"
                            stroke-width="1.5"
                            stroke="currentColor"
                            class="w-6 h-6 stroke-neutral-400"
                        >
                            <path
                                stroke-linecap="round"
                                stroke-linejoin="round"
                                d="M7.5 14.25v2.25m3-4.5v4.5m3-6.75v6.75m3-9v9M6 20.25h12A2.25 2.25 0 0 0 20.25 18V6A2.25 2.25 0 0 0 18 3.75H6A2.25 2.25 0 0 0 3.75 6v12A2.25 2.25 0 0 0 6 20.25Z"
                            />
                        </svg>
                        "My Pools"
                        <span class="ml-auto text-lg leading-none font-normal">"›"</span>
                    </a>
                </li>
                <li>
                    <a
                        href="#"
                        class="hover:no-underline no-underline flex items-center gap-3 px-3 py-2 rounded text-neutral-200 hover:bg-neutral-700 ease-linear transition-all duration-200"
                    >
                        <svg
                            xmlns="http://www.w3.org/2000/svg"
                            fill="none"
                            viewBox="0 0 24 24"
                            stroke-width="1.5"
                            stroke="currentColor"
                            class="w-6 h-6 stroke-neutral-400"
                        >
                            <path
                                stroke-linecap="round"
                                stroke-linejoin="round"
                                d="M12 6v6h4.5m4.5 0a9 9 0 1 1-18 0 9 9 0 0 1 18 0Z"
                            />
                        </svg>
                        "Activity"
                        <span class="ml-auto text-lg leading-none font-normal">"›"</span>
                    </a>
                </li>
                <li>
                    <div
                        on:click=toggle_menu
                        class="hover:no-underline cursor-default no-underline flex items-center gap-3 px-3 py-2 rounded hover:bg-neutral-700 transition-all ease-linear duration-200"
                    >
                        <svg
                            xmlns="http://www.w3.org/2000/svg"
                            class="stroke-neutral-400"
                            height="24px"
                            viewBox="0 0 24 24"
                            width="24px"
                            stroke-width="1.5"
                            stroke="currentColor"
                            fill="none"
                        >
                            <path
                                stroke-linecap="round"
                                stroke-linejoin="round"
                                d="M10.343 3.94c.09-.542.56-.94 1.11-.94h1.093c.55 0 1.02.398 1.11.94l.149.894c.07.424.384.764.78.93.398.164.855.142 1.205-.108l.737-.527a1.125 1.125 0 0 1 1.45.12l.773.774c.39.389.44 1.002.12 1.45l-.527.737c-.25.35-.272.806-.107 1.204.165.397.505.71.93.78l.893.15c.543.09.94.559.94 1.109v1.094c0 .55-.397 1.02-.94 1.11l-.894.149c-.424.07-.764.383-.929.78-.165.398-.143.854.107 1.204l.527.738c.32.447.269 1.06-.12 1.45l-.774.773a1.125 1.125 0 0 1-1.449.12l-.738-.527c-.35-.25-.806-.272-1.203-.107-.398.165-.71.505-.781.929l-.149.894c-.09.542-.56.94-1.11.94h-1.094c-.55 0-1.019-.398-1.11-.94l-.148-.894c-.071-.424-.384-.764-.781-.93-.398-.164-.854-.142-1.204.108l-.738.527c-.447.32-1.06.269-1.45-.12l-.773-.774a1.125 1.125 0 0 1-.12-1.45l.527-.737c.25-.35.272-.806.108-1.204-.165-.397-.506-.71-.93-.78l-.894-.15c-.542-.09-.94-.56-.94-1.109v-1.094c0-.55.398-1.02.94-1.11l.894-.149c.424-.07.765-.383.93-.78.165-.398.143-.854-.108-1.204l-.526-.738a1.125 1.125 0 0 1 .12-1.45l.773-.773a1.125 1.125 0 0 1 1.45-.12l.737.527c.35.25.807.272 1.204.107.397-.165.71-.505.78-.929l.15-.894Z"
                            />
                            <path
                                stroke-linecap="round"
                                stroke-linejoin="round"
                                d="M15 12a3 3 0 1 1-6 0 3 3 0 0 1 6 0Z"
                            />
                        </svg>
                        "Settings"
                        <span class="ml-auto text-lg leading-none font-normal">"›"</span>
                    </div>
                </li>
            </ul>
            <hr class="m-0 border-neutral-600" />
            // <!-- Token List -->
            <div class="px-1 pt-2">
                // <!-- Wallet Header -->
                <div class="flex items-center gap-3 px-3 py-2 text-neutral-200 font-semibold">
                    <svg
                        xmlns="http://www.w3.org/2000/svg"
                        fill="none"
                        viewBox="0 0 24 24"
                        stroke-width="1.5"
                        stroke="currentColor"
                        class="w-6 h-6 stroke-neutral-400"
                    >
                        <path
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            d="M2.25 8.25h19.5M2.25 9h19.5m-16.5 5.25h6m-6 2.25h3m-3.75 3h15a2.25 2.25 0 0 0 2.25-2.25V6.75A2.25 2.25 0 0 0 19.5 4.5h-15a2.25 2.25 0 0 0-2.25 2.25v10.5A2.25 2.25 0 0 0 4.5 19.5Z"
                        />
                    </svg>
                    // <svg xmlns="http://www.w3.org/2000/svg" height="24px" viewBox="0 -960 960 960" width="24px" fill="#f6c177"><path d="M200-200v-560 560Zm0 80q-33 0-56.5-23.5T120-200v-560q0-33 23.5-56.5T200-840h560q33 0 56.5 23.5T840-760v100h-80v-100H200v560h560v-100h80v100q0 33-23.5 56.5T760-120H200Zm320-160q-33 0-56.5-23.5T440-360v-240q0-33 23.5-56.5T520-680h280q33 0 56.5 23.5T880-600v240q0 33-23.5 56.5T800-280H520Zm280-80v-240H520v240h280Zm-160-60q25 0 42.5-17.5T700-480q0-25-17.5-42.5T640-540q-25 0-42.5 17.5T580-480q0 25 17.5 42.5T640-420Z"/></svg>
                    // <span>"💰"</span>
                    "Wallet"
                </div>
                // <!-- Token Item -->
                <div class="flex items-center justify-between px-3 py-2 rounded hover:bg-neutral-700">
                    <div class="flex items-center gap-3">
                        <img src=format!("{BASE_URL}{}", "/icons/uscrt.png") class="w-6 h-6" />
                        <div>
                            <div class="text-sm font-semibold">SCRT</div>
                            <div class="text-xs text-gray-400">Secret</div>
                        </div>
                    </div>
                    <div class="text-right">
                        <div class="text-sm font-semibold">
                            {move || user_balance.get().and_then(Result::ok)}
                        </div>
                        <div class="text-xs text-gray-400">$0</div>
                    </div>
                </div>

                // <!-- Token Item -->
                <div class="flex items-center justify-between px-3 py-2 rounded hover:bg-neutral-700">
                    <div class="flex items-center gap-3">
                        <img
                            src="https://raw.githubusercontent.com/traderjoe-xyz/joe-tokenlists/main/logos/0xB97EF9Ef8734C71904D8002F8b6Bc66Dd9c48a6E/logo.png"
                            alt="USDC logo"
                            class="w-6 h-6"
                        />
                        <div>
                            <div class="text-sm font-semibold">USDC</div>
                            <div class="text-xs text-gray-400">USD Coin</div>
                        </div>
                    </div>
                    <div class="text-right">
                        <div class="text-sm font-semibold">0</div>
                        <div class="text-xs text-gray-400">$0</div>
                    </div>
                </div>

                // <!-- Token Item -->
                <div class="flex items-center justify-between px-3 py-2 rounded hover:bg-neutral-700">
                    <div class="flex items-center gap-3">
                        <img src=format!("{BASE_URL}{}", "/icons/amber.svg") class="w-6 h-6" />
                        <div>
                            <div class="text-sm font-semibold">AMBER</div>
                            <div class="text-xs text-gray-400">Amber</div>
                        </div>
                    </div>
                    <div class="text-right">
                        <div class="text-sm font-semibold">0</div>
                        <div class="text-xs text-gray-400">$0</div>
                    </div>
                </div>
            </div>
        </dialog>
    }
}

#[component]
pub fn SettingsMenu(
    dialog_ref: NodeRef<Dialog>,
    toggle_menu: impl Fn(MouseEvent) + 'static,
) -> impl IntoView {
    info!("rendering <SettingMenu/>");

    let url_input = NodeRef::<Input>::new();
    let chain_id_input = NodeRef::<Input>::new();

    let endpoint = use_context::<Endpoint>().expect("endpoint context missing!");
    let chain_id = use_context::<ChainId>().expect("chain id context missing!");
    let keplr = use_context::<KeplrSignals>().expect("keplr signals context missing!");

    let disable_keplr = move |_| {
        Keplr::disable(CHAIN_ID);
        keplr.enabled.set(false);
        // keplr.key.set(None);
    };

    // This is an example of using "uncontrolled" inputs. The values are not known by the
    // application until the form is submitted.
    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        // stop the page from reloading!
        ev.prevent_default();
        // here, we'll extract the value from the input
        let value = url_input
            .get()
            // event handlers can only fire after the view
            // is mounted to the DOM, so the `NodeRef` will be `Some`
            .expect("<input> should be mounted")
            // `leptos::HtmlElement<html::Input>` implements `Deref`
            // to a `web_sys::HtmlInputElement`.
            // this means we can call`HtmlInputElement::value()`
            // to get the current value of the input
            .value();
        endpoint.set(value);

        let value = chain_id_input
            .get()
            .expect("<input> should be mounted")
            .value();
        chain_id.set(value);
    };

    view! {
        <dialog node_ref=dialog_ref class="inset-0 rounded border-neutral-200">
            // NOTE: In this case, the effect is so small, it's not worth worrying about.
            // class=("invisible", move || dialog_ref.get().is_some_and(|dialog| !dialog.open()))
            <div class="flex flex-col gap-4 items-center">
                <button on:click=toggle_menu class="self-stretch">
                    "Close Menu"
                </button>
                <SuggestChains />
                // TODO: just use a regular signal setter
                <form class="flex flex-col gap-4" on:submit=on_submit>
                    <div>"Node Endpoint"</div>
                    <input type="text" value=NODE node_ref=url_input class="w-64" />
                    // <input type="text" value=CHAIN_ID node_ref=chain_id_input />
                    <input type="submit" value="Update" class="" />
                </form>
                <button
                    on:click=disable_keplr
                    class="border-blue-500 text-blue-500 border-solid hover:bg-neutral-800 rounded-sm bg-[initial]"
                >
                    Disconnect Wallet
                </button>
            </div>
        </dialog>
    }
}

#[component]
fn ToastMaster() -> impl IntoView {
    info!("rendering <ToastMaster/>");

    view! {
        <div class="toast-container">
            <div class="toast">"Hello"</div>
        </div>
    }
}

#[component]
fn Modal() -> impl IntoView {
    info!("rendering <Modal/>");

    on_cleanup(|| {
        info!("cleaning up <Modal/>");
    });

    let keplr = use_context::<KeplrSignals>().expect("keplr signals context missing!");

    let is_keplr_enabled = move || keplr.enabled.get();
    let my_address = move || {
        keplr
            .key
            .get()
            .and_then(Result::ok)
            .unwrap_or_default()
            .bech32_address
    };

    // Creating a NodeRef allows using methods on the HtmlElement directly
    let dialog_ref = NodeRef::<Dialog>::new();

    let open_modal = move |_| {
        log!("show modal");
        let node = dialog_ref.get().unwrap();
        node.show_modal().expect("I don't know what I expected");
    };
    let close_modal = move |_| {
        log!("close modal");
        let node = dialog_ref.get().unwrap();
        node.close();
    };

    view! {
        <dialog node_ref=dialog_ref>
            <p>"Connected?: "{is_keplr_enabled}</p>
            <p>"Address: "{my_address}</p>
            <button on:click=close_modal>"OK"</button>
        </dialog>
        <button on:click=open_modal>"Example Modal"</button>
    }
}
