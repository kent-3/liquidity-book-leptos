use crate::{
    constants::contracts::*, constants::TOKEN_MAP, error::Error, prelude::SYMBOL_TO_ADDR, ChainId,
    Endpoint, KeplrSignals, CHAIN_ID, NODE,
};
use ammber_sdk::{
    contract_interfaces::lb_router::{self, CreateLbPairResponse},
    utils::get_id_from_price,
};
use cosmwasm_std::Addr;
use keplr::Keplr;
use leptos::prelude::*;
use liquidity_book::core::TokenType;
use lucide_leptos::ArrowLeft;
use rsecret::{
    secret_client::CreateTxSenderOptions,
    tx::{compute::MsgExecuteContractRaw, ComputeServiceClient},
    TxOptions,
};
use secretrs::{compute::MsgExecuteContractResponse, tx::Msg, AccountId};
use std::str::FromStr;
use tracing::{debug, error, info};

#[component]
pub fn PoolCreator() -> impl IntoView {
    info!("rendering <PoolCreator/>");

    on_cleanup(move || {
        info!("cleaning up <PoolCreator/>");
    });

    let endpoint = use_context::<Endpoint>().expect("endpoint context missing!");
    let chain_id = use_context::<ChainId>().expect("chain_id context missing!");
    let keplr = use_context::<KeplrSignals>().expect("keplr signals context missing!");

    let (token_x, set_token_x) = signal("AMBER".to_string());
    let (token_y, set_token_y) = signal("SSCRT".to_string());
    let (bin_step, set_bin_step) = signal(100u16);
    let (active_price, set_active_price) = signal("1.0".to_string());

    let create_lb_pair = Action::new_local(move |_: &()| {
        let url = NODE;
        let chain_id = CHAIN_ID;

        let token_x = token_x.get();
        let token_y = token_y.get();
        let bin_step = bin_step.get();
        // TODO: avoid floating point
        let price = active_price
            .get()
            .parse::<f64>()
            .expect("Invalid price format");

        let token_x = SYMBOL_TO_ADDR
            .get(&token_x)
            .and_then(|address| TOKEN_MAP.get(address))
            .expect("token not found")
            .clone();
        let token_y = SYMBOL_TO_ADDR
            .get(&token_y)
            .and_then(|address| TOKEN_MAP.get(address))
            .expect("token not found")
            .clone();

        let active_id = get_id_from_price(price, bin_step);

        async move {
            if false {
                return Err(Error::generic(
                    "this helps the compiler infer the Error type",
                ));
            }

            let key = Keplr::get_key(&chain_id).await?;
            keplr.enabled.set(true);
            // let wallet = Keplr::get_offline_signer_only_amino(&chain_id);
            let wallet = Keplr::get_offline_signer(&chain_id);
            let enigma_utils = Keplr::get_enigma_utils(&chain_id).into();

            // TODO: I guess I need to make this type use Strings instead of &'static str, because the
            // values are not static in this application (user is able to set them to anything).
            let options = CreateTxSenderOptions {
                url,
                chain_id,
                wallet: wallet.into(),
                wallet_address: key.bech32_address.clone().into(),
                enigma_utils,
            };
            let wasm_web_client = tonic_web_wasm_client::Client::new(url.to_string());
            let compute_service_client = ComputeServiceClient::new(wasm_web_client, options);

            let lb_router_contract = &LB_ROUTER;

            let msg = MsgExecuteContractRaw {
                sender: AccountId::from_str(key.bech32_address.as_ref())?,
                contract: AccountId::from_str(lb_router_contract.address.as_ref())?,
                msg: lb_router::ExecuteMsg::CreateLbPair {
                    token_x: TokenType::CustomToken {
                        contract_addr: Addr::unchecked(token_x.contract_address),
                        token_code_hash: token_x.code_hash,
                    },
                    token_y: TokenType::CustomToken {
                        contract_addr: Addr::unchecked(token_y.contract_address),
                        token_code_hash: token_y.code_hash,
                    },
                    active_id,
                    bin_step,
                },
                sent_funds: vec![],
            };

            debug!("{:#?}", msg);

            let tx_options = TxOptions {
                gas_limit: 1_000_000,
                ..Default::default()
            };

            let tx = compute_service_client
                .execute_contract(msg, lb_router_contract.code_hash.clone(), tx_options)
                .await
                .map_err(Error::from)
                .inspect(|tx_response| info!("{tx_response:?}"))
                .inspect_err(|error| error!("{error}"))?;

            if tx.code != 0 {
                error!("{}", tx.raw_log);
            }

            debug!("hello");
            let data = MsgExecuteContractResponse::from_any(&tx.data[0])
                .inspect_err(|e| error! {"{e}"})?
                .data;
            debug!("hello");
            let create_lb_pair_response = serde_json::from_slice::<CreateLbPairResponse>(&data)?;
            debug!("hello");

            debug!("LbPair: {:?}", create_lb_pair_response.lb_pair);

            Ok(())
        }
    });

    // TODO:
    let create_pair_handler = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let token_x = token_x.get();
        let token_y = token_y.get();
        let bin_step = bin_step.get();
        let active_price = active_price.get();

        let token_x = SYMBOL_TO_ADDR
            .get(&token_x)
            .and_then(|address| TOKEN_MAP.get(address))
            .expect("token not found");
        let token_y = SYMBOL_TO_ADDR
            .get(&token_y)
            .and_then(|address| TOKEN_MAP.get(address))
            .expect("token not found");

        debug!("{}", token_x.contract_address);
        debug!("{}", token_y.contract_address);
        debug!("{}", bin_step);
        debug!("{}", active_price);

        create_lb_pair.dispatch(());
    };

    view! {
        <a
            href="/liquidity-book-leptos/pool"
            class="inline-flex gap-x-2 items-center text-neutral-500 text-sm font-bold cursor-pointer no-underline"
        >
            <ArrowLeft size=14 color="#737373" />
            "Back to pools list"
        </a>
        <div class="py-3 text-2xl font-bold text-center sm:text-left">"Create New Pool"</div>
        <form
            class="container max-w-xs space-y-4 py-1 mx-auto sm:mx-0"
            on:submit=create_pair_handler
        >
            <label class="block">
                <span class="mb-1 block">"Select Token"</span>
                <select
                    class="block p-1 font-bold w-full max-w-xs"
                    name="token_x"
                    title="Select Token"
                    on:input=move |ev| set_token_x.set(event_target_value(&ev))
                >
                    <option value="AMBER">"AMBER"</option>
                    <option value="SHD">"SHD"</option>
                    <option value="STKDSCRT">"stkd-SCRT"</option>
                </select>
            </label>
            <label class="block">
                <span class="mb-1 block">"Select Quote Asset"</span>
                <select
                    class="block p-1 font-bold w-full max-w-xs"
                    name="token_y"
                    title="Select Quote Asset"
                    on:input=move |ev| set_token_y.set(event_target_value(&ev))
                >
                    <option value="SSCRT">"sSCRT"</option>
                    <option value="STKDSCRT">"stkd-SCRT"</option>
                    <option value="SNOBLEUSDC">"sUSDC"</option>
                </select>
            </label>
            <label class="block">
                <span class="mb-1 block">"Select Bin Step"</span>
                <div class="block box-border font-semibold w-full max-w-xs space-x-4">
                    <label class="cursor-pointer">
                        <input
                            class=""
                            type="radio"
                            name="binStep"
                            value=1
                            on:input=move |ev| {
                                set_bin_step.set(event_target_value(&ev).parse().unwrap())
                            }
                        />
                        "0.01%"
                    </label>
                    <label class="cursor-pointer">
                        <input
                            class=""
                            type="radio"
                            name="binStep"
                            value=10
                            on:input=move |ev| {
                                set_bin_step.set(event_target_value(&ev).parse().unwrap())
                            }
                        />
                        "0.10%"
                    </label>
                    <label class="cursor-pointer">
                        <input
                            class=""
                            type="radio"
                            name="binStep"
                            value=20
                            on:input=move |ev| {
                                set_bin_step.set(event_target_value(&ev).parse().unwrap())
                            }
                        />
                        "0.20%"
                    </label>
                </div>
            </label>
            <label class="block">
                <span class="mb-1 block">"Enter Active Price"</span>
                <input
                    name="active_price"
                    class="block p-1 font-bold w-full max-w-xs box-border"
                    type="number"
                    inputmode="decimal"
                    min="0"
                    placeholder="0.0"
                    title="Enter Active Price"
                    on:input=move |ev| set_active_price.set(event_target_value(&ev))
                />
            </label>
            <button class="w-full p-1 !mt-6" type="submit">
                Create Pool
            </button>
        </form>
    }
}
