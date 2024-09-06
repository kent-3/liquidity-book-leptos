#![allow(unused)]

// use codee::string::FromToStringCodec;
// use leptos_use::storage::use_local_storage;

use leptos::{
    ev::MouseEvent,
    html::{Dialog, Input},
    logging::log,
    prelude::*,
};
use leptos_router::{
    components::{FlatRoutes, Route, Router, Routes, A},
    StaticSegment,
};
use leptos_router_macro::path;
use secret_toolkit_snip20::{QueryMsg, TokenInfoResponse};
use send_wrapper::SendWrapper;
use tonic_web_wasm_client::Client;
use tracing::{debug, error, info};
use web_sys::{js_sys, wasm_bindgen::JsValue};

use rsecret::{
    query::{bank::BankQuerier, compute::ComputeQuerier},
    secret_network_client::CreateQuerierOptions,
};

mod components;
mod constants;
mod error;
mod keplr;
mod prelude;
mod routes;
mod state;
mod types;
mod utils;

use components::Spinner2;
use constants::{CHAIN_ID, GRPC_URL};
use error::Error;
use keplr::{keplr_sys, Keplr, Key};
use routes::{pool::*, trade::*};
use state::{KeplrSignals, TokenMap, WasmClient};
use types::Coin;

#[component]
pub fn App() -> impl IntoView {
    info!("rendering <App/>");

    // Global Context

    let keplr = KeplrSignals::new();
    let wasm_client = WasmClient::new();
    let token_map = TokenMap::new();
    debug!("Loaded {} tokens", token_map.len());

    provide_context(keplr);
    provide_context(wasm_client);
    provide_context(token_map);

    let keplr = use_context::<KeplrSignals>().expect("keplr signals context missing!");
    let wasm_client = use_context::<WasmClient>().expect("wasm client context missing!");
    let token_map = use_context::<TokenMap>().expect("tokens context missing!");

    // Event Listeners

    let keplr_keystorechange_handle =
        window_event_listener_untyped("keplr_keystorechange", move |_| {
            warn!("Key store in Keplr is changed. You may need to refetch the account info.");
        });

    // let update_grpc_url = move |_| {
    //     debug!("updating client_options.grpc_url");
    //     wasm_client.set(Client::new("https://foobar.com".to_string()))
    // };

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

    let enable_keplr = move |_| {
        enable_keplr_action.dispatch(());
    };

    // let disable_keplr = move |_| {
    //     keplr_sys::disable(CHAIN_ID);
    //     keplr.enabled.set(false);
    //     keplr.key.set(None);
    // };

    // Node references

    let options_dialog_ref = NodeRef::<Dialog>::new();

    // Effects

    // open the dialog whenever the "enable_keplr_action" is pending
    // Effect::new(move |_| match dialog_ref.get() {
    //     Some(dialog) => match enable_keplr_action.pending().get() {
    //         true => {
    //             let _ = dialog.show_modal();
    //         }
    //         false => dialog.close(),
    //     },
    //     None => (),
    // });

    Owner::on_cleanup(move || {
        info!("cleaning up <Aoo/>");
        keplr_keystorechange_handle.remove()
    });

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
                    <div class="my-3 font-bold text-3xl">"Trader Crow 2"</div>
                    // terrible, but it works...
                    <Show when=move || {
                        keplr.key.get().map(|key| key.is_ok()).unwrap_or_default()
                    }>
                        <p class="text-sm outline outline-2 outline-offset-8 outline-neutral-500">
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
                    <A href="/">"Home"</A>
                    <A href="/pool">"Pool"</A>
                    <A href="/trade">"Trade"</A>
                    <A href="/analytics">"Analytics"</A>
                </nav>
                <hr />
            </header>
            <main class="overflow-x-auto">
                <FlatRoutes fallback=|| "This page could not be found.">
                    <Route path=path!("/") view=Home />
                    <Route path=path!("/pool") view=Pool />
                    <Route path=path!("/pool/:token_a/:token_b/:basis_points") view=PoolManager />
                    <Route path=path!("/pool/create") view=PoolCreator />
                    <Route path=path!("/trade") view=Trade />
                </FlatRoutes>
            </main>
            <LoadingModal when=enable_keplr_action.pending() message="Requesting Connection" />
            <OptionsMenu dialog_ref=options_dialog_ref toggle_menu=toggle_options_menu />
        </Router>
    }
}

#[component]
pub fn LoadingModal(when: Memo<bool>, #[prop(into)] message: String) -> impl IntoView {
    let dialog_ref = NodeRef::<Dialog>::new();

    Effect::new_sync(move |_| match dialog_ref.get() {
        Some(dialog) => match when.get() {
            true => {
                let _ = dialog.show_modal();
            }
            false => dialog.close(),
        },
        None => (),
    });

    view! {
        <dialog node_ref=dialog_ref class="absolute inset-0 flex items-center">
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

    // let dialog_ref = NodeRef::<Dialog>::new();
    let input_element = NodeRef::<Input>::new();

    let keplr = use_context::<KeplrSignals>().expect("keplr signals context missing!");
    let wasm_client = use_context::<WasmClient>().expect("wasm client context missing!");

    let disable_keplr = move |_| {
        keplr_sys::disable(CHAIN_ID);
        keplr.enabled.set(false);
        // keplr.key.set(None);
    };

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        // stop the page from reloading!
        ev.prevent_default();

        debug!("updating wasm_client grpc_url");

        // here, we'll extract the value from the input
        let value = input_element
            .get()
            // event handlers can only fire after the view
            // is mounted to the DOM, so the `NodeRef` will be `Some`
            .expect("<input> should be mounted")
            // `leptos::HtmlElement<html::Input>` implements `Deref`
            // to a `web_sys::HtmlInputElement`.
            // this means we can call`HtmlInputElement::value()`
            // to get the current value of the input
            .value();
        wasm_client.set(Client::new(value));
    };

    view! {
        <dialog node_ref=dialog_ref class="absolute inset-0 flex flex-col gap-4 items-center">
            <button on:click=toggle_menu class="self-stretch">
                "Close Menu"
            </button>
            <form class="flex gap-4" on:submit=on_submit>
                <input type="text" value=GRPC_URL node_ref=input_element />
                <input type="submit" value="Submit" class="min-w-fit" />
            </form>
            <button
                on:click=disable_keplr
                class="border-blue-500 text-blue-500 border-solid hover:bg-neutral-800 rounded-sm bg-[initial]"
            >
                Disconnect Wallet
            </button>
        </dialog>
    }
}

#[component]
fn Home() -> impl IntoView {
    info!("rendering <Home/>");

    let keplr = use_context::<KeplrSignals>().expect("keplr signals context missing!");
    let wasm_client = use_context::<WasmClient>().expect("wasm client context missing!");
    let token_map = use_context::<TokenMap>().expect("tokens context missing!");

    // whenever the key store changes, this will re-set 'is_keplr_enabled' to true, triggering a
    // reload of everything subscribed to that signal
    let keplr_keystorechange_handle =
        window_event_listener_untyped("keplr_keystorechange", move |_| {
            keplr.enabled.set(true);
        });

    on_cleanup(move || {
        info!("cleaning up <Home/>");
        keplr_keystorechange_handle.remove()
    });

    // Effect::new(move |_| {
    //     if keplr.enabled.get() {
    //         spawn_local(async move {
    //             let key: Option<Key> = Keplr::get_key(CHAIN_ID).await.ok();
    //             keplr.key.set(key);
    //         })
    //     }
    // });

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
        move || keplr.key.get(),
        move |key| {
            SendWrapper::new(async move {
                if let Some(Ok(key)) = key {
                    let bank = BankQuerier::new(wasm_client.get_untracked());
                    match bank.balance(key.bech32_address, "uscrt").await {
                        Ok(balance) => {
                            let balance: Coin = balance.balance.unwrap().into();
                            Ok(balance.to_string())
                        }
                        // TODO: do better with these Error semantics
                        Err(error) => {
                            error!("{error}");
                            Err(Error::from(error))
                        }
                    }
                } else {
                    Err(Error::generic("no wallet key found"))
                }
            })
        },
    );

    let encryption_utils = secretrs::EncryptionUtils::new(None, "secret-4").unwrap();
    // TODO: revisit this. url is not needed, EncryptionUtils should be a trait
    let options = CreateQuerierOptions {
        url: "https://grpc.mainnet.secretsaturn.net",
        chain_id: CHAIN_ID,
        encryption_utils,
    };

    // TODO: move all static resources like this (query response is always the same) to a separate
    // module. Implement caching with local storage. They can all use a random account for the
    // EncryptionUtils, since they don't depend on user address.
    let contract_address = "secret1s09x2xvfd2lp2skgzm29w2xtena7s8fq98v852";
    let code_hash = "9a00ca4ad505e9be7e6e6dddf8d939b7ec7e9ac8e109c8681f10db9cacb36d42";
    let token_info = Resource::new(
        || (),
        move |_| {
            debug!("loading token_info resource");
            let compute = ComputeQuerier::new(wasm_client.get_untracked(), options.clone());
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
        <Show when=move || keplr.enabled.get() fallback=|| view! { <p>Nothing to see here</p> }>
            <pre>
                {move || {
                    format!("{:#?}", keplr.key.get().and_then(Result::ok).unwrap_or_default())
                }}
            </pre>
            // Errors related to general chain queries
            // the fallback receives a signal containing current errors
            <ErrorBoundary fallback=|errors| {
                view! {
                    <div class="error">
                        <p>"Errors: "</p>
                        // we can render a list of errors as strings, if we'd like
                        <ul>
                            {move || {
                                errors
                                    .get()
                                    .into_iter()
                                    .map(|(_, e)| view! { <li>{e.to_string()}</li> })
                                    .collect_view()
                            }}
                        </ul>
                    </div>
                }
            }>
                <p>{move || token_info.get()}</p>
            </ErrorBoundary>
            // Errors from user-specific queries should have a separate ErrorBoundary
            <ErrorBoundary fallback=|errors| {
                view! {
                    <div class="error">
                        <p>"Errors: "</p>
                        <ul>
                            {move || {
                                errors
                                    .get()
                                    .into_iter()
                                    .map(|(_, e)| view! { <li>{e.to_string()}</li> })
                                    .collect_view()
                            }}
                        </ul>
                    </div>
                }
            }>
                <Suspense fallback=move || view! { <p>"Loading (user_balance)..."</p> }>
                    <p>{move || Suspend::new(async move { user_balance.await })}</p>
                </Suspense>
            </ErrorBoundary>
            <Suspense>
                <h2>"Viewing Keys"</h2>
                <ul class="overflow-x-auto">{viewing_keys_list}</ul>
            </Suspense>
        </Show>
    }
}

#[component]
fn Modal(// Signal that will be toggled when the button is clicked.
    // setter: WriteSignal<bool>,
) -> impl IntoView {
    info!("rendering <Modal/>");

    on_cleanup(|| {
        info!("cleaning up <Modal/>");
    });

    // Examples using write signal as prop
    // setter.set(true);
    // setter.update(|value| *value = !*value);

    // Example using read signal from context
    // let getter =
    //     use_context::<ReadSignal<bool>>().expect("there to be an 'enabled' signal provided");

    let keplr = use_context::<KeplrSignals>().expect("keplr signals context missing!");
    let wasm_client = use_context::<WasmClient>().expect("wasm client context missing!");

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

        // Example using context
        // ctx.keplr_enabled.update(|value| *value = !*value);
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
