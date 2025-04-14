#![allow(unused)]

use crate::prelude::*;
use ammber_components::{Spinner2, SuggestChains, WalletMenu};
use ammber_core::{
    constants::{CHAIN_ID, NODE, TOKEN_MAP},
    state::{ChainId, Endpoint, KeplrSignals, TokenMap},
    support::{chain_batch_query, chain_query},
    Error,
};
use ammber_sdk::contract_interfaces::{
    lb_factory::{self, LbPairAtIndexResponse},
    lb_pair::LbPair,
};
use batch_query::{BatchItemResponseStatus, BatchQueryParams, BatchQueryParsedResponse};
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

mod constants;
mod error;
mod prelude;
mod routes;

use routes::{nav::Nav, pool::*, trade::*};

pub const BASE_URL: &str = "/liquidity-book-leptos";

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

        let pairs = chain_batch_query(queries)
            .await
            .map(extract_pairs_from_batch)
            .unwrap_or_default();

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
            // <div id="background"></div>
            <header class="bg-background z-40">
                <div class="p-4 flex justify-between items-center border-b">
                    <div class="flex flex-row items-center gap-4">
                        <img
                            src=format!("{BASE_URL}{}", "/icons/logo-2.png")
                            class="h-8 hover:rotate-3 transition-transform duration-300 cursor-default"
                        />
                        // <div
                        // id="mainTitle"
                        // class="m-0 font-bold text-3xl line-clamp-1 transition-transform duration-300 cursor-default"
                        // style="font-feature-settings: \"cv06\" 1, \"cv13\" 1;"
                        // >
                        // "Liquidity Book"
                        // </div>
                        <svg
                            class="h-8 fill-foreground -translate-y-0.5"
                            viewBox="0 0 214 48"
                            xmlns="http://www.w3.org/2000/svg"
                        >
                            <path d="M15.0374 47.9313C12.3522 47.9313 9.8683 47.2824 7.58583 45.9845C5.30337 44.6419 3.46845 42.6951 2.08107 40.1441C0.69369 37.5931 0 34.5051 0 30.88V29.8059C0 26.1808 0.69369 23.0927 2.08107 20.5417C3.46845 17.9908 5.30337 16.0663 7.58583 14.7685C9.91305 13.4258 12.3969 12.7545 15.0374 12.7545C18.0807 12.7545 20.3855 13.2692 21.9519 14.2985C23.5631 15.2831 24.7491 16.4244 25.5099 17.7222H26.7183V13.6944H35.0425V37.9959C35.0425 39.3385 35.6691 40.0098 36.9222 40.0098H39.6075V46.9915H32.8943C31.2832 46.9915 29.9406 46.5439 28.8665 45.6488C27.8371 44.7538 27.3224 43.523 27.3224 41.9566V41.8895H26.1141C25.3533 43.5454 24.1225 44.9775 22.4218 46.1859C20.7659 47.3495 18.3045 47.9313 15.0374 47.9313ZM17.5884 40.5469C20.2289 40.5469 22.3995 39.7189 24.1001 38.063C25.8455 36.3624 26.7183 33.9009 26.7183 30.6786V30.0073C26.7183 26.785 25.8455 24.3459 24.1001 22.6899C22.3547 20.9893 20.1841 20.139 17.5884 20.139C14.9927 20.139 12.8221 20.9893 11.0767 22.6899C9.33125 24.3459 8.45854 26.785 8.45854 30.0073V30.6786C8.45854 33.9009 9.33125 36.3624 11.0767 38.063C12.8221 39.7189 14.9927 40.5469 17.5884 40.5469Z" />
                            <path d="M173.107 47.9313C169.795 47.9313 166.864 47.2376 164.313 45.8502C161.807 44.4181 159.837 42.4265 158.405 39.8756C157.018 37.2798 156.324 34.2365 156.324 30.7457V29.9401C156.324 26.4493 157.018 23.4284 158.405 20.8774C159.793 18.2817 161.74 16.2901 164.246 14.9027C166.752 13.4706 169.661 12.7545 172.973 12.7545C176.24 12.7545 179.082 13.493 181.498 14.9699C183.915 16.402 185.795 18.4159 187.138 21.0117C188.48 23.5627 189.151 26.5388 189.151 29.9401V32.8268H164.917C165.007 35.1092 165.857 36.9665 167.468 38.3987C169.079 39.8308 171.048 40.5469 173.376 40.5469C175.748 40.5469 177.493 40.0322 178.612 39.0029C179.731 37.9735 180.581 36.8323 181.163 35.5792L188.077 39.2042C187.451 40.3679 186.533 41.6434 185.325 43.0307C184.161 44.3734 182.595 45.537 180.626 46.5216C178.657 47.4614 176.15 47.9313 173.107 47.9313ZM164.984 26.5164H180.559C180.38 24.592 179.596 23.048 178.209 21.8844C176.866 20.7208 175.099 20.139 172.906 20.139C170.623 20.139 168.811 20.7208 167.468 21.8844C166.125 23.048 165.297 24.592 164.984 26.5164Z" />
                            <path d="M193.995 46.9919V13.6948H202.319V17.4542H203.528C204.02 16.1115 204.825 15.1269 205.944 14.5004C207.108 13.8738 208.45 13.5605 209.972 13.5605H214V21.0793H209.838C207.69 21.0793 205.922 21.6611 204.534 22.8247C203.147 23.9435 202.453 25.6889 202.453 28.0609V46.9919H193.995Z" />
                            <path d="M137.816 47.9317C134.818 47.9317 132.513 47.4171 130.902 46.3877C129.291 45.3584 128.105 44.2171 127.344 42.964H126.136V46.9919H117.811V0H126.27V17.5213H127.478C127.971 16.7157 128.619 15.9549 129.425 15.2388C130.275 14.5227 131.372 13.9409 132.714 13.4934C134.102 13.0011 135.803 12.7549 137.816 12.7549C140.502 12.7549 142.986 13.4263 145.268 14.7689C147.551 16.0668 149.385 17.9912 150.773 20.5422C152.16 23.0932 152.854 26.1812 152.854 29.8063V30.8804C152.854 34.5055 152.16 37.5935 150.773 40.1445C149.385 42.6955 147.551 44.6423 145.268 45.9849C142.986 47.2828 140.502 47.9317 137.816 47.9317ZM135.265 40.5473C137.861 40.5473 140.032 39.7193 141.777 38.0634C143.523 36.3628 144.395 33.9013 144.395 30.679V30.0077C144.395 26.7854 143.523 24.3463 141.777 22.6904C140.077 20.9897 137.906 20.1394 135.265 20.1394C132.67 20.1394 130.499 20.9897 128.754 22.6904C127.008 24.3463 126.136 26.7854 126.136 30.0077V30.679C126.136 33.9013 127.008 36.3628 128.754 38.0634C130.499 39.7193 132.67 40.5473 135.265 40.5473Z" />
                            <path d="M43.243 46.9914V13.6943H51.5673V17.3194H52.7757C53.3575 16.2005 54.3197 15.2383 55.6623 14.4327C57.005 13.5824 58.7728 13.1572 60.9657 13.1572C63.3377 13.1572 65.2397 13.6271 66.6719 14.567C68.104 15.4621 69.2005 16.6481 69.9613 18.1249H71.1697C71.9305 16.6928 73.0046 15.5068 74.392 14.567C75.7793 13.6271 77.7485 13.1572 80.2995 13.1572L80.9216 13.1574C83.2936 13.1574 85.1957 13.6273 86.6278 14.5672C88.0599 15.4623 89.1564 16.6483 89.9172 18.1251H91.1256C91.8864 16.693 92.9605 15.507 94.3479 14.5672C95.7353 13.6273 97.7045 13.1574 100.255 13.1574C102.314 13.1574 104.171 13.605 105.827 14.5001C107.528 15.3504 108.871 16.6706 109.855 18.4608C110.885 20.2062 111.399 22.4215 111.399 25.1068V46.9916H102.941V25.711C102.941 23.8761 102.471 22.5111 101.531 21.616C100.591 20.6761 99.2709 20.2062 97.5702 20.2062C95.6458 20.2062 94.1465 20.8328 93.0724 22.0859C92.0431 23.2943 91.5284 25.0397 91.5284 27.3221V46.9916L82.9848 46.9914V25.7108C82.9848 23.8759 82.5149 22.5109 81.575 21.6158C80.6352 20.6759 79.3149 20.206 77.6143 20.206C75.6898 20.206 74.2756 20.8328 73.2015 22.0859C72.1722 23.2943 71.6575 25.0397 71.6575 27.3221V46.9916L63.1139 46.9914V25.7108C63.1139 23.8759 62.644 22.5109 61.7041 21.6158C60.7643 20.6759 59.4441 20.206 57.7434 20.206C55.819 20.206 54.3197 20.8326 53.2456 22.0857C52.2163 23.2941 51.7016 25.0395 51.7016 27.3219V46.9914H43.243Z" />
                        </svg>
                        <div class="hidden sm:inline-flex">
                            <Nav />
                        </div>
                    </div>
                    <Show when=move || keplr.key.get().and_then(|key| key.ok()).is_some()>
                        <p class="hidden md:block text-sm text-muted-foreground leading-none px-4 py-1.5 border border-solid border-muted-foreground rounded-sm">
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
                <div class="sm:hidden block px-1 py-0.5 border-b">
                    <Nav />
                </div>
            </header>
            <main class="flex-1 px-2.5 lg:px-8 py-3 overflow-x-auto">
                <Routes transition=true fallback=|| "This page could not be found.">
                    <Route path=path!("liquidity-book-leptos") view=Trade />
                    <ParentRoute path=path!("/liquidity-book-leptos/pool") view=Pools>
                        <Route path=path!("") view=PoolBrowser />
                        <Route path=path!("create") view=PoolCreator />
                        <ParentRoute path=path!("/:token_a/:token_b/:bps") view=Pool>
                            <Route path=path!("") view=|| view! { <Redirect path="manage" /> } />
                            <ParentRoute path=path!("/manage") view=PoolManager>
                                <Route path=path!("") view=|| view! { <Redirect path="add" /> } />
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
        endpoint.set(Box::leak(value.into_boxed_str()));

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
