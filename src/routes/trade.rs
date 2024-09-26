use std::str::FromStr;

use crate::{
    constants::GRPC_URL,
    keplr::{Keplr, KeplrOfflineSigner, KeplrOfflineSignerOnlyAmino},
    liquidity_book::{constants::addrs::LB_PAIR_CONTRACT, contract_interfaces::*},
    prelude::CHAIN_ID,
    state::*,
};
use leptos::html::Select;
use leptos::prelude::*;
use leptos_router::{
    hooks::{query_signal_with_options, use_navigate},
    NavigateOptions,
};
use rsecret::{
    secret_network_client::CreateTxSenderOptions,
    tx::{ComputeServiceClient, TxSender},
    TxOptions,
};
use secretrs::{AccountId, EncryptionUtils};
use send_wrapper::SendWrapper;
use shade_protocol::c_std::Uint128;
use tracing::{debug, info};

#[component]
pub fn Trade() -> impl IntoView {
    info!("rendering <Pool/>");

    on_cleanup(move || {
        info!("cleaning up <Pool/>");
    });

    let keplr = use_context::<KeplrSignals>().expect("keplr signals context missing!");

    // prevents scrolling to the top of the page each time a query param changes
    let nav_options = NavigateOptions {
        scroll: false,
        ..Default::default()
    };

    let (token_x, set_token_x) =
        query_signal_with_options::<String>("from".to_string(), nav_options.clone());
    let (token_y, set_token_y) =
        query_signal_with_options::<String>("to".to_string(), nav_options.clone());

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
        SendWrapper::new(async move {
            let wasm_web_client = tonic_web_wasm_client::Client::new(GRPC_URL.to_string());
            let wallet = Keplr::get_offline_signer(CHAIN_ID);
            // FIXME: panics if keplr is not enabled. instead, this should attempt to enable keplr
            let key = keplr.key.await.expect("Problem getting the Keplr key");
            let encryption_utils = Keplr::get_enigma_utils(CHAIN_ID);
            let options = CreateTxSenderOptions::<KeplrOfflineSigner> {
                url: GRPC_URL,
                chain_id: "secret-4",
                wallet: wallet.into(),
                wallet_address: key.bech32_address.into(),
                // FIXME: make the keplr enigma utils work here
                encryption_utils: EncryptionUtils::new(None, "secret-4").unwrap(),
            };
            let compute_service_client = ComputeServiceClient::new(wasm_web_client, options);
            let sender =
                AccountId::new("secret", &key.address).expect("Error creating sender AccountId");
            let contract =
                AccountId::from_str(LB_PAIR_CONTRACT.address.clone().to_string().as_ref())
                    .expect("Error creating contract AccountId");
            let msg = lb_pair::ExecuteMsg::SwapTokens {
                offer: shade_protocol::swap::core::TokenAmount {
                    token: shade_protocol::swap::core::TokenType::CustomToken {
                        contract_addr: LB_PAIR_CONTRACT.address.clone(),
                        token_code_hash: LB_PAIR_CONTRACT.code_hash.clone(),
                    },
                    amount: Uint128::from_str("1000000").expect("Uint128 parse from_str error"),
                },
                expected_return: Some(
                    Uint128::from_str("995000").expect("Uint128 parse from_str error"),
                ),
                to: None,
                padding: None,
            };
            let msg = secretrs::compute::MsgExecuteContract {
                sender,
                contract,
                msg: serde_json::to_vec(&msg).expect("serde problem"),
                sent_funds: vec![],
            };
            let tx_options = TxOptions {
                gas_limit: 500_000,
                ..Default::default()
            };
            let result = compute_service_client
                .execute_contract(msg, LB_PAIR_CONTRACT.code_hash.clone(), tx_options)
                .await;
        })
    });

    // Effect::new(move || debug!("{:?}", amount_x.get()));
    // Effect::new(move || debug!("{:?}", amount_y.get()));
    // Effect::new(move || debug!("{:?}", swap_for_y.get()));

    view! {
        <div class="p-2">
            <div class="text-3xl font-bold mb-4">"Trade"</div>
            <div class="container max-w-sm space-y-6">
                <div class="space-y-2">
                    <div class="flex justify-between">
                        <div>"From"</div>
                        <div class="py-0 px-2 hover:bg-violet-500/20 text-ellipsis">
                            "Balance: 👀"
                        </div>
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
                                set_token_x.set(Some(token_x));
                            }
                            prop:value=move || token_x.get().unwrap_or_default()
                        >
                            <option value="" disabled selected>"Select Token"</option>
                            <option value="sSCRT">sSCRT</option>
                            <option value="stkd-SCRT">"stkd-SCRT"</option>
                            <option value="SHD">SHD</option>
                            <option value="SILK">SILK</option>
                            <option value="AMBER">AMBER</option>
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
                                set_token_y.set(Some(token_y));
                            }
                            prop:value=move || token_x.get().unwrap_or_default()
                        >
                            <option value="" disabled selected>"Select Token"</option>
                            <option value="sSCRT">sSCRT</option>
                            <option value="stkd-SCRT">"stkd-SCRT"</option>
                            <option value="SHD">SHD</option>
                            <option value="SILK">SILK</option>
                            <option value="AMBER">AMBER</option>
                        </select>
                    </div>
                </div>
                        <button class="p-1 block">"Estimate Swap"</button>
                        <button class="p-1 block" on:click=move |_| _ = swap.dispatch(())>"Swap!"</button>
            </div>
        </div>
    }
}
