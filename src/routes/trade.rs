use crate::{
    constants::{GRPC_URL, TOKEN_MAP},
    keplr::Keplr,
    liquidity_book::{constants::addrs::LB_PAIR_CONTRACT, contract_interfaces::*},
    prelude::{CHAIN_ID, COMPUTE_QUERY},
    state::*,
    utils::humanize_token_amount,
    LoadingModal,
};
use cosmwasm_std::Uint128;
use leptos::either::Either;
use leptos::either::EitherOf3;
use leptos::{html::Select, prelude::*};
use leptos_router::{hooks::query_signal_with_options, NavigateOptions};
use rsecret::{
    query::compute::ComputeQuerier, secret_client::CreateTxSenderOptions, tx::ComputeServiceClient,
    TxOptions,
};
use secretrs::AccountId;
use send_wrapper::SendWrapper;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::sync::Arc;
use tonic_web_wasm_client::Client as WebWasmClient;
use tracing::{debug, info};
use web_sys::MouseEvent;

// #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
// pub struct BalanceResponse {
//     pub balance: Balance,
// }

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

#[derive(thiserror::Error, serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub enum Snip20Error {
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

    // #[error("Wrong viewing key for this address or viewing key not set")]
    #[error("{0}!")]
    ViewingKey(String),
}

impl Snip20Error {
    pub fn generic(msg: impl Into<String>) -> Self {
        Self::Generic(msg.into())
    }
}

use serde_json::Error as SerdeJsonError;
impl From<SerdeJsonError> for Snip20Error {
    fn from(err: SerdeJsonError) -> Self {
        Snip20Error::Generic(format!("Deserialization error: {}", err))
    }
}

use rsecret::Error as SecretError;
impl From<SecretError> for Snip20Error {
    fn from(err: SecretError) -> Self {
        Snip20Error::SecretClient(err.to_string())
    }
}

pub async fn query_snip20_balance(
    enabled: bool,
    maybe_key: Option<Result<crate::keplr::Key, crate::Error>>,
    maybe_contract_address: Option<String>,
    endpoint: String,
) -> Result<String, Snip20Error> {
    if !enabled {
        return Err(Snip20Error::KeplrDisabled);
    }

    let key = maybe_key
        .and_then(|res| res.ok())
        .ok_or(Snip20Error::KeplrKey)?;

    let contract_address = maybe_contract_address.ok_or(Snip20Error::NoToken)?;

    // TODO: if missing, query token info (and add it to the map?)
    let token = TOKEN_MAP
        .get(&contract_address)
        .ok_or(Snip20Error::UnknownToken)?;

    let vk = Keplr::get_secret_20_viewing_key(CHAIN_ID, &contract_address)
        .await
        .inspect_err(|err| error!("{err:?}"))
        .map_err(|err| Snip20Error::Generic(err.to_string()))?;

    debug!("Found viewing key for {}: {}", token.metadata.symbol, vk);

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
    let query = secret_toolkit_snip20::QueryMsg::Balance { address, key: vk };

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
        SnipQueryResponse::Balance(balance) => Ok(balance.amount.humanize(token.metadata.decimals)),
        SnipQueryResponse::ViewingKeyError(viewing_key_error) => {
            Err(Snip20Error::ViewingKey(viewing_key_error.msg)).inspect_err(|err| error!("{err}"))
        }
    }
}

#[component]
pub fn SnipBalance(token_address: Signal<Option<String>>) -> impl IntoView {
    let endpoint = use_context::<Endpoint>().expect("endpoint context missing!");
    let keplr = use_context::<KeplrSignals>().expect("keplr signals context missing!");
    // let token_map = use_context::<TokenMap>().expect("tokens context missing!");

    // let (token_map, _) = signal(token_map.0);

    // TODO: this should return a Result<Uint128> instead. That way the value can be manipulated
    // elsewhere (like changing to full precision on hover).
    let token_balance = Resource::new(
        move || (keplr.enabled.get(), keplr.key.get(), token_address.get()),
        move |(enabled, maybe_key, maybe_contract_address)| {
            // let map = token_map.get();
            let endpoint = endpoint.get();

            SendWrapper::new({
                query_snip20_balance(enabled, maybe_key, maybe_contract_address, endpoint)
                // async move {
                //     if !enabled {
                //         return Err(Snip20Error::KeplrDisabled);
                //     }
                //
                //     let key = maybe_key
                //         .and_then(|res| res.ok())
                //         .ok_or(Snip20Error::KeplrKey)?;
                //
                //     let contract_address = maybe_contract_address.ok_or(Snip20Error::NoToken)?;
                //
                //     // TODO: if missing, query token info (and add it to the map?)
                //     let token = TOKEN_MAP
                //         .get(&contract_address)
                //         .ok_or(Snip20Error::UnknownToken)?;
                //
                //     let vk = Keplr::get_secret_20_viewing_key(CHAIN_ID, &contract_address)
                //         .await
                //         .inspect_err(|err| error!("{err:?}"))
                //         .map_err(|err| Snip20Error::Generic(err.to_string()))?;
                //
                //     debug!("Found viewing key for {}: {}", token.metadata.symbol, vk);
                //
                //     let compute = ComputeQuerier::new(
                //         WebWasmClient::new(endpoint),
                //         Keplr::get_enigma_utils(CHAIN_ID).into(),
                //     );
                //
                //     // TODO: make rsecret do this part?
                //     let code_hash = compute
                //         .code_hash_by_contract_address(&token.contract_address)
                //         .await?;
                //
                //     debug!(
                //         "contract_address: {}\n\
                //                     code_hash: {}",
                //         &token.contract_address, code_hash
                //     );
                //
                //     let address = key.bech32_address;
                //     let query = secret_toolkit_snip20::QueryMsg::Balance { address, key: vk };
                //
                //     debug!("query: {query:#?}");
                //
                //     // possible responses are:
                //     // {"balance":{"amount":"800000"}}
                //     // {"viewing_key_error":{"msg":"Wrong viewing key for this address or viewing key not set"}}
                //
                //     let response = compute
                //         .query_secret_contract(&token.contract_address, code_hash, query)
                //         .await?;
                //
                //     debug!("response: {response}");
                //
                //     let response = serde_json::from_str::<SnipQueryResponse>(&response)?;
                //
                //     match response {
                //         SnipQueryResponse::Balance(balance) => {
                //             Ok(balance.amount.humanize(token.metadata.decimals))
                //         }
                //         SnipQueryResponse::ViewingKeyError(viewing_key_error) => {
                //             Err(Snip20Error::ViewingKey(viewing_key_error.msg))
                //         }
                //     }
                // }
            })
        },
    );

    view! {
        <div class="snip-balance" on:hover=|_: MouseEvent| ()>
            <Suspense fallback=|| {
                view! { <div class="py-0 px-2 text-ellipsis text-sm">"Loading..."</div> }
            }>
                {move || Suspend::new(async move {
                    match token_balance.await.clone() {
                        Ok(amount) => {
                            // not sure we even need the Either in this case
                            Either::Left(
                                view! {
                                    // TODO: copy balance on click
                                    <div
                                        on:click=|_: MouseEvent| ()
                                        class="py-0 px-2 hover:bg-violet-500/20 text-ellipsis text-sm"
                                    >
                                        {amount}
                                    </div>
                                },
                            )
                        }
                        // These error types are mild enough that it's not worth showing an error
                        Err(
                            error @ (Snip20Error::KeplrDisabled
                            | Snip20Error::KeplrKey
                            | Snip20Error::NoToken),
                        ) => {
                            Either::Right(
                                view! {
                                    <div
                                        title=error.to_string()
                                        class="py-0 px-2 cursor-default text-ellipsis text-sm"
                                    >
                                        "Balance: 👀"
                                    </div>
                                },
                            )
                        }
                        Err(error) => {
                            Either::Right(
                                view! {
                                    <div
                                        title=error.to_string()
                                        class="py-0 px-2 text-violet-400 text-bold text-sm cursor-default hover:bg-violet-500/20 text-ellipsis"
                                    >
                                        "Error 🛈"
                                    </div>
                                },
                            )
                        }
                    }
                })}
            </Suspense>
        </div>
    }
}

#[component]
pub fn Trade() -> impl IntoView {
    info!("rendering <Pool/>");

    on_cleanup(move || {
        info!("cleaning up <Pool/>");
    });

    let endpoint = use_context::<Endpoint>().expect("endpoint context missing!");
    let keplr = use_context::<KeplrSignals>().expect("keplr signals context missing!");
    let token_map = use_context::<TokenMap>().expect("tokens context missing!");

    // prevents scrolling to the top of the page each time a query param changes
    let nav_options = NavigateOptions {
        scroll: false,
        ..Default::default()
    };

    let (token_x, set_token_x) = query_signal_with_options::<String>("from", nav_options.clone());
    let (token_y, set_token_y) = query_signal_with_options::<String>("to", nav_options.clone());

    // let token_map = Arc::new(token_map.0);
    let (token_map, _) = signal(token_map.0);

    // let token_x_balance = Resource::new(
    //     move || (token_x.get(), keplr.enabled.get()),
    //     move |(contract_address, enabled)| {
    //         let map = token_map.get();
    //
    //         SendWrapper::new({
    //             async move {
    //                 if !enabled {
    //                     return "Balance: 👀".to_string();
    //                 }
    //                 let Some(contract_address) = contract_address else {
    //                     return "Select a token".to_string();
    //                 };
    //                 let Some(token) = map.get(&contract_address) else {
    //                     return "Token not in map".to_string();
    //                 };
    //                 match Keplr::get_secret_20_viewing_key(CHAIN_ID, &contract_address).await {
    //                     Ok(vk) => {
    //                         debug!("Found viewing key for {}!\n{vk}", token.metadata.symbol);
    //                         let compute = ComputeQuerier::new(
    //                             wasm_client.get_untracked(),
    //                             Keplr::get_enigma_utils(CHAIN_ID).into(),
    //                         );
    //                         let code_hash = compute
    //                             .code_hash_by_contract_address(&token.contract_address)
    //                             .await
    //                             .expect("failed to query the code hash");
    //                         debug!(
    //                             "contract_address: {}\n\
    //                                 code_hash: {}",
    //                             &token.contract_address, code_hash
    //                         );
    //                         let address =
    //                             keplr.key.get_untracked().unwrap().unwrap().bech32_address;
    //                         let query = secret_toolkit_snip20::QueryMsg::Balance {
    //                             address: address,
    //                             key: vk,
    //                         };
    //                         debug!("query: {query:?}");
    //                         let result = compute
    //                             .query_secret_contract(&token.contract_address, code_hash, query)
    //                             .await
    //                             .unwrap();
    //                         debug!("{result}");
    //
    //                         let result: BalanceResponse = serde_json::from_str(&result).unwrap();
    //                         format!("Balance: {}", result.balance.amount.to_string())
    //                     }
    //                     Err(err) => {
    //                         debug!("{}", err.to_string());
    //                         "viewing key missing".to_string()
    //                     }
    //                 }
    //             }
    //         })
    //     },
    // );

    let token_y_balance = AsyncDerived::new_unsync({
        // let map = token_map.clone();
        move || {
            // let map = map.clone();
            let url = endpoint.get();
            async move {
                if let Some(token) = token_y.get() {
                    let map = token_map.get();
                    let token = map.get(&token).unwrap();
                    match Keplr::get_secret_20_viewing_key(CHAIN_ID, &token.contract_address).await
                    {
                        Ok(vk) => {
                            debug!("Found viewing key for {}!", token.metadata.symbol);
                            let compute = ComputeQuerier::new(
                                WebWasmClient::new(url),
                                Keplr::get_enigma_utils(CHAIN_ID).into(),
                            );
                            let code_hash = compute
                                .code_hash_by_contract_address(&token.contract_address)
                                .await
                                .expect("failed to query the code hash");
                            let address = keplr.key.get().unwrap().unwrap().bech32_address;
                            let query = secret_toolkit_snip20::QueryMsg::Balance {
                                address: address,
                                key: vk,
                            };
                            let result = compute
                                .query_secret_contract(&token.contract_address, code_hash, query)
                                .await
                                .unwrap();
                            let result: secret_toolkit_snip20::Balance =
                                serde_json::from_str(&result).unwrap();
                            let factor = 10u128.pow(token.metadata.decimals as u32);
                            let result = result.amount.u128() as f64 / factor as f64;
                            format!("Balance: {}", result.to_string())
                        }
                        Err(err) => {
                            debug!("{}", err.to_string());
                            "viewing key missing".to_string()
                        }
                    }
                } else {
                    "Select a token".to_string()
                }
            }
        }
    });

    let (amount_x, set_amount_x) = signal(String::default());
    let (amount_y, set_amount_y) = signal(String::default());
    let (swap_for_y, set_swap_for_y) = signal(true);

    let select_x_node_ref = NodeRef::<Select>::new();
    let select_y_node_ref = NodeRef::<Select>::new();

    Effect::new(move || {
        let token_x = token_x.get().unwrap_or_default();
        if let Some(select_x) = select_x_node_ref.get() {
            select_x.set_value(&token_x)
        }
    });

    Effect::new(move || {
        let token_y = token_y.get().unwrap_or_default();
        if let Some(select_y) = select_y_node_ref.get() {
            select_y.set_value(&token_y)
        }
    });

    let swap = Action::new(move |_: &()| {
        let url = endpoint.get();
        SendWrapper::new(async move {
            use cosmwasm_std::Uint128;
            use rsecret::tx::compute::MsgExecuteContractRaw;
            use secretrs::proto::cosmos::tx::v1beta1::BroadcastMode;
            use shade_protocol::swap::core::{TokenAmount, TokenType};

            let Ok(key) = Keplr::get_key(CHAIN_ID).await else {
                return error!("Could not get key from Keplr");
            };

            // NOTE: For any method on Keplr that returns a promise (almost all of them), if it's Ok,
            // that means keplr is enabled. We can use this fact to update any UI that needs to
            // know if Keplr is enabled. Modifying this signal will cause everything subscribed
            // to react. I don't want to trigger that reaction every single time though... which it
            // currently does. This will trigger the AsyncDerived signal to get the key. Maybe
            // that's fine since it's trivial.
            keplr.enabled.set(true);

            let wallet = Keplr::get_offline_signer_only_amino(CHAIN_ID);
            let enigma_utils = Keplr::get_enigma_utils(CHAIN_ID).into();

            let options = CreateTxSenderOptions {
                url: GRPC_URL,
                chain_id: CHAIN_ID,
                wallet: wallet.into(),
                wallet_address: key.bech32_address.into(),
                enigma_utils,
            };

            let wasm_web_client = tonic_web_wasm_client::Client::new(url);
            let compute_service_client = ComputeServiceClient::new(wasm_web_client, options);

            // TODO: decide on using return error vs expect
            let Ok(sender) = AccountId::new("secret", &key.address) else {
                return error!("Error creating sender AccountId");
            };
            // let Ok(contract) = AccountId::from_str(LB_PAIR_CONTRACT.address.as_ref()) else {
            //     return error!("Error creating contract AccountId");
            // };
            let contract = AccountId::from_str("secret1k0jntykt7e4g3y88ltc60czgjuqdy4c9e8fzek")
                .expect("Error creating contract AccountId");
            let msg = secret_toolkit_snip20::HandleMsg::Send {
                recipient: "secret17m7gyp4h9df56a2fryt48zt37ksrsrvvqha8he".to_string(),
                recipient_code_hash: None,
                amount: Uint128::from(1u128),
                msg: None,
                memo: None,
                padding: None,
            };
            // let msg = lb_pair::ExecuteMsg::SwapTokens {
            //     offer: TokenAmount {
            //         token: TokenType::CustomToken {
            //             contract_addr: LB_PAIR_CONTRACT.address.clone(),
            //             token_code_hash: LB_PAIR_CONTRACT.code_hash.clone(),
            //         },
            //         amount: Uint128::from_str("1000000").expect("Uint128 parse from_str error"),
            //     },
            //     expected_return: Some(
            //         Uint128::from_str("995000").expect("Uint128 parse from_str error"),
            //     ),
            //     to: None,
            //     padding: None,
            // };
            let msg = MsgExecuteContractRaw {
                sender,
                contract,
                msg,
                sent_funds: vec![],
            };
            let tx_options = TxOptions {
                gas_limit: 50_000,
                broadcast_mode: BroadcastMode::Sync,
                wait_for_commit: true,
                ..Default::default()
            };

            let result = compute_service_client
                .execute_contract(
                    msg,
                    "af74387e276be8874f07bec3a87023ee49b0e7ebe08178c49d0a49c3c98ed60e",
                    tx_options,
                )
                .await;

            match result {
                Ok(ok) => info!("{:?}", ok),
                Err(error) => error!("{}", error),
            }
        })
    });

    view! {
        <LoadingModal when=swap.pending() message="Preparing Transaction" />
        <div class="p-2">
            <div class="text-3xl font-bold mb-4">"Trade"</div>
            <div class="container max-w-sm space-y-6">
                <div class="space-y-2">
                    <div class="flex justify-between">
                        <div>"From"</div>
                        <SnipBalance token_address=token_x.into() />
                    </div>
                    <div class="flex justify-between space-x-2">
                        <input
                            class="p-1 "
                            type="number"
                            placeholder="0.0"
                            prop:value=move || amount_x.get()
                            on:change=move |ev| {
                                let new_value = event_target_value(&ev);
                                set_amount_x.set(new_value.parse().unwrap_or_default());
                                set_amount_y.set("".to_string());
                                set_swap_for_y.set(true);
                            }
                        />
                        <select
                            node_ref=select_x_node_ref
                            class="p-1 w-28"
                            title="Select Token X"
                            on:input=move |ev| {
                                let token_x = event_target_value(&ev);
                                set_token_x.set(None);
                                set_token_x.set(Some(token_x));
                            }
                            prop:value=move || token_x.get().unwrap_or_default()
                        >
                            <option value="" disabled selected>
                                "Select Token"
                            </option>
                            <option value="secret1k0jntykt7e4g3y88ltc60czgjuqdy4c9e8fzek">
                                sSCRT
                            </option>
                            <option value="secret1k6u0cy4feepm6pehnz804zmwakuwdapm69tuc4">
                                "stkd-SCRT"
                            </option>
                            <option value="secret153wu605vvp934xhd4k9dtd640zsep5jkesstdm">
                                SHD
                            </option>
                            <option value="secret1fl449muk5yq8dlad7a22nje4p5d2pnsgymhjfd">
                                SILK
                            </option>
                            <option value="secret1s09x2xvfd2lp2skgzm29w2xtena7s8fq98v852">
                                AMBER
                            </option>
                        </select>
                    </div>
                </div>
                <div class="space-y-2">
                    <div class="flex justify-between">
                        <div>"To"</div>
                        <div class="py-0 px-2 hover:bg-violet-500/20 text-ellipsis">
                            "Balance: 👀"
                        </div>
                    </div>
                    <div class="flex justify-between space-x-2">
                        <input
                            class="p-1 "
                            type="number"
                            placeholder="0.0"
                            prop:value=move || amount_y.get()
                            on:change=move |ev| {
                                let new_value = event_target_value(&ev);
                                set_amount_y.set(new_value.parse().unwrap_or_default());
                                set_amount_x.set("".to_string());
                                set_swap_for_y.set(false);
                            }
                        />
                        <select
                            node_ref=select_y_node_ref
                            class="p-1 w-28"
                            title="Select Token Y"
                            on:change=move |ev| {
                                let token_y = event_target_value(&ev);
                                set_token_y.set(None);
                                set_token_y.set(Some(token_y));
                            }
                            prop:value=move || token_y.get().unwrap_or_default()
                        >
                            <option value="" disabled selected>
                                "Select Token"
                            </option>
                            <option value="secret1k0jntykt7e4g3y88ltc60czgjuqdy4c9e8fzek">
                                sSCRT
                            </option>
                            <option value="secret1k6u0cy4feepm6pehnz804zmwakuwdapm69tuc4">
                                "stkd-SCRT"
                            </option>
                            <option value="secret153wu605vvp934xhd4k9dtd640zsep5jkesstdm">
                                SHD
                            </option>
                            <option value="secret1fl449muk5yq8dlad7a22nje4p5d2pnsgymhjfd">
                                SILK
                            </option>
                            <option value="secret1s09x2xvfd2lp2skgzm29w2xtena7s8fq98v852">
                                AMBER
                            </option>
                        </select>
                    </div>
                </div>
                <button class="p-1 block">"Estimate Swap"</button>
                <button
                    class="p-1 block"
                    disabled=move || !keplr.enabled.get()
                    on:click=move |_| _ = swap.dispatch(())
                >
                    "Swap!"
                </button>
            </div>
        </div>
    }
}
