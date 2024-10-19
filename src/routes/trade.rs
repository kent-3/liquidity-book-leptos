use crate::{
    components::Secret20Balance,
    constants::{GRPC_URL, TOKEN_MAP},
    keplr::Keplr,
    liquidity_book::{constants::addrs::LB_PAIR_CONTRACT, contract_interfaces::*},
    prelude::{CHAIN_ID, COMPUTE_QUERY},
    state::*,
    LoadingModal,
};
use leptos::{html::Select, prelude::*};
use leptos_router::{hooks::query_signal_with_options, NavigateOptions};
use rsecret::{
    query::compute::ComputeQuerier, secret_client::CreateTxSenderOptions, tx::ComputeServiceClient,
    TxOptions,
};
use secretrs::AccountId;
use send_wrapper::SendWrapper;
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
                <button title="todo!()" disabled class="p-1 block">
                    "Estimate Swap"
                </button>
                <button
                    class="p-1 block"
                    disabled=move || !keplr.enabled.get()
                    on:click=move |_| _ = swap.dispatch(())
                >
                    "Test Transaction!"
                </button>
                <span class="text-xs">"(This will send 1 micro sSCRT to yourself)"</span>
            </div>
        </div>
    }
}
