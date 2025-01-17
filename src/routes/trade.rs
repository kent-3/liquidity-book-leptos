use crate::{
    components::Secret20Balance,
    constants::{contracts::*, CHAIN_ID, NODE, SYMBOL_TO_ADDR, TOKEN_MAP},
    error::Error,
    state::*,
    LoadingModal,
};
use ammber_sdk::contract_interfaces::{
    lb_router::{Path, Version},
    *,
};
use cosmwasm_std::{to_binary, Addr, Uint128, Uint64};
use keplr::Keplr;
use leptos::{html, logging::*, prelude::*};
use leptos_router::{hooks::query_signal_with_options, NavigateOptions};
use liquidity_book::core::{TokenAmount, TokenType};
use rsecret::tx::compute::MsgExecuteContractRaw;
use rsecret::{secret_client::CreateTxSenderOptions, tx::ComputeServiceClient, TxOptions};
use secretrs::proto::cosmos::tx::v1beta1::BroadcastMode;
use secretrs::AccountId;
use std::str::FromStr;
use thaw::*;
use tracing::{debug, info};

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

    let (amount_x, set_amount_x) = signal(String::default());
    let (amount_y, set_amount_y) = signal(String::default());
    let (swap_for_y, set_swap_for_y) = signal(true);

    let select_x_node_ref = NodeRef::<html::Select>::new();
    let select_y_node_ref = NodeRef::<html::Select>::new();

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

    let amount_in = RwSignal::new(String::new());

    let get_quote: Action<(), Result<Quote, Error>> = Action::new(move |_: &()| {
        let token_x = token_x.get();
        let token_y = token_y.get();
        let amount_in = amount_in.get();

        async move {
            // TODO: token y is the quote asset, right?

            let Some(token_x_address) = token_x else {
                return Err(Error::generic("No token X selected!"));
            };
            let Some(token_y_address) = token_y else {
                return Err(Error::generic("No token Y selected!"));
            };

            debug!("YO");

            let token_x_code_hash = TOKEN_MAP
                .get(&token_x_address)
                .map(|t| t.code_hash.clone())
                .ok_or(Error::UnknownToken)
                .inspect_err(|error| error!("{:?}", error))?;
            let token_y_code_hash = TOKEN_MAP
                .get(&token_y_address)
                .map(|t| t.code_hash.clone())
                .ok_or(Error::UnknownToken)
                .inspect_err(|error| error!("{:?}", error))?;

            let token_x = TokenType::CustomToken {
                contract_addr: Addr::unchecked(token_x_address),
                token_code_hash: token_x_code_hash,
            };
            let token_y = TokenType::CustomToken {
                contract_addr: Addr::unchecked(token_y_address),
                token_code_hash: token_y_code_hash,
            };

            let route = vec![token_x, token_y];
            let amount_in = Uint128::from_str(&amount_in).unwrap();

            LB_QUOTER
                .find_best_path_from_amount_in(route, amount_in)
                .await
        }
    });

    let path = move || {
        if let Some(Ok(quote)) = get_quote.value().get() {
            Some(Path {
                pair_bin_steps: quote.bin_steps,
                versions: quote.versions,
                token_path: quote.route,
            })
        } else {
            None
        }
    };

    // TODO: input should be the quote!
    let swap = Action::new_local(move |_: &()| {
        // TODO: Use the dynamic versions instead.
        // let url = endpoint.get();
        // let chain_id = chain_id.get();
        let url = NODE;
        let chain_id = CHAIN_ID;

        async move {
            let amount_in =
                Uint128::from_str(amount_in.get().as_str()).expect("Uint128 parse from_str error");

            let Ok(key) = Keplr::get_key(CHAIN_ID).await else {
                return Err(Error::generic("Could not get key from Keplr"));
            };

            // NOTE: For any method on Keplr that returns a promise (almost all of them), if it's Ok,
            // that means keplr is enabled. We can use this fact to update any UI that needs to
            // know if Keplr is enabled. Modifying this signal will cause everything subscribed
            // to react. I don't want to trigger that reaction every single time though... which it
            // currently does. This will trigger the AsyncDerived signal to get the key. Maybe
            // that's fine since it's trivial.
            keplr.enabled.set(true);

            // let wallet = Keplr::get_offline_signer_only_amino(CHAIN_ID);
            let wallet = Keplr::get_offline_signer(chain_id);
            let enigma_utils = Keplr::get_enigma_utils(chain_id).into();

            let options = CreateTxSenderOptions {
                url: NODE,
                chain_id: CHAIN_ID,
                wallet: wallet.into(),
                wallet_address: key.bech32_address.clone().into(),
                enigma_utils,
            };

            // TODO: this isn't making sense... why am I providing a url both here and in the options?
            let wasm_web_client = tonic_web_wasm_client::Client::new(url.to_string());
            let compute_service_client = ComputeServiceClient::new(wasm_web_client, options);

            let sender = AccountId::new("secret", &key.address)?;
            let contract = AccountId::from_str(path().unwrap().token_path[0].address().as_str())?;

            let swap_msg = lb_router::ExecuteMsg::SwapExactTokensForTokens {
                amount_in,
                amount_out_min: Uint128::from_str("1").expect("Uint128 parse from_str error"),
                path: path().unwrap(),
                to: key.bech32_address.clone(),
                deadline: Uint64::from(2736132325u64),
            };

            debug!("{swap_msg:#?}");

            let send_msg = secret_toolkit_snip20::HandleMsg::Send {
                recipient: LB_ROUTER.address.to_string(),
                recipient_code_hash: Some(LB_ROUTER.code_hash.clone()),
                amount: amount_in,
                msg: Some(to_binary(&swap_msg).unwrap()),
                memo: None,
                padding: None,
            };

            let msg = MsgExecuteContractRaw {
                sender,
                contract,
                msg: send_msg,
                sent_funds: vec![],
            };

            let tx_options = TxOptions {
                gas_limit: 500_000,
                broadcast_mode: BroadcastMode::Sync,
                wait_for_commit: true,
                ..Default::default()
            };

            let tx = compute_service_client
                .execute_contract(
                    msg,
                    path().unwrap().token_path[0].address().as_str(),
                    tx_options,
                )
                .await
                .inspect(|tx_response| info!("{tx_response:?}"))
                .inspect_err(|error| error!("{error}"))?;

            if tx.code != 0 {
                error!("{}", tx.raw_log);
            }

            // match tx {
            //     Ok(ok) => info!("{:?}", ok),
            //     Err(error) => error!("{}", error),
            // }

            Ok(())
        }
    });

    use liquidity_book::interfaces::lb_quoter::Quote;

    let current_quote = move || {
        get_quote
            .value()
            .get()
            .and_then(Result::ok)
            .map(|quote| serde_json::to_string_pretty(&quote).unwrap())
        // .map(|quote| format!("{:#?}", quote))
    };

    // returns the final amount (the output token)
    let amount_out = move || {
        get_quote
            .value()
            .get()
            .and_then(Result::ok)
            .map(|mut quote| quote.amounts.pop())
            .flatten()
            .map(|amount| amount.to_string())
    };

    // pub struct Quote {
    //     pub route: Vec<TokenType>,
    //     pub pairs: Vec<ContractInfo>,
    //     pub bin_steps: Vec<u16>,
    //     pub versions: Vec<Version>,
    //     pub amounts: Vec<Uint128>,
    //     pub virtual_amounts_without_slippage: Vec<Uint128>,
    //     pub fees: Vec<Uint128>,
    // }

    view! {
        <LoadingModal when=swap.pending() message="Preparing Transaction... (watch the console)" />
        <div class="grid gap-4 sm:grid-cols-[minmax(0px,7fr)_minmax(0px,5fr)] grid-cols-1 grid-rows-2 sm:grid-rows-1">
            <div class="container block align-middle sm:row-auto row-start-2 outline outline-2 outline-neutral-700 rounded">
                // <p class="text-center italic text-neutral-500">"what should go here?"</p>
                <pre class="px-2 text-xs whitespace-pre-wrap text-neutral-300">{current_quote}</pre>
            </div>
            <div class="container space-y-6 sm:row-auto row-start-1">
                <div class="space-y-2">
                    <div class="flex justify-between">
                        <label class="block mb-1 text-base" for="from-token">
                            "From"
                        </label>
                        // <div>"From"</div>
                        <Secret20Balance token_address=token_x.into() />
                    </div>
                    <div class="flex justify-between space-x-2">
                        <input
                            id="from-token"
                            class="p-1 w-full"
                            type="number"
                            placeholder="0.0"
                            bind:value=amount_in
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
                            <option value=SYMBOL_TO_ADDR.get("SSCRT")>sSCRT</option>
                            <option value=SYMBOL_TO_ADDR.get("STKDSCRT")>"stkd-SCRT"</option>
                            <option value=SYMBOL_TO_ADDR.get("AMBER")>AMBER</option>
                            <option value=SYMBOL_TO_ADDR.get("SHD")>SHD</option>
                        </select>
                    </div>
                </div>
                <div class="space-y-2">
                    <div class="flex justify-between">
                        <div>"To"</div>
                        <Secret20Balance token_address=token_y.into() />
                    </div>
                    <div class="flex justify-between space-x-2">
                        <input
                            class="p-1 w-full"
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
                            <option value=SYMBOL_TO_ADDR.get("SSCRT")>sSCRT</option>
                            <option value=SYMBOL_TO_ADDR.get("STKDSCRT")>"stkd-SCRT"</option>
                            <option value=SYMBOL_TO_ADDR.get("AMBER")>AMBER</option>
                            <option value=SYMBOL_TO_ADDR.get("SHD")>SHD</option>
                        </select>
                    </div>
                </div>
                <button
                    class="p-1 block"
                    disabled=move || amount_in.get().is_empty()
                    on:click=move |_| _ = get_quote.dispatch(())
                >
                    "Estimate Swap"
                </button>
                <Show when=move || amount_out().is_some() fallback=|| ()>
                    <p>"Amount out: " {amount_out}</p>
                </Show>
                <button
                    class="p-1 block"
                    disabled=move || !keplr.enabled.get()
                    on:click=move |_| _ = swap.dispatch(())
                >
                    "Swap"
                </button>
            // <span class="text-xs">"(This will send 1 micro sSCRT to yourself)"</span>
            </div>
        </div>
    }
}
