use crate::{
    error::Error,
    liquidity_book::{
        constants::addrs::LB_ROUTER,
        contract_interfaces::lb_router::{self, CreateLbPairResponse},
        utils::get_id_from_price,
    },
    ChainId, Endpoint, Keplr, KeplrSignals, CHAIN_ID, GRPC_URL,
};
use cosmwasm_std::Addr;
use leptos::prelude::*;
use rsecret::{
    secret_client::CreateTxSenderOptions,
    tx::{compute::MsgExecuteContractRaw, ComputeServiceClient},
    TxOptions,
};
use secretrs::{compute::MsgExecuteContractResponse, tx::Msg, AccountId};
use send_wrapper::SendWrapper;
use shade_protocol::swap::core::TokenType;
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

    let (token_x, set_token_x) = signal("TOKENX".to_string());
    let (token_y, set_token_y) = signal("TOKENY".to_string());
    let (bin_step, set_bin_step) = signal("100".to_string());
    let (active_price, set_active_price) = signal("1".to_string());

    // TODO:
    let create_pool = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let token_x = token_x.get();
        let token_y = token_y.get();
        let bin_step = bin_step.get();
        let active_price = active_price.get();

        debug!("{}", token_x);
        debug!("{}", token_y);
        debug!("{}", bin_step);
        debug!("{}", active_price);

        // ...
    };

    let create_lb_pair = Action::new_local(move |_: &()| {
        let url = GRPC_URL;
        let chain_id = CHAIN_ID;

        let token_x = token_x.get();
        let token_y = token_y.get();
        let bin_step = bin_step
            .get()
            .parse::<u16>()
            .expect("Invalid bin step format");
        let price = active_price
            .get()
            .parse::<f64>()
            .expect("Invalid price format");
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
                // TODO: get contract hashes for each token and make them into TokenType
                msg: lb_router::ExecuteMsg::CreateLbPair {
                    token_x: TokenType::CustomToken {
                        contract_addr: Addr::unchecked(&token_x),
                        token_code_hash: todo!(),
                    },
                    token_y: TokenType::CustomToken {
                        contract_addr: Addr::unchecked(&token_y),
                        token_code_hash: todo!(),
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

    view! {
        <a
            href="/pool"
            class="block text-neutral-200/50 text-sm font-bold cursor-pointer no-underline"
        >
            "🡨 Back to pools list"
        </a>
        <div class="py-3 text-2xl font-bold text-center sm:text-left">"Create New Pool"</div>
        <form class="container max-w-xs space-y-4 py-1 mx-auto sm:mx-0" on:submit=create_pool>
            <label class="block">
                "Select Token"
                <select
                    class="block p-1 font-bold w-full max-w-xs"
                    name="token_x"
                    title="Select Token"
                    on:input=move |ev| set_token_x.set(event_target_value(&ev))
                >
                    <option value="TOKENX">"TOKEN X"</option>
                    <option value="sSCRT">sSCRT</option>
                    <option value="SHD">SHD</option>
                    <option value="AMBER">AMBER</option>
                    <option value="SILK">SILK</option>
                </select>
            </label>
            <label class="block">
                "Select Quote Asset"
                <select
                    class="block p-1 font-bold w-full max-w-xs"
                    name="token_y"
                    title="Select Quote Asset"
                    on:input=move |ev| set_token_y.set(event_target_value(&ev))
                >
                    <option value="TOKENY">"TOKEN Y"</option>
                    <option value="sSCRT">sSCRT</option>
                    <option value="stkd-SCRT">stkd-SCRT</option>
                    <option value="SILK">SILK</option>
                </select>
            </label>
            <label class="block">
                "Select Bin Step"
                <div class="block box-border pt-1 font-semibold w-full max-w-xs space-x-4">
                    <label class="cursor-pointer">
                        <input
                            class=""
                            type="radio"
                            name="binStep"
                            value="25"
                            on:input=move |ev| set_bin_step.set(event_target_value(&ev))
                        />
                        "0.25%"
                    </label>
                    <label class="cursor-pointer">
                        <input
                            class=""
                            type="radio"
                            name="binStep"
                            value="50"
                            on:input=move |ev| set_bin_step.set(event_target_value(&ev))
                        />
                        "0.5%"
                    </label>
                    <label class="cursor-pointer">
                        <input
                            class=""
                            type="radio"
                            name="binStep"
                            value="100"
                            on:input=move |ev| set_bin_step.set(event_target_value(&ev))
                        />
                        "1%"
                    </label>
                </div>
            </label>
            <label class="block">
                "Enter Active Price"
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
