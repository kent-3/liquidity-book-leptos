#![allow(unused)]

// use codee::string::FromToStringCodec;
// use leptos_use::storage::use_local_storage;

use leptos::{
    ev::MouseEvent,
    html::{Dialog, Input},
    logging::log,
    prelude::*,
};
use leptos_router::components::{ParentRoute, Route, Router, Routes, A};
use leptos_router_macro::path;
use prelude::SYMBOL_TO_ADDR;
use rsecret::query::{bank::BankQuerier, compute::ComputeQuerier};
use secret_toolkit_snip20::{QueryMsg, TokenInfoResponse};
use secretrs::utils::EnigmaUtils;
use send_wrapper::SendWrapper;
use tonic_web_wasm_client::Client;
use tracing::{debug, error, info, trace};
use web_sys::{js_sys, wasm_bindgen::JsValue};

mod components;
mod constants;
mod error;
mod keplr;
mod liquidity_book;
mod prelude;
mod routes;
mod state;
mod types;
mod utils;

use components::{Spinner2, SuggestChains};
use constants::{CHAIN_ID, GRPC_URL, TOKEN_MAP};
use error::Error;
use keplr::{Keplr, Key};
use routes::{pool::*, trade::*};
use state::{ChainId, Endpoint, KeplrSignals, TokenMap};
use types::Coin;

#[component]
pub fn App() -> impl IntoView {
    info!("rendering <App/>");

    // Global Contexts

    provide_context(Endpoint::default());
    provide_context(ChainId::default());
    provide_context(KeplrSignals::new());
    provide_context(TokenMap::new());

    let endpoint = use_context::<Endpoint>().expect("endpoint context missing!");
    let chain_id = use_context::<ChainId>().expect("chain id context missing!");
    let keplr = use_context::<KeplrSignals>().expect("keplr signals context missing!");
    let token_map = use_context::<TokenMap>().expect("tokens context missing!");

    debug!("{} Keplr tokens", token_map.len());
    debug!(
        "{:#?}",
        token_map
            .iter()
            .map(|(_, token)| token.metadata.symbol.clone())
            .collect::<Vec<String>>()
    );
    debug!("{} SecretFoundation tokens", TOKEN_MAP.len());
    debug!(
        "{:#?}",
        TOKEN_MAP
            .iter()
            .map(|(_, token)| token.symbol.clone())
            .collect::<Vec<String>>()
    );

    let sscrt_address = SYMBOL_TO_ADDR.get("sSCRT");
    debug!("{sscrt_address:?}");

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

    let options_dialog_ref = NodeRef::<Dialog>::new();

    // HTML Elements

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

    view! {
        <Router>
            <div class="background-image"></div>
            <header>
                <div class="flex justify-between items-center">
                    <div class="my-3 font-bold text-3xl line-clamp-1">"Trader Crow 2"</div>
                    <Show when=move || keplr.key.get().and_then(|key| key.ok()).is_some()>
                        <p class="hidden sm:block text-sm outline outline-2 outline-offset-8 outline-neutral-500">
                            "Connected as "<strong>{key_name}</strong>
                        </p>
                    </Show>
                    <Show
                        when=move || keplr.enabled.get()
                        fallback=move || {
                            view! {
                                <button
                                    on:click=enable_keplr
                                    disabled=enable_keplr_action.pending()
                                >
                                    Connect Wallet
                                </button>
                            }
                        }
                    >
                        <button on:click=toggle_options_menu>"Options"</button>
                    </Show>
                </div>
                <hr />
                <nav>
                    <A href="/trader-crow-leptos/">"Home"</A>
                    <A href="/trader-crow-leptos/pool">"Pool"</A>
                    <A href="/trader-crow-leptos/trade">"Trade"</A>
                    <A href="/trader-crow-leptos/analytics">"Analytics"</A>
                </nav>
                <hr />
            </header>
            <main class="overflow-x-auto">
                <Routes fallback=|| "This page could not be found.">
                    <Route path=path!("/trader-crow-leptos/") view=Home />
                    <ParentRoute path=path!("/trader-crow-leptos/pool") view=Pool>
                        <Route path=path!("/") view=PoolBrowser />
                        <Route path=path!("/create") view=PoolCreator />
                        <ParentRoute path=path!("/:token_a/:token_b/:bps") view=PoolManager>
                            <Route path=path!("/") view=|| () />
                            <Route path=path!("/add") view=AddLiquidity />
                            <Route path=path!("/remove") view=RemoveLiquidity />
                        </ParentRoute>
                    </ParentRoute>
                    <Route path=path!("/trader-crow-leptos/trade") view=Trade />
                </Routes>
            </main>
            <LoadingModal when=enable_keplr_action.pending() message="Requesting Connection" />
            <OptionsMenu dialog_ref=options_dialog_ref toggle_menu=toggle_options_menu />
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
pub fn OptionsMenu(
    dialog_ref: NodeRef<Dialog>,
    toggle_menu: impl Fn(MouseEvent) + 'static,
) -> impl IntoView {
    info!("rendering <OptionsMenu/>");

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
        <dialog node_ref=dialog_ref class="inset-0">
            // NOTE: In this case, the effect is so small, it's not worth worrying about.
            // class=("invisible", move || dialog_ref.get().is_some_and(|dialog| !dialog.open()))
            <div class="flex flex-col gap-4 items-center">
                <button on:click=toggle_menu class="self-stretch">
                    "Close Menu"
                </button>
                <SuggestChains/>
                <div>"Node Configuration"</div>
                <form class="flex flex-col gap-4" on:submit=on_submit>
                    <input type="text" value=GRPC_URL node_ref=url_input class="w-64" />
                    <input type="text" value=CHAIN_ID node_ref=chain_id_input />
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
                            keys.push((
                                token.metadata.name.clone(),
                                token.contract_address.clone(),
                                key,
                            ));
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

    let enigma_utils = EnigmaUtils::new(None, "secret-4").unwrap();

    // TODO: move all static resources like this (query response is always the same) to a separate
    // module. Implement caching with local storage. They can all use a random account for the
    // EncryptionUtils, since they don't depend on user address.
    let contract_address = "secret1s09x2xvfd2lp2skgzm29w2xtena7s8fq98v852";
    let code_hash = "9a00ca4ad505e9be7e6e6dddf8d939b7ec7e9ac8e109c8681f10db9cacb36d42";
    let token_info = Resource::new(
        || (),
        move |_| {
            debug!("loading token_info resource");
            let compute =
                ComputeQuerier::new(Client::new(endpoint.get()), enigma_utils.clone().into());
            SendWrapper::new(async move {
                let query = QueryMsg::TokenInfo {};
                compute
                    .query_secret_contract(contract_address, code_hash, query)
                    .await
                    .map_err(Error::generic)
            })
        },
    );

    view! {
        <div class="max-w-lg">
            <h1 class="font-semibold">Introduction</h1>
            <p>
                "This project presents an efficient Automated Market Maker (AMM)
                protocol, modeled after the Liquidity Book protocol used by Trader Joe ("
                <a
                    href="https://docs.traderjoexyz.com/concepts/concentrated-liquidity"
                    target="_blank"
                    rel="noopener noreferrer"
                >
                    Liquidity Book docs
                </a>"). The protocol retains the key features of its predecessor, such as:"
            </p>
            <ul>
                <li>
                    <strong>No Slippage:</strong>
                    <span>Enabling token swaps with zero slippage within designated bins</span>
                </li>
                <li>
                    <strong>Adaptive Pricing:</strong>
                    <span>
                        Offering Liquidity Providers extra dynamic fees during periods of
                        increased market volatility
                    </span>
                </li>
                <li>
                    <strong>Enhanced Capital Efficiency:</strong>
                    <span>
                        Facilitating high-volume trading with minimal liquidity requirements
                    </span>
                </li>
                <li>
                    <strong>Customizable Liquidity:</strong>
                    <span>
                        Liquidity providers can customize their liquidity distributions
                        based on their strategy
                    </span>
                </li>
            </ul>
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
