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
use routes::{nav::Nav, pool::*, trade::*};
use state::{ChainId, Endpoint, KeplrSignals, TokenMap};
use types::Coin;

// TODO: If possible, use batch queries for resources. Combine the outputs in a struct
// and use that as the return type of the Resource.

#[derive(Clone)]
pub struct NumberOfLbPairs(pub Resource<u32>);

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

    // TODO: How can we get these lb_pair queries to only happen when navigating to the "pool" route,
    // but not re-run every time that page loads? We need some kind of in-memory cache for this,
    // because we do want it to re-run if the user refreshes the page (to load any new pairs).

    let number_of_lb_pairs: Resource<u32> = Resource::new(
        move || (),
        move |_| async {
            LB_FACTORY
                .get_number_of_lb_pairs()
                .await
                .unwrap_or_default()
        },
    );

    let all_lb_pairs: Resource<Vec<LbPair>> = Resource::new(
        move || (),
        move |_| async move {
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

            // TODO: change BATCH_QUERY_ROUTER to automatically know the current chain_id
            chain_query::<BatchQueryResponse>(
                BATCH_QUERY_ROUTER.pulsar.code_hash.clone(),
                BATCH_QUERY_ROUTER.pulsar.address.to_string(),
                batch_query_message,
            )
            .await
            .map(parse_batch_query)
            .map(extract_pairs_from_batch)
            .unwrap()
        },
    );

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
                                        // "⚙"
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
                                    // <svg xmlns="http://www.w3.org/2000/svg" class="fill-white" height="24px" viewBox="0 -960 960 960" width="24px"><path d="M433-80q-27 0-46.5-18T363-142l-9-66q-13-5-24.5-12T307-235l-62 26q-25 11-50 2t-39-32l-47-82q-14-23-8-49t27-43l53-40q-1-7-1-13.5v-27q0-6.5 1-13.5l-53-40q-21-17-27-43t8-49l47-82q14-23 39-32t50 2l62 26q11-8 23-15t24-12l9-66q4-26 23.5-44t46.5-18h94q27 0 46.5 18t23.5 44l9 66q13 5 24.5 12t22.5 15l62-26q25-11 50-2t39 32l47 82q14 23 8 49t-27 43l-53 40q1 7 1 13.5v27q0 6.5-2 13.5l53 40q21 17 27 43t-8 49l-48 82q-14 23-39 32t-50-2l-60-26q-11 8-23 15t-24 12l-9 66q-4 26-23.5 44T527-80h-94Zm7-80h79l14-106q31-8 57.5-23.5T639-327l99 41 39-68-86-65q5-14 7-29.5t2-31.5q0-16-2-31.5t-7-29.5l86-65-39-68-99 42q-22-23-48.5-38.5T533-694l-13-106h-79l-14 106q-31 8-57.5 23.5T321-633l-99-41-39 68 86 64q-5 15-7 30t-2 32q0 16 2 31t7 30l-86 65 39 68 99-42q22 23 48.5 38.5T427-266l13 106Zm42-180q58 0 99-41t41-99q0-58-41-99t-99-41q-59 0-99.5 41T342-480q0 58 40.5 99t99.5 41Zm-2-140Z"/></svg>
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
        <dialog node_ref=dialog_ref class="mr-0 mt-2 border border-neutral-600 rounded py-3 px-0">
            // <!-- Header -->
            <div class="flex items-center justify-between w-72 px-6 pb-3">
                <div class="flex items-center gap-3">
                    <div class="w-8 h-8 flex items-center justify-center bg-transparent outline outline-[1.5px] outline-foam shadow-foam-glow rounded-full">
                        <img class="w-5 h-5" src="/icons/SECRET_FOAM-ICON_RGB.svg" />
                    // <svg class="w-5 h-5 fill-foam" xmlns="http://www.w3.org/2000/svg" version="1.1" viewBox="0 0 308.156 312.193">
                    // <path d="M308.156,94.509c0-4.797-2.467-9.127-6.588-11.583L169.465,4.246c-9.492-5.662-21.283-5.662-30.775,0L6.588,82.927C2.514,85.355.067,89.613.001,94.345c.002,5.785.005,15.249.006,21.035-.165,4.982,2.326,9.514,6.58,12.049l46.44,27.669c.76.453.76,1.554,0,2.006l-46.44,27.659C2.474,187.216.008,191.535,0,196.323c0,5.872,0,15.479,0,21.351,0,4.808,2.467,9.137,6.588,11.583l132.102,78.691c4.746,2.831,10.064,4.246,15.393,4.246s10.636-1.415,15.382-4.246l132.102-78.691c4.121-2.446,6.588-6.775,6.588-11.583,0-5.872,0-15.479,0-21.351-.008-4.788-2.474-9.107-6.588-11.559l-46.44-27.659c-.76-.453-.76-1.554,0-2.006l46.44-27.669c4.121-2.456,6.588-6.785,6.588-11.583v-21.337ZM144.935,137.889c2.82-1.675,5.984-2.519,9.148-2.519s6.317.843,9.138,2.519l30.567,18.213-30.567,18.203c-5.641,3.362-12.645,3.362-18.286,0l-30.567-18.203,30.567-18.213ZM277.58,206.012c.76.453.76,1.554,0,2.007l-114.359,68.113c-5.641,3.362-12.645,3.362-18.286,0L30.575,208.018c-.76-.453-.76-1.554,0-2.006l21.431-12.769c4.188-2.496,6.754-7.011,6.754-11.887v-20.078c0-1.182,1.287-1.914,2.303-1.31l77.627,46.151c4.746,2.82,10.064,4.236,15.393,4.236s10.636-1.416,15.382-4.236l22.651-13.495c4.214-2.511,6.784-7.065,6.755-11.97l-.113-18.898c-.007-1.186,1.284-1.926,2.304-1.318l76.518,45.573ZM249.145,150.749c0,1.182-1.287,1.914-2.303,1.31l-77.377-45.986c-9.492-5.651-21.283-5.651-30.775,0l-22.796,13.578c-4.215,2.511-6.785,7.065-6.756,11.971l.117,19.341c.005.909-.984,1.475-1.765,1.01L30.576,106.181c-.76-.453-.761-1.554,0-2.007l114.359-68.113c2.82-1.686,5.984-2.529,9.148-2.529s6.317.843,9.138,2.529l114.359,68.113c.76.453.76,1.554,0,2.007l-21.68,12.915c-4.189,2.495-6.755,7.011-6.755,11.887v19.765Z"/>
                    // </svg>
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
                    class="w-10 h-10 p-0 bg-transparent active:bg-neutral-900 hover:bg-neutral-700 hover:outline-gold hover:saturate-150 hover:shadow-gold-glow transition-all ease-standard duration-200 rounded-full inline-flex items-center justify-center outline outline-[1.5px] outline-offset-0 outline-transparent border border-solid border-neutral-700"
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
                // <svg
                // width="16"
                // height="18"
                // viewBox="0 0 16 18"
                // fill="none"
                // xmlns="http://www.w3.org/2000/svg"
                // focusable="false"
                // class="w-4 h-4 inline-block fill-gold"
                // aria-hidden="true"
                // >
                // <path
                // fill-rule="evenodd"
                // clip-rule="evenodd"
                // d="M9 1C9 0.447715 8.55229 0 8 0C7.44772 0 7 0.447715 7 1V6C7 6.55228 7.44772 7 8 7C8.55229 7 9 6.55228 9 6V1ZM4.9989 4.86666C5.47754 4.59113 5.64219 3.97975 5.36666 3.5011C5.09113 3.02246 4.47975 2.85781 4.0011 3.13334C1.61296 4.50808 0 7.08185 0 10.034C0 14.4381 3.58632 18 8 18C12.4137 18 16 14.4381 16 10.034C16 7.08185 14.387 4.50808 11.9989 3.13334C11.5203 2.85781 10.9089 3.02246 10.6333 3.5011C10.3578 3.97975 10.5225 4.59113 11.0011 4.86666C12.7976 5.90081 14 7.82945 14 10.034C14 13.3244 11.3183 16 8 16C4.68169 16 2 13.3244 2 10.034C2 7.82945 3.20243 5.90081 4.9989 4.86666Z"
                // fill="current"
                // ></path>
                // </svg>
                </button>
            </div>
            <hr class="m-0 border-neutral-600" />
            // <!-- Menu Items -->
            <ul class="space-y-1 px-1 py-2 list-none font-semibold text-base">
                <li>
                    <a
                        href="#"
                        class="hover:no-underline no-underline flex items-center gap-3 px-3 py-2 rounded text-neutral-200 hover:bg-neutral-800 ease-linear transition-all duration-200"
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
                        // <Icon icon=icondata::MdiPool height="1.25rem" width="1.25rem" style="color: #f6c177;" />
                        // <svg xmlns="http://www.w3.org/2000/svg" height="24px" viewBox="0 -960 960 960" width="24px" fill="#f6c177"><path d="M213-280q-29 0-42 12.5T135-250q-23 5-39-4.5T80-283q0-15 13-29t41-28q5-3 26.5-11.5T217-360q55 0 74 20t56 20q37 0 55.5-20t76.5-20q58 0 78.5 20t57.5 20q37 0 54-20t74-20q23 0 44.5 5.5T831-338q25 13 37 27t12 28q0 19-16 28.5t-39 4.5q-23-5-36.5-17.5T747-280q-37 0-55.5 20T615-240q-58 0-78.5-20T479-280q-37 0-55.5 20T346-240q-59 0-77-20t-56-20Zm0-160q-28 0-41.5 12.5T135-410q-23 5-39-4.5T80-443q0-15 13-29t41-28q5-3 26.5-11.5T217-520q55 0 74 20t56 20q37 0 55.5-20t76.5-20q58 0 78.5 20t56.5 20q36 0 54-20t74-20q35 0 56.5 8.5T825-500q29 15 42 28.5t13 28.5q0 19-16.5 28.5T824-410q-23-5-36-17.5T747-440q-37 0-55.5 20T615-400q-58 0-78.5-20T479-440q-37 0-54.5 20T348-400q-59 0-78.5-20T213-440Zm0-160q-28 0-41.5 12.5T135-570q-23 5-39-4.5T80-603q0-15 13-29t41-28q5-3 26.5-11.5T217-680q55 0 74 20t56 20q37 0 55.5-20t76.5-20q58 0 78.5 20t56.5 20q36 0 54-20t74-20q35 0 56.5 8.5T825-660q29 15 42 28.5t13 28.5q0 19-16.5 28.5T824-570q-23-5-36-17.5T747-600q-37 0-55.5 20T615-560q-58 0-78.5-20T479-600q-37 0-54.5 20T348-560q-59 0-78.5-20T213-600Z"/></svg>
                        // <span>"🌊"</span>
                        "My Pools"
                        <span class="ml-auto text-lg leading-none font-normal">"›"</span>
                    </a>
                </li>
                <li>
                    <a
                        href="#"
                        class="hover:no-underline no-underline flex items-center gap-3 px-3 py-2 rounded text-neutral-200 hover:bg-neutral-800 ease-linear transition-all duration-200"
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
                        // <svg xmlns="http://www.w3.org/2000/svg" height="24px" viewBox="0 -960 960 960" width="24px" fill="#f6c177"><path d="M480-120q-126 0-223-76.5T131-392q-4-15 6-27.5t27-14.5q16-2 29 6t18 24q24 90 99 147t170 57q117 0 198.5-81.5T760-480q0-117-81.5-198.5T480-760q-69 0-129 32t-101 88h70q17 0 28.5 11.5T360-600q0 17-11.5 28.5T320-560H160q-17 0-28.5-11.5T120-600v-160q0-17 11.5-28.5T160-800q17 0 28.5 11.5T200-760v54q51-64 124.5-99T480-840q75 0 140.5 28.5t114 77q48.5 48.5 77 114T840-480q0 75-28.5 140.5t-77 114q-48.5 48.5-114 77T480-120Zm40-376 100 100q11 11 11 28t-11 28q-11 11-28 11t-28-11L452-452q-6-6-9-13.5t-3-15.5v-159q0-17 11.5-28.5T480-680q17 0 28.5 11.5T520-640v144Z"/></svg>
                        // <span>"🔄"</span>
                        "Activity"
                        <span class="ml-auto text-lg leading-none font-normal">"›"</span>
                    </a>
                </li>
                <li>
                    <div
                        on:click=toggle_menu
                        class="hover:no-underline cursor-default no-underline flex items-center gap-3 px-3 py-2 rounded hover:bg-neutral-800 transition-all ease-linear duration-200"
                    >
                        // <svg xmlns="http://www.w3.org/2000/svg" height="24px" viewBox="0 -960 960 960" width="24px" fill="#f6c177"><path d="M433-80q-27 0-46.5-18T363-142l-9-66q-13-5-24.5-12T307-235l-62 26q-25 11-50 2t-39-32l-47-82q-14-23-8-49t27-43l53-40q-1-7-1-13.5v-27q0-6.5 1-13.5l-53-40q-21-17-27-43t8-49l47-82q14-23 39-32t50 2l62 26q11-8 23-15t24-12l9-66q4-26 23.5-44t46.5-18h94q27 0 46.5 18t23.5 44l9 66q13 5 24.5 12t22.5 15l62-26q25-11 50-2t39 32l47 82q14 23 8 49t-27 43l-53 40q1 7 1 13.5v27q0 6.5-2 13.5l53 40q21 17 27 43t-8 49l-48 82q-14 23-39 32t-50-2l-60-26q-11 8-23 15t-24 12l-9 66q-4 26-23.5 44T527-80h-94Zm7-80h79l14-106q31-8 57.5-23.5T639-327l99 41 39-68-86-65q5-14 7-29.5t2-31.5q0-16-2-31.5t-7-29.5l86-65-39-68-99 42q-22-23-48.5-38.5T533-694l-13-106h-79l-14 106q-31 8-57.5 23.5T321-633l-99-41-39 68 86 64q-5 15-7 30t-2 32q0 16 2 31t7 30l-86 65 39 68 99-42q22 23 48.5 38.5T427-266l13 106Zm42-180q58 0 99-41t41-99q0-58-41-99t-99-41q-59 0-99.5 41T342-480q0 58 40.5 99t99.5 41Zm-2-140Z"/></svg>
                        // <span>"⚙️"</span>
                        // <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-6 h-6 stroke-neutral-400">
                        // <path stroke-linecap="round" stroke-linejoin="round" d="M4.5 12a7.5 7.5 0 0 0 15 0m-15 0a7.5 7.5 0 1 1 15 0m-15 0H3m16.5 0H21m-1.5 0H12m-8.457 3.077 1.41-.513m14.095-5.13 1.41-.513M5.106 17.785l1.15-.964m11.49-9.642 1.149-.964M7.501 19.795l.75-1.3m7.5-12.99.75-1.3m-6.063 16.658.26-1.477m2.605-14.772.26-1.477m0 17.726-.26-1.477M10.698 4.614l-.26-1.477M16.5 19.794l-.75-1.299M7.5 4.205 12 12m6.894 5.785-1.149-.964M6.256 7.178l-1.15-.964m15.352 8.864-1.41-.513M4.954 9.435l-1.41-.514M12.002 12l-3.75 6.495" />
                        // </svg>
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
                <div class="flex items-center justify-between px-3 py-2 rounded hover:bg-neutral-800">
                    <div class="flex items-center gap-3">
                        // <img src="/icons/scrt-black-192.png" alt="SCRT logo" class="w-6 h-6" />
                        // <svg width="24" height="24" viewBox="0 0 40 40" fill="none" xmlns="http://www.w3.org/2000/svg">
                        // <path d="M34.1427 5.86C30.3634 2.08 25.3429 0 20.0021 0C8.97208 0 0 8.972 0 20C0 31.028 8.97208 40 20.0021 40C25.3429 40 30.3674 37.92 34.1427 34.14C37.9221 30.36 40 25.34 40 20C40 14.66 37.9221 9.636 34.1427 5.86ZM20.0021 38.616C9.73677 38.616 1.38525 30.264 1.38525 20C1.38525 9.736 9.73677 1.384 20.0021 1.384C30.2672 1.384 38.6187 9.736 38.6187 20C38.6187 30.264 30.2672 38.616 20.0021 38.616Z" fill="#FF3912"/>
                        // <path d="M20.002 38.1654C9.89834 38.1654 1.67834 29.9449 1.67834 19.8425C1.67834 9.74016 9.89834 1.51968 20.002 1.51968C30.1055 1.51968 38.3255 9.74016 38.3255 19.8425C38.3255 29.9449 30.1055 38.1654 20.002 38.1654Z" fill="#040506"/>
                        // <path d="M30.7155 15.66C30.7155 15.3219 30.5416 15.0168 30.2512 14.8437L20.9421 9.29925C20.2732 8.90025 19.4423 8.90025 18.7734 9.29925L9.4643 14.8438C9.17721 15.0149 9.00477 15.315 9.00012 15.6484C9.00026 16.0561 9.00047 16.723 9.00054 17.1307C8.98892 17.4818 9.16445 17.8012 9.46423 17.9798L12.7368 19.9296C12.7904 19.9615 12.7904 20.0391 12.7368 20.071L9.46423 22.0201C9.17439 22.1929 9.00061 22.4973 9.00005 22.8347C9.00005 23.2485 9.00005 23.9255 9.00005 24.3393C9.00005 24.6781 9.1739 24.9832 9.4643 25.1555L18.7734 30.7008C19.1078 30.9003 19.4826 31 19.8581 31C20.2336 31 20.6076 30.9003 20.9421 30.7008L30.2511 25.1555C30.5415 24.9832 30.7154 24.6781 30.7154 24.3393C30.7154 23.9255 30.7154 23.2485 30.7154 22.8347C30.7148 22.4973 30.541 22.1929 30.2511 22.0202L26.9786 20.0711C26.925 20.0391 26.925 19.9615 26.9786 19.9297L30.2511 17.9799C30.5415 17.8068 30.7154 17.5018 30.7154 17.1636V15.6601L30.7155 15.66ZM19.2135 18.7169C19.4122 18.5989 19.6351 18.5394 19.8581 18.5394C20.0811 18.5394 20.3033 18.5988 20.5021 18.7169L22.6561 20.0004L20.5021 21.2831C20.1045 21.52 19.611 21.52 19.2135 21.2831L17.0594 20.0004L19.2135 18.7169ZM28.5608 23.5175C28.6144 23.5494 28.6144 23.627 28.5608 23.6589L20.5021 28.4587C20.1045 28.6957 19.611 28.6957 19.2135 28.4587L11.1546 23.6588C11.1011 23.6269 11.1011 23.5493 11.1546 23.5175L12.6649 22.6177C12.96 22.4418 13.1408 22.1236 13.1408 21.78V20.3651C13.1408 20.2818 13.2315 20.2302 13.3031 20.2728L18.7734 23.525C19.1078 23.7237 19.4826 23.8235 19.8581 23.8235C20.2336 23.8235 20.6076 23.7237 20.9421 23.525L22.5383 22.574C22.8352 22.3971 23.0163 22.0762 23.0143 21.7305L23.0063 20.3988C23.0058 20.3152 23.0968 20.2631 23.1687 20.3059L28.5608 23.5175ZM26.557 19.6232C26.557 19.7064 26.4663 19.758 26.3947 19.7155L20.9421 16.4749C20.2732 16.0767 19.4423 16.0767 18.7734 16.4749L17.167 17.4317C16.8699 17.6087 16.6888 17.9296 16.6909 18.2753L16.6991 19.6382C16.6995 19.7023 16.6298 19.7422 16.5748 19.7094L11.1547 16.4825C11.1012 16.4506 11.1011 16.373 11.1547 16.3411L19.2135 11.5412C19.4122 11.4224 19.6351 11.363 19.8581 11.363C20.0811 11.363 20.3033 11.4224 20.5021 11.5412L28.5608 16.3411C28.6144 16.373 28.6144 16.4506 28.5608 16.4825L27.033 17.3926C26.7378 17.5684 26.557 17.8867 26.557 18.2303V19.6232Z" fill="#FF3912"/>
                        // </svg>

                        // <svg xmlns="http://www.w3.org/2000/svg" fill="#FF3912" width="24" height="24" viewBox="0 0 308.156 312.193">
                        // <path class="cls-1" d="M308.156,94.509c0-4.797-2.467-9.127-6.588-11.583L169.465,4.246c-9.492-5.662-21.283-5.662-30.775,0L6.588,82.927C2.514,85.355.067,89.613.001,94.345c.002,5.785.005,15.249.006,21.035-.165,4.982,2.326,9.514,6.58,12.049l46.44,27.669c.76.453.76,1.554,0,2.006l-46.44,27.659C2.474,187.216.008,191.535,0,196.323c0,5.872,0,15.479,0,21.351,0,4.808,2.467,9.137,6.588,11.583l132.102,78.691c4.746,2.831,10.064,4.246,15.393,4.246s10.636-1.415,15.382-4.246l132.102-78.691c4.121-2.446,6.588-6.775,6.588-11.583,0-5.872,0-15.479,0-21.351-.008-4.788-2.474-9.107-6.588-11.559l-46.44-27.659c-.76-.453-.76-1.554,0-2.006l46.44-27.669c4.121-2.456,6.588-6.785,6.588-11.583v-21.337ZM144.935,137.889c2.82-1.675,5.984-2.519,9.148-2.519s6.317.843,9.138,2.519l30.567,18.213-30.567,18.203c-5.641,3.362-12.645,3.362-18.286,0l-30.567-18.203,30.567-18.213ZM277.58,206.012c.76.453.76,1.554,0,2.007l-114.359,68.113c-5.641,3.362-12.645,3.362-18.286,0L30.575,208.018c-.76-.453-.76-1.554,0-2.006l21.431-12.769c4.188-2.496,6.754-7.011,6.754-11.887v-20.078c0-1.182,1.287-1.914,2.303-1.31l77.627,46.151c4.746,2.82,10.064,4.236,15.393,4.236s10.636-1.416,15.382-4.236l22.651-13.495c4.214-2.511,6.784-7.065,6.755-11.97l-.113-18.898c-.007-1.186,1.284-1.926,2.304-1.318l76.518,45.573ZM249.145,150.749c0,1.182-1.287,1.914-2.303,1.31l-77.377-45.986c-9.492-5.651-21.283-5.651-30.775,0l-22.796,13.578c-4.215,2.511-6.785,7.065-6.756,11.971l.117,19.341c.005.909-.984,1.475-1.765,1.01L30.576,106.181c-.76-.453-.761-1.554,0-2.007l114.359-68.113c2.82-1.686,5.984-2.529,9.148-2.529s6.317.843,9.138,2.529l114.359,68.113c.76.453.76,1.554,0,2.007l-21.68,12.915c-4.189,2.495-6.755,7.011-6.755,11.887v19.765Z"/>
                        // </svg>

                        <img src="/icons/uscrt.png" class="w-6 h-6" />
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
                <div class="flex items-center justify-between px-3 py-2 rounded hover:bg-neutral-800">
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
                <div class="flex items-center justify-between px-3 py-2 rounded hover:bg-neutral-800">
                    <div class="flex items-center gap-3">
                        <img src="/icons/amber.svg" alt="AMBER logo" class="w-6 h-6" />
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
fn Home() -> impl IntoView {
    info!("rendering <Home/>");

    let endpoint = use_context::<Endpoint>().expect("endpoint context missing!");
    let keplr = use_context::<KeplrSignals>().expect("keplr signals context missing!");
    let token_map = use_context::<TokenMap>().expect("tokens context missing!");

    let viewing_keys = Resource::new(
        move || keplr.key.track(),
        move |_| {
            let tokens = token_map.clone();
            SendWrapper::new(async move {
                if keplr.enabled.get_untracked() {
                    debug!("gathering viewing_keys");
                    let mut keys = Vec::new();
                    for (_, token) in tokens.iter() {
                        let key_result =
                            Keplr::get_secret_20_viewing_key(CHAIN_ID, &token.contract_address)
                                .await;

                        if let Ok(key) = key_result {
                            keys.push((token.name.clone(), token.contract_address.clone(), key));
                        }
                    }
                    debug!("Found {} viewing keys.", keys.len());
                    keys
                } else {
                    vec![]
                }
            })
        },
    );

    let viewing_keys_list = move || {
        Suspend::new(async move {
            viewing_keys
                .await
                .into_iter()
                .map(|(name, address, key)| {
                    view! {
                        <li>
                            <strong>{name}</strong>
                            ", "
                            {address}
                            ": "
                            {key}
                        </li>
                    }
                })
                .collect_view()
        })
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
            })
        },
    );

    // TODO: move all static resources like this (query response is always the same) to a separate
    // module. Implement caching with local storage. They can all use a random account for the
    // EncryptionUtils, since they don't depend on user address.

    // let enigma_utils = EnigmaUtils::new(None, "secret-4").unwrap();
    // let contract_address = "secret1s09x2xvfd2lp2skgzm29w2xtena7s8fq98v852";
    // let code_hash = "9a00ca4ad505e9be7e6e6dddf8d939b7ec7e9ac8e109c8681f10db9cacb36d42";
    // let token_info = Resource::new(
    //     || (),
    //     move |_| {
    //         debug!("loading token_info resource");
    //         let compute =
    //             ComputeQuerier::new(Client::new(endpoint.get()), enigma_utils.clone().into());
    //         SendWrapper::new(async move {
    //             let query = QueryMsg::TokenInfo {};
    //             compute
    //                 .query_secret_contract(contract_address, code_hash, query)
    //                 .await
    //                 .map_err(Error::generic)
    //         })
    //     },
    // );

    view! {
        <div class="p-2 max-w-lg">
            <div class="text-3xl font-bold mb-4">"Introduction"</div>
            <p>
                "This project presents an efficient Automated Market Maker (AMM)
                protocol, modeled after the Liquidity Book protocol used by Trader Joe ("
                <a
                    href="https://docs.traderjoexyz.com/concepts/concentrated-liquidity"
                    target="_blank"
                    rel="noopener noreferrer"
                >
                    "Liquidity Book docs"
                </a>"). The protocol retains the key features of its predecessor, such as:"
            </p>
            <ul>
                <li>
                    <strong>"No Slippage: "</strong>
                    <span>"Enabling token swaps with zero slippage within designated bins"</span>
                </li>
                <li>
                    <strong>"Adaptive Pricing: "</strong>
                    <span>
                        "Offering Liquidity Providers extra dynamic fees during periods of
                        increased market volatility"
                    </span>
                </li>
                <li>
                    <strong>"Enhanced Capital Efficiency: "</strong>
                    <span>
                        "Facilitating high-volume trading with minimal liquidity requirements"
                    </span>
                </li>
                <li>
                    <strong>"Customizable Liquidity: "</strong>
                    <span>
                        "Liquidity providers can customize their liquidity distributions
                        based on their strategy"
                    </span>
                </li>
            </ul>
        </div>
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
