// #![allow(unused)]

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
    ev,
    html::{Dialog, Input},
    logging::*,
    prelude::*,
    task::spawn_local,
};
use leptos_meta::*;
use leptos_router::components::{ParentRoute, Redirect, Route, Router, Routes, A};
use leptos_router_macro::path;
use lucide_leptos::{
    ArrowLeft, ArrowUpDown, ChevronRight, ExternalLink, Factory, Flame, FlaskConical, Gauge,
    History, KeyRound, Plus, Power, Router as RouterIcon, Settings, Wallet, Waves, WavesLadder,
    Wrench, X,
};
use tracing::{debug, error, info, warn};
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

use components::{Spinner2, SuggestChains, WalletMenu};
use constants::{CHAIN_ID, NODE, TOKEN_MAP};
use error::Error;
use routes::{nav::Nav, pool::*, trade::*};
use state::{ChainId, Endpoint, KeplrSignals, TokenMap};

// TODO: configure this to be different in dev mode
// pub static BASE_URL: &str = "";
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

    let (keplr_enabled, set_keplr_enabled, _) =
        use_local_storage::<bool, FromToStringCodec>("is_keplr_enabled");

    // TODO: Enable this later. I keep it off during development to prevent needless queries.
    // if keplr_enabled.get() {
    //     keplr.enabled.set(true)
    // }

    // NOTE: For any method on Keplr that returns a promise (almost all of them), if it's Ok,
    // that means keplr is enabled. We can use this fact to update any UI that needs to
    // know if Keplr is enabled. Modifying this signal will cause everything subscribed
    // to react.
    //
    // keplr.enabled.set(true);

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
        // let storage = window()
        //     .local_storage()
        //     .expect("local storage not available?")
        //     .expect("local storage returned none?");
        //
        // match storage.get_item("number_of_lb_pairs") {
        //     Ok(None) => {
        //         let number = LB_FACTORY
        //             .get_number_of_lb_pairs()
        //             .await
        //             .unwrap_or_default();
        //
        //         let _ = storage.set_item("number_of_lb_pairs", &number.to_string());
        //
        //         number
        //     }
        //     Ok(Some(number)) => number.parse::<u32>().unwrap(),
        //     _ => 0,
        // }

        LB_FACTORY
            .get_number_of_lb_pairs()
            .await
            .unwrap_or_default()
    });

    // Effect::new(move || {
    //     if let Some(number) = number_of_lb_pairs.get().as_deref() {
    //         set_id.set(*number)
    //     }
    // });

    async fn batch_query_all_lb_pairs(number: u32) -> Vec<LbPair> {
        let i = number;
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
        let pairs = chain_query::<BatchQueryResponse>(
            BATCH_QUERY_ROUTER.pulsar.code_hash.clone(),
            BATCH_QUERY_ROUTER.pulsar.address.to_string(),
            batch_query_message,
        )
        .await
        .map(parse_batch_query)
        .map(extract_pairs_from_batch)
        .unwrap();

        pairs
    }

    // TODO: how to do time in wasm? "time not implemented on this platform"
    // however, the time thing is really not necessary because the query only runs once per site
    // load, so a user can simply refresh the page to re-run the query.
    // use std::time::{Duration, Instant};

    // let time_of_last_query = RwSignal::new(Instant::now());

    // TODO: currently running this query every time a user visits the site / refreshes the page.
    // makes sense for now but need to think of something else eventually
    let all_lb_pairs: LocalResource<Vec<LbPair>> = LocalResource::new(move || async move {
        // let storage = window()
        //     .local_storage()
        //     .expect("local storage not available?")
        //     .expect("local storage returned none?");

        // let time_since_last_query = Instant::now() - time_of_last_query.get();

        // match storage.get_item("all_lb_pairs") {
        //     Ok(None) => {
        //         let number = number_of_lb_pairs.await;
        //         let pairs = batch_query_all_lb_pairs(number).await;
        //
        //         let _ = storage.set_item("all_lb_pairs", &serde_json::to_string(&pairs).unwrap());
        //
        //         pairs
        //     }
        //     Ok(Some(pairs)) => {
        //         // if time_since_last_query > Duration::from_secs(5) {
        //         //     let number = number_of_lb_pairs.await;
        //         //     let pairs = batch_query_all_lb_pairs(number).await;
        //         //
        //         //     let _ =
        //         //         storage.set_item("all_lb_pairs", &serde_json::to_string(&pairs).unwrap());
        //         //     time_of_last_query.set(Instant::now());
        //         //
        //         //     pairs
        //         // } else {
        //         serde_json::from_str(&pairs).unwrap()
        //     }
        //     _ => vec![],
        // }

        if let Some(number) = number_of_lb_pairs.get() {
            let pairs = batch_query_all_lb_pairs(*number).await;
            pairs
        } else {
            vec![]
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

    // NOTE: If a user has already allowed the site to connect, it still opens the extension (at
    // least the sidebar version) as though to ask for approval, but it just opens normally. Kind
    // of an odd behavior.
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
                        set_keplr_enabled.set(true);
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

    let enable_keplr = move |_: ev::MouseEvent| {
        enable_keplr_action.dispatch(());
    };

    let disable_keplr = move |_: ev::MouseEvent| {
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

    // let theme = RwSignal::new(Theme::dark());

    view! {
        <Router>
            // <div class="background-image"></div>
            <header>
                <div class="pt-2 px-4 flex justify-between items-center">
                    <div
                        id="mainTitle"
                        class="my-2 font-bold text-3xl line-clamp-1 transition-transform duration-300 cursor-default"
                        style="font-feature-settings: \"cv06\" 1, \"cv13\" 1;"
                    >
                        "Liquidity Book"
                    </div>
                    <Show when=move || keplr.key.get().and_then(|key| key.ok()).is_some()>
                        <p class="hidden sm:block text-sm text-muted-foreground leading-none px-4 py-1.5 border border-solid border-muted-foreground rounded-sm">
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
                                        class="min-w-24 text-sm font-medium py-2 px-4 border-none
                                        bg-primary text-primary-foreground rounded-md"
                                    >
                                        <div class="h-6 flex flex-row items-center gap-2">
                                            <Wallet size=16 />
                                            "Connect Wallet"
                                        </div>
                                    </button>
                                }
                            }
                        >
                            <div class="relative inline-block">
                                <button
                                    on:click=toggle_wallet_menu
                                    class="min-w-24 text-sm font-medium leading-none py-2 px-4 border-none
                                    bg-secondary text-secondary-foreground rounded-md"
                                >
                                    <div class="h-6 flex flex-row items-center gap-2">
                                        <Wallet size=16 />
                                        "Wallet Menu"
                                    // {move || key_address().map(shorten_address)}
                                    </div>
                                </button>
                                <WalletMenu
                                    dialog_ref=wallet_dialog_ref
                                    toggle_menu=toggle_options_menu
                                />
                            </div>
                        </Show>
                    </div>
                </div>
                <hr class="mt-2 mb-1 border border-border" />
                <Nav />
                <hr class="mt-1 mb-2 border border-border" />
            // <hr class="mt-1 mb-2 border border-[oklch(0.560_0.012_286)]" />
            </header>
            <main class="px-4 overflow-x-auto">
                <Routes transition=true fallback=|| "This page could not be found.">
                    <Route path=path!("liquidity-book-leptos") view=Trade />
                    <ParentRoute path=path!("/liquidity-book-leptos/pool") view=Pools>
                        <Route path=path!("") view=PoolBrowser />
                        <Route path=path!("create") view=PoolCreator />
                        // TODO: instead of having add/remove liquidity be the nested routes, have
                        // 'manage' and 'analytics' as the nested routes
                        <ParentRoute path=path!("/:token_a/:token_b/:bps") view=Pool>
                            <Route path=path!("") view=|| view! { <Redirect path="manage" /> } />
                            <ParentRoute path=path!("/manage") view=PoolManager>
                                <Route path=path!("") view=|| () />
                                <Route path=path!("add") view=AddLiquidity />
                                <Route path=path!("remove") view=RemoveLiquidity />
                            </ParentRoute>
                            <Route path=path!("analytics") view=PoolAnalytics />
                        </ParentRoute>
                    </ParentRoute>
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
        <dialog node_ref=dialog_ref>
            // prevent focus ring from showing around the main div
            <div tabindex="0"></div>
            // NOTE: when 'display: none' is toggled on/off, some of the animation gets lost,
            // so it's better to use 'visibility: hidden' instead of 'display: none'.
            // Tailwind's 'invisible' = 'visibility: hidden' and 'hidden' = 'display: none'
            // The svg will be spinning invisibly, but it's worth it for the nicer animation.
            // class=("invisible", move || !when.get())
            <div class="align-middle inline-flex items-center justify-center gap-3">
                <Spinner2 size="h-8 w-8" />
                <div class="font-bold">{message}</div>
            </div>
        </dialog>
    }
}

#[component]
pub fn SettingsMenu(
    dialog_ref: NodeRef<Dialog>,
    toggle_menu: impl Fn(ev::MouseEvent) + 'static,
) -> impl IntoView {
    info!("rendering <SettingMenu/>");

    let url_input = NodeRef::<Input>::new();
    let chain_id_input = NodeRef::<Input>::new();

    let endpoint = use_context::<Endpoint>().expect("endpoint context missing!");
    let chain_id = use_context::<ChainId>().expect("chain id context missing!");
    let keplr = use_context::<KeplrSignals>().expect("keplr signals context missing!");

    let disable_keplr = move |_: ev::MouseEvent| {
        Keplr::disable(CHAIN_ID);
        keplr.enabled.set(false);
        // keplr.key.set(None);
    };

    // This is an example of using "uncontrolled" inputs. The values are not known by the
    // application until the form is submitted.
    let on_submit = move |ev: ev::SubmitEvent| {
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
            // <button
            // on:click=disable_keplr
            // class="border-blue-500 text-blue-500 border-solid hover:bg-neutral-800 rounded-sm bg-[initial]"
            // >
            // Disconnect Wallet
            // </button>
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
