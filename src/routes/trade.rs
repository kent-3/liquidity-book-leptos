use crate::{
    components::Secret20Balance,
    constants::{GRPC_URL, TOKEN_MAP},
    error::Error,
    prelude::{Querier, CHAIN_ID},
    state::*,
    LoadingModal,
};
use ammber_sdk::{
    constants::addrs::{LB_AMBER, LB_PAIR, LB_QUOTER, LB_ROUTER, LB_SSCRT},
    contract_interfaces::{
        lb_router::{Path, Version},
        *,
    },
};
use cosmwasm_std::{to_binary, Addr, Uint128, Uint64};
use keplr::Keplr;
use leptos::{html::Select, logging::*, prelude::*};
use leptos_router::{hooks::query_signal_with_options, NavigateOptions};
use rsecret::{
    query::compute::ComputeQuerier, secret_client::CreateTxSenderOptions, tx::ComputeServiceClient,
    TxOptions,
};
use secretrs::AccountId;
use std::str::FromStr;
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

    let amount_in = RwSignal::new(String::new());

    let get_quote = Action::new_local(move |_: &()| {
        async move {
            use cosmwasm_std::Uint128;
            use shade_protocol::swap::core::TokenType;

            // TODO: token y is the quote asset, right?

            let token_x = TokenType::CustomToken {
                contract_addr: LB_AMBER.address.clone(),
                token_code_hash: LB_AMBER.code_hash.clone(),
            };

            let token_y = TokenType::CustomToken {
                contract_addr: LB_SSCRT.address.clone(),
                token_code_hash: LB_SSCRT.code_hash.clone(),
            };

            let amount_in = amount_in.get();

            // TODO: really want to write this differently, like
            // let quote = ILbQuoter(LB_QUOTER.clone()).find_best_path_from_amount_in(tokens. amount_in)?;

            let quote = lb_quoter::QueryMsg::FindBestPathFromAmountIn {
                route: vec![token_x, token_y],
                amount_in: Uint128::from_str(&amount_in).unwrap(),
            }
            .do_query(&LB_QUOTER)
            .await
            .inspect_err(|error| error!("{:?}", error))
            .inspect(|response| debug!("{:?}", response))
            .and_then(|response| Ok(serde_json::from_str::<lb_quoter::Quote>(&response)?));

            debug!("{:#?}", quote);

            quote
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
        let url = GRPC_URL;
        let chain_id = CHAIN_ID;
        async move {
            use cosmwasm_std::Uint128;
            use rsecret::tx::compute::MsgExecuteContractRaw;
            use secretrs::proto::cosmos::tx::v1beta1::BroadcastMode;
            use shade_protocol::swap::core::{TokenAmount, TokenType};

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
                url: GRPC_URL,
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
                    // FIXME: get from the quote.route or path.token_path
                    "9a00ca4ad505e9be7e6e6dddf8d939b7ec7e9ac8e109c8681f10db9cacb36d42",
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

    view! {
        <LoadingModal when=swap.pending() message="Preparing Transaction... (watch the console)" />
        <div class="p-2">
            <div class="text-3xl font-bold mb-4">"Trade"</div>
            <div class="container max-w-sm space-y-6">
                <div class="space-y-2">
                    <div class="flex justify-between">
                        <div>"From"</div>
                        <Secret20Balance token_address=token_x.into() />
                    </div>
                    <div class="flex justify-between space-x-2">
                        <input
                            class="p-1 "
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
                        <Secret20Balance token_address=token_y.into() />
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
                <button class="p-1 block" on:click=move |_|{_=get_quote.dispatch(())}>
                    "Estimate Swap"
                </button>
                                            // returns the final amount (the output token)
        <p>{move || format!("{:?}", get_quote.value().get().and_then(|result| result.map(|mut quote| quote.amounts.pop()).ok()).unwrap_or_default()) } </p>
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
