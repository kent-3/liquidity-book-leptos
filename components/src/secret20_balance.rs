#![allow(unused)]

use ammber_core::{
    constants::{self, CHAIN_ID, NODE, TOKEN_MAP},
    state::{ChainId, Endpoint, KeplrSignals, TokenMap},
    utils::alert,
};
use cosmwasm_std::Uint128;
use keplr::Keplr;
use leptos::{either::EitherOf4, logging::*, prelude::*};
use lucide_leptos::{Eye, EyeClosed};
use rsecret::query::compute::ComputeQuerier;
use send_wrapper::SendWrapper;
use serde::{Deserialize, Serialize};
use tonic_web_wasm_client::Client as WebWasmClient;
use tracing::{debug, trace};
use web_sys::MouseEvent;

// TODO: add a way to "suggest_token" if "there is no matched secret20"

#[component]
pub fn Secret20Balance(#[prop(into)] token_address: Signal<Option<String>>) -> impl IntoView {
    let endpoint = use_context::<Endpoint>().expect("endpoint context missing!");
    let chain_id = use_context::<ChainId>().expect("chain_id context missing!");
    let keplr = use_context::<KeplrSignals>().expect("keplr context missing!");

    let token_symbol = move || {
        token_address
            .get()
            .and_then(|address| TOKEN_MAP.get(&address).cloned())
            .map(|t| t.symbol)
    };

    let suggest_token = Action::new_local(move |contract_address: &String| {
        let chain_id = chain_id.chain_id.get();
        let contract_address = contract_address.clone();
        // This still doesn't work right. It only works if the user already has the token in their wallet
        // let viewing_key = Some("hola");

        async move {
            Keplr::suggest_token(&chain_id, &contract_address, None)
                .await
                .is_ok()
        }
    });

    let token_balance = Resource::new(
        move || {
            (
                keplr.enabled.get(),
                keplr.key.get(),
                token_address.get(),
                suggest_token.value().get(), // this will reload the resource after the token is added
            )
        },
        move |(enabled, maybe_key, maybe_contract_address, token_suggested)| {
            let endpoint = endpoint.get();
            let chain_id = chain_id.get();
            SendWrapper::new({
                async move {
                    if !enabled {
                        return Err(Error::KeplrDisabled);
                    }
                    let key = maybe_key.and_then(Result::ok).ok_or(Error::KeplrKey)?;
                    let contract_address = maybe_contract_address.ok_or(Error::NoToken)?;
                    // TODO: if missing, query token info (and add it to the map?)
                    let token = TOKEN_MAP
                        .get(&contract_address)
                        .ok_or(Error::UnknownToken)?;
                    let vk = Keplr::get_secret_20_viewing_key(&chain_id, &contract_address)
                        .await
                        .inspect_err(|err| error!("{err:?}"))
                        .map_err(|err| Error::Generic(err.to_string()))?;
                    debug!("Found viewing key for {}: {}", token.symbol, vk);
                    query_snip20_balance(key, token.clone(), vk, endpoint.to_string()).await
                }
            })
        },
    );

    // The middle error types are mild enough that it's not worth showing an error
    // TODO: copy balance on click
    view! {
        <Suspense fallback=|| {
            view! { <div class="py-0 px-2 text-ellipsis text-sm">"Loading..."</div> }
        }>
            {move || Suspend::new(async move {
                match token_balance.await.clone() {
                    Ok(amount) => {
                        EitherOf4::A(
                            view! {
                                <div class="inline-flex gap-1 items-center text-sm rounded-md cursor-default text-muted-foreground">
                                    <button
                                        tabindex=0
                                        class="p-0 border-0 hover:text-primary cursor-pointer rounded-sm"
                                        on:click=|_: MouseEvent| alert("TODO: hide balance")
                                    >
                                        <Eye size=16 />
                                    </button>
                                    <button
                                        tabindex="-1"
                                        on:click=|_: MouseEvent| alert("TODO: use max balance")
                                        class="p-0 border-0 inline-flex items-center gap-2"
                                    >
                                        "Balance: "
                                        <span
                                            tabindex=0
                                            class="text-white font-medium max-w-[12rem] truncate break-all cursor-pointer
                                            hover:text-primary focus-visible:text-primary focus-visible:outline-none"
                                        >
                                            {amount}
                                        </span>
                                    </button>
                                </div>
                            },
                        )
                    }
                    Err(error @ (Error::KeplrDisabled | Error::KeplrKey | Error::NoToken)) => {
                        EitherOf4::B(view! {})
                    }
                    Err(error) => {
                        if error.to_string() == "There is no matched secret20!" {
                            EitherOf4::C(
                                view! {
                                    <div class="group relative leading-none">
                                        <button
                                            on:click=move |_| {
                                                _ = suggest_token
                                                    .dispatch_local(token_address.get().unwrap_or_default());
                                            }
                                            class="p-0 border-none inline-flex items-center gap-1 cursor-default text-ellipsis text-sm text-muted-foreground"
                                        >
                                            <EyeClosed size=16 />
                                            "View Balance"
                                        </button>
                                        <div class="absolute bottom-full left-1/2 -translate-x-1/2 mb-0 px-2 py-1
                                        invisible group-hover:visible opacity-0 group-hover:opacity-100 transition-opacity duration-100 ease-in
                                        border border-solid border-border
                                        bg-popover text-popover-foreground text-xs font-semibold rounded-md whitespace-nowrap">
                                            "Add " {token_symbol()} " to wallet"
                                        </div>
                                    </div>
                                },
                            )
                        } else {
                            EitherOf4::D(
                                view! {
                                    // class="p-0 inline-flex items-center gap-1 cursor-default text-ellipsis text-sm text-muted-foreground"
                                    // class="p-0 inline-flex items-center gap-1 cursor-default text-ellipsis text-sm text-muted-foreground"
                                    // class="p-0 inline-flex items-center gap-1 cursor-default text-ellipsis text-sm text-muted-foreground"
                                    // class="p-0 inline-flex items-center gap-1 cursor-default text-ellipsis text-sm text-muted-foreground"
                                    // class="p-0 inline-flex items-center gap-1 cursor-default text-ellipsis text-sm text-muted-foreground"
                                    // class="p-0 inline-flex items-center gap-1 cursor-default text-ellipsis text-sm text-muted-foreground"
                                    // class="p-0 inline-flex items-center gap-1 cursor-default text-ellipsis text-sm text-muted-foreground"
                                    // class="p-0 inline-flex items-center gap-1 cursor-default text-ellipsis text-sm text-muted-foreground"
                                    // class="p-0 inline-flex items-center gap-1 cursor-default text-ellipsis text-sm text-muted-foreground"
                                    // class="p-0 inline-flex items-center gap-1 cursor-default text-ellipsis text-sm text-muted-foreground"
                                    // class="p-0 inline-flex items-center gap-1 cursor-default text-ellipsis text-sm text-muted-foreground"
                                    // class="p-0 inline-flex items-center gap-1 cursor-default text-ellipsis text-sm text-muted-foreground"
                                    // class="p-0 inline-flex items-center gap-1 cursor-default text-ellipsis text-sm text-muted-foreground"
                                    // class="p-0 inline-flex items-center gap-1 cursor-default text-ellipsis text-sm text-muted-foreground"
                                    // class="p-0 inline-flex items-center gap-1 cursor-default text-ellipsis text-sm text-muted-foreground"
                                    // class="p-0 inline-flex items-center gap-1 cursor-default text-ellipsis text-sm text-muted-foreground"
                                    // class="p-0 inline-flex items-center gap-1 cursor-default text-ellipsis text-sm text-muted-foreground"
                                    // class="p-0 inline-flex items-center gap-1 cursor-default text-ellipsis text-sm text-muted-foreground"
                                    // class="p-0 inline-flex items-center gap-1 cursor-default text-ellipsis text-sm text-muted-foreground"
                                    // class="p-0 inline-flex items-center gap-1 cursor-default text-ellipsis text-sm text-muted-foreground"
                                    // class="p-0 inline-flex items-center gap-1 cursor-default text-ellipsis text-sm text-muted-foreground"
                                    // class="p-0 inline-flex items-center gap-1 cursor-default text-ellipsis text-sm text-muted-foreground"
                                    // class="p-0 inline-flex items-center gap-1 cursor-default text-ellipsis text-sm text-muted-foreground"
                                    // class="p-0 inline-flex items-center gap-1 cursor-default text-ellipsis text-sm text-muted-foreground"
                                    // class="p-0 inline-flex items-center gap-1 cursor-default text-ellipsis text-sm text-muted-foreground"
                                    // class="p-0 inline-flex items-center gap-1 cursor-default text-ellipsis text-sm text-muted-foreground"
                                    // class="p-0 inline-flex items-center gap-1 cursor-default text-ellipsis text-sm text-muted-foreground"
                                    // class="p-0 inline-flex items-center gap-1 cursor-default text-ellipsis text-sm text-muted-foreground"
                                    <div
                                        title=error.to_string()
                                        class="group relative leading-none text-primary font-semibold text-sm cursor-default text-ellipsis"
                                    >
                                        "Error 🛈"
                                        <div class="absolute bottom-full left-1/2 -translate-x-1/2 mb-2 invisible group-hover:visible
                                        bg-popover text-popover-foreground text-xs font-medium px-2 py-1 rounded shadow-sm whitespace-nowrap">
                                            {error.to_string()}
                                        </div>
                                    </div>
                                },
                            )
                        }
                    }
                }
            })}
        </Suspense>
    }
}

pub async fn query_snip20_balance(
    key: keplr::Key,
    token: ammber_core::constants::Token,
    viewing_key: String,
    endpoint: String,
) -> Result<String, Error> {
    let compute = ComputeQuerier::new(
        WebWasmClient::new(endpoint),
        Keplr::get_enigma_utils(CHAIN_ID).into(),
    );

    // TODO: make rsecret do this part?
    let code_hash = compute
        .code_hash_by_contract_address(&token.contract_address)
        .await?;

    debug!(
        "contract_address: {}\n\
                                    code_hash: {}",
        &token.contract_address, code_hash
    );

    let address = key.bech32_address;
    let query = secret_toolkit_snip20::QueryMsg::Balance {
        address,
        key: viewing_key,
    };

    debug!("query: {query:#?}");

    // possible responses are:
    // {"balance":{"amount":"800000"}}
    // {"viewing_key_error":{"msg":"Wrong viewing key for this address or viewing key not set"}}

    let response = compute
        .query_secret_contract(&token.contract_address, code_hash, query)
        .await?;

    debug!("response: {response}");

    let response = serde_json::from_str::<SnipQueryResponse>(&response)?;

    match response {
        SnipQueryResponse::Balance(balance) => Ok(balance.amount.humanize(token.decimals)),
        SnipQueryResponse::ViewingKeyError(viewing_key_error) => {
            Err(Error::ViewingKey(viewing_key_error.msg)).inspect_err(|err| error!("{err}"))
        }
    }
}

use rsecret::Error as SecretError;
use serde_json::Error as SerdeJsonError;

#[derive(thiserror::Error, serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub enum Error {
    #[error("{0}!")]
    Generic(String),

    #[error("Keplr is not enabled!")]
    KeplrDisabled,

    #[error("Keplr key not found!")]
    KeplrKey,

    #[error("No token address provided!")]
    NoToken,

    #[error("Token not in map!")]
    UnknownToken,

    #[error("Secret Client error: {0}!")]
    SecretClient(String),

    #[error("{0}!")]
    ViewingKey(String),
}

impl Error {
    pub fn generic(msg: impl Into<String>) -> Self {
        Self::Generic(msg.into())
    }
}

impl From<SerdeJsonError> for Error {
    fn from(err: SerdeJsonError) -> Self {
        Error::Generic(format!("Deserialization error: {}", err))
    }
}

impl From<SecretError> for Error {
    fn from(err: SecretError) -> Self {
        Error::SecretClient(err.to_string())
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SnipQueryResponse {
    Balance(Balance),
    ViewingKeyError(ViewingKeyError),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Balance {
    pub amount: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct ViewingKeyError {
    pub msg: String,
}

pub trait BalanceFormatter {
    fn humanize(&self, decimals: impl Into<u32>) -> String;
    fn humanize_with_precision(
        &self,
        decimals: impl Into<u32>,
        precision: impl Into<i32>,
    ) -> String;
}

impl BalanceFormatter for cosmwasm_std::Uint128 {
    fn humanize(&self, decimals: impl Into<u32>) -> String {
        let value = self.u128();
        let decimals = decimals.into();
        let factor = 10u128.pow(decimals);

        let integer_part = value / factor;
        let fractional_part = value % factor;

        format!(
            "{}.{:0width$}",
            integer_part,
            fractional_part,
            width = decimals as usize
        )
    }

    fn humanize_with_precision(
        &self,
        decimals: impl Into<u32>,
        precision: impl Into<i32>,
    ) -> String {
        let value = self.u128();
        let decimals = decimals.into();
        let factor = 10u128.pow(decimals);

        let integer_part = value / factor;
        let fractional_part = value % factor;

        // If precision is less than decimals, we need to round
        let precision = precision.into() as u32;
        if precision < decimals {
            let rounding_factor = 10u128.pow(decimals - precision);
            let fractional_rounded = (fractional_part + rounding_factor / 2) / rounding_factor;

            // If rounding overflows (i.e., turns 999 -> 1000), we need to adjust the integer part
            if fractional_rounded >= 10u128.pow(precision) {
                format!(
                    "{}.{:0width$}",
                    integer_part + 1,
                    0u128,
                    width = precision as usize
                )
            } else {
                format!(
                    "{}.{:0width$}",
                    integer_part,
                    fractional_rounded,
                    width = precision as usize
                )
            }
        } else {
            // No rounding needed, just display with full decimals
            format!(
                "{}.{:0width$}",
                integer_part,
                fractional_part,
                width = decimals as usize
            )
        }
    }
}
