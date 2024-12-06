use crate::keplr::experimental::*;
use crate::{error::Error, keplr::Keplr};
use leptos::logging::*;
use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, LazyLock};
use tracing::{debug, trace};
use web_sys::{js_sys, wasm_bindgen::JsValue, MouseEvent};

static DEVNET: LazyLock<ChainInfo> = LazyLock::new(|| ChainInfo {
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
    features: vec!["secretwasm".to_string()],
});

static TESTNET: LazyLock<ChainInfo> = LazyLock::new(|| ChainInfo {
    chain_id: "pulsar-3".to_string(),
    chain_name: "Pulsar".to_string(),
    rpc: "https://rpc.pulsar.scrttestnet.com".to_string(),
    rest: "https://api.pulsar.scrttestnet.com".to_string(),
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
    features: vec!["secretwasm".to_string()],
});

#[component]
pub fn SuggestChains() -> impl IntoView {
    let suggest_chain_action: Action<ChainInfo, bool, LocalStorage> =
        Action::new_unsync(move |chain_info: &ChainInfo| {
            let chain_info = chain_info.to_owned();
            async move {
                let keplr_extension = js_sys::Reflect::get(&window(), &JsValue::from_str("keplr"))
                    .expect("unable to check for `keplr` property");
                if keplr_extension.is_undefined() || keplr_extension.is_null() {
                    window()
                        .alert_with_message("keplr not found")
                        .expect("alert failed");
                    false
                } else {
                    debug!("Trying to suggest chain to Keplr...");
                    match Keplr::suggest_chain(chain_info).await {
                        Ok(_) => {
                            debug!("Keplr is enabled");
                            true
                        }
                        Err(e) => {
                            error!("{e}");
                            false
                        }
                    }
                }
            }
        });

    let suggest_testnet = move |_: MouseEvent| {
        let _ = suggest_chain_action.dispatch(TESTNET.clone());
    };
    let suggest_devnet = move |_: MouseEvent| {
        let _ = suggest_chain_action.dispatch(DEVNET.clone());
    };

    view! {
        <div class="suggest-chains space-y-2 w-full">
        <div class="text-center text-sm font-bold">"Add Chains to Keplr"</div>
            <div class="flex gap-3">
            <button class="w-full" on:click=suggest_testnet>"pulsar-3"</button>
            <button class="w-full" on:click=suggest_devnet>"secretdev-1"</button>
        </div>
        </div>
    }
}
