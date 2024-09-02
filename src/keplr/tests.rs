use crate::keplr::{suggest_chain_types::*, AccountData, Keplr, Key};
use crate::CHAIN_ID;
use keplr_sys; // normally you wouldn't use keplr_sys directly
use leptos::prelude::*;
use leptos::web_sys::console;
use tracing::{debug, info};
use web_sys::js_sys;

async fn enable_keplr(chain_id: impl ToString) -> bool {
    debug!("Trying to enable Keplr...");
    Keplr::enable(vec![chain_id.to_string()]).await.is_ok()
}

// the "keplrOfflineSigner" object is used in the client constructor
async fn get_account(chain_id: &str) -> AccountData {
    let signer = keplr_sys::get_offline_signer_only_amino(chain_id);
    let accounts = signer.get_accounts().await.unwrap();
    let accounts = js_sys::Array::from(&accounts);
    let account = accounts.get(0);

    let account: AccountData = serde_wasm_bindgen::from_value(account).unwrap();
    log!("{account:#?}");

    account
}

async fn get_key(chain_id: &str) -> Key {
    let result = Keplr::get_key(chain_id).await;

    match result {
        Ok(ref key) => log!("{key:#?}"),
        Err(ref e) => log!("{e}"),
    }

    result.unwrap_or_default()
}

// internal use only
fn get_enigma_utils(chain_id: &str) {
    let js_value = keplr_sys::get_enigma_utils(chain_id);

    console::log_1(&js_value);
}

async fn get_secret_20_viewing_key(chain_id: &str, contract_address: String) -> String {
    let key = Keplr::get_secret_20_viewing_key(chain_id, &contract_address)
        .await
        .unwrap();

    log!("{key}");

    key
}

async fn suggest_token(chain_id: &str, contract_address: &str, viewing_key: Option<&str>) {
    let _ = Keplr::suggest_token(chain_id, contract_address, viewing_key).await;

    // if you want to handle the error case (user closes the pop up):

    // let result = Keplr::suggest_token(chain_id, contract_address, viewing_key)
    //     .await
    //     .map_err(Into::into);
    //
    // match result {
    //     Ok(_) => log!("token added?"),
    //     Err(ref e) => error!("{e}"),
    // }
}

#[component]
pub fn KeplrTests() -> impl IntoView {
    log!("rendering <KeplrTests/>");

    on_cleanup(|| {
        log!("cleaning up <KeplrTests/>");
    });

    let enable_keplr_action: Action<(), bool, SyncStorage> =
        Action::new_unsync_with_value(Some(false), |_: &()| enable_keplr(CHAIN_ID));
    let get_account_action: Action<(), AccountData, SyncStorage> =
        Action::new_unsync(|_: &()| get_account(CHAIN_ID));
    let get_key_action: Action<(), Key, SyncStorage> =
        Action::new_unsync(|_: &()| get_key(CHAIN_ID));
    let get_viewing_key_action: Action<String, String, SyncStorage> =
        Action::new_unsync(|input: &String| {
            let token_address = input.clone();
            get_secret_20_viewing_key(CHAIN_ID, token_address)
        });
    let suggest_token_action: Action<(), (), SyncStorage> = Action::new_unsync(move |_: &()| {
        suggest_token(
            CHAIN_ID,
            "secret1s09x2xvfd2lp2skgzm29w2xtena7s8fq98v852",
            Some("hola"),
        )
    });

    let suggest_chain_action: Action<(), (), SyncStorage> =
        Action::new_unsync(move |_: &()| suggest());

    // on:click Handlers

    let enable_keplr = move |_| _ = enable_keplr_action.dispatch(());
    let get_account = move |_| _ = get_account_action.dispatch(());
    let get_key = move |_| _ = get_key_action.dispatch(());
    // TODO: handle the Error when the user does not have a viewing key (in Keplr)
    let get_viewing_key = move |_| {
        _ = get_viewing_key_action
            .dispatch("secret1vkq022x4q8t8kx9de3r84u669l65xnwf2lg3e6".to_string())
    };
    let suggest_token = move |_| _ = suggest_token_action.dispatch_local(());
    let suggest_chain = move |_| _ = suggest_chain_action.dispatch_local(());

    // Action Value Signals

    let account = get_account_action.value();
    let key = get_key_action.value();
    let viewing_key = get_viewing_key_action.value();

    // Non-Actions

    let get_enigma_utils = move |_| get_enigma_utils(CHAIN_ID);
    let disable_keplr = move |_| {
        keplr_sys::disable(CHAIN_ID);
        enable_keplr_action.value().set(Some(false));
        info!("Keplr Disabled");
    };

    // Log to console any time the value changes
    Effect::new(move |_| {
        if let Some(status) = enable_keplr_action.value().get() {
            debug!("keplr_enabled={}", status);
        }
    });

    let keplr_status = move || match enable_keplr_action.value().get() {
        Some(true) => "enabled",
        Some(false) => "disabled",
        None => "error: signal empty",
    };

    view! {
        <h2>"Keplr Tests"</h2>

        <div class="grid grid-cols-[auto,1fr] gap-x-4 gap-y-2 overflow-auto items-center ">

            <button on:click=enable_keplr>Enable</button>
            <code class="font-mono max-w-max">"enable(CHAIN_ID)"</code>

            <button on:click=suggest_chain>"Suggest Chain"</button>
            <code class="font-mono max-w-max">"experimentalSuggestChain(...)"</code>

            <button on:click=get_account>"Get Account"</button>
            <code class="font-mono max-w-max">"keplrOfflineSigner.getAccounts(CHAIN_ID)"</code>

            <button on:click=get_key>"Get Key"</button>
            <code class="font-mono max-w-max">"getKey(CHAIN_ID)"</code>

            <button on:click=get_enigma_utils>"Get Enigma Utils"</button>
            <code class="font-mono max-w-max">"getEnigmaUtils(CHAIN_ID)"</code>

            <button on:click=suggest_token>"Suggest Token (AMBER)"</button>
            <code class="font-mono max-w-max">"suggestToken(CHAIN_ID, contract_address)"</code>

            <button on:click=get_viewing_key>"Get Viewing Key (USDC)"</button>
            <code class="font-mono max-w-max">
                "getSecret20ViewingKey(CHAIN_ID, contract_address)"
            </code>

            <button on:click=disable_keplr>"Disable"</button>
            <code class="font-mono max-w-max">"disable(CHAIN_ID)"</code>

        </div>

        <pre class="overflow-x-auto">
            {move || account.get().map(|value| format!("{value:#?}"))}
        </pre>
        <pre class="overflow-x-auto">{move || key.get().map(|value| format!("{value:#?}"))}</pre>
        <pre class="overflow-x-auto">
            {move || viewing_key.get().map(|value| format!("Viewing Key: {value:#?}"))}
        </pre>
    }
}

pub async fn suggest() {
    let chain_info = SuggestingChainInfo {
        chain_id: "secretdev-1".to_string(),
        chain_name: "localsecret".to_string(),
        rpc: "http://127.0.0.1:26657".to_string(),
        rest: "http://127.0.0.1:1317".to_string(),
        bip44: Bip44 { coin_type: 529 },
        bech32_config: Bech32Config {
            bech32_prefix_acc_addr: "secret".to_string(),
            bech32_prefix_acc_pub: "secretpub".to_string(),
            bech32_prefix_val_addr: "secretvaloper".to_string(),
            bech32_prefix_val_pub: "secretvaloperpub".to_string(),
            bech32_prefix_cons_addr: "secretvalcons".to_string(),
            bech32_prefix_cons_pub: "secretvalconspub".to_string(),
        },
        currencies: vec![Currency {
            coin_denom: "SCRT".to_string(),
            coin_minimal_denom: "uscrt".to_string(),
            coin_decimals: 6,
            coin_gecko_id: "secret".to_string(),
        }],
        fee_currencies: vec![FeeCurrency {
            coin_denom: "SCRT".to_string(),
            coin_minimal_denom: "uscrt".to_string(),
            coin_decimals: 6,
            coin_gecko_id: "secret".to_string(),
            gas_price_step: GasPriceStep {
                low: 0.1,
                average: 0.25,
                high: 0.5,
            },
        }],
        stake_currency: Currency {
            coin_denom: "SCRT".to_string(),
            coin_minimal_denom: "uscrt".to_string(),
            coin_decimals: 6,
            coin_gecko_id: "secret".to_string(),
        },
    };

    let chain_info_js = serde_wasm_bindgen::to_value(&chain_info).unwrap();
    let _ = keplr_sys::suggest_chain(chain_info_js).await;
}
