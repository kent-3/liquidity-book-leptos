use crate::{
    constants::Querier,
    error::Error,
    keplr::Keplr,
    liquidity_book::{
        constants::{
            addrs::{LB_CONTRACTS, LB_FACTORY, LB_ROUTER},
            liquidity_config::{
                LiquidityConfiguration, LiquidityShape, BID_ASK, CURVE, SPOT_UNIFORM, WIDE,
            },
        },
        contract_interfaces::{
            lb_factory::{self, LbPairInformation},
            lb_router::{self, AddLiquidityResponse, LiquidityParameters},
        },
        utils::{get_id_from_price, get_price_from_id},
    },
    prelude::{CHAIN_ID, GRPC_URL},
    state::*,
    utils::{alert, latest_block},
};
use cosmwasm_std::{Addr, ContractInfo, Uint128, Uint64};
use leptos::{logging::*, prelude::*};
use leptos_router::{
    hooks::{query_signal_with_options, use_params, use_params_map, use_query_map},
    NavigateOptions,
};
use rsecret::{
    query::tendermint::TendermintQuerier,
    secret_client::CreateTxSenderOptions,
    tx::{compute::MsgExecuteContractRaw, ComputeServiceClient},
    TxOptions,
};
use secretrs::{compute::MsgExecuteContractResponse, tx::Msg, AccountId};
use send_wrapper::SendWrapper;
use std::str::FromStr;
use tonic_web_wasm_client::Client;
use tracing::{debug, info, trace};
use web_sys::MouseEvent;

#[component]
pub fn AddLiquidity() -> impl IntoView {
    info!("rendering <AddLiquidity/>");

    let endpoint = use_context::<Endpoint>().expect("endpoint context missing!");
    let chain_id = use_context::<ChainId>().expect("chain_id context missing!");
    let keplr = use_context::<KeplrSignals>().expect("keplr signals context missing!");
    let token_map = use_context::<TokenMap>().expect("tokens context missing!");

    let params = use_params_map();
    let token_a = move || params.read().get("token_a").unwrap_or_default();
    let token_b = move || params.read().get("token_b").unwrap_or_default();
    let basis_points = move || params.read().get("bps").unwrap_or_default();

    // prevents scrolling to the top of the page each time a query param changes
    let nav_options = NavigateOptions {
        scroll: false,
        ..Default::default()
    };
    let (active_id, _) = query_signal_with_options::<String>("active_id", nav_options.clone());
    let (price_by, set_price_by) =
        query_signal_with_options::<String>("price_by", nav_options.clone());

    // TODO: Figure out how to obtain the token code hashes efficiently.
    let lb_pair_information: Resource<LbPairInformation> = Resource::new(
        move || (token_a(), token_b(), basis_points()),
        move |(token_a, token_b, basis_points)| {
            SendWrapper::new(async move {
                // Assume token_x has the code_hash from the current deployment.
                let token_x = ContractInfo {
                    address: Addr::unchecked(token_a),
                    code_hash: LB_CONTRACTS.snip25.code_hash.clone(),
                };
                // Assume token_y is sSCRT
                let token_y = ContractInfo {
                    address: Addr::unchecked(token_b),
                    code_hash: LB_CONTRACTS.snip20.code_hash.clone(),
                };
                let bin_step = basis_points.parse::<u16>().unwrap();

                lb_factory::QueryMsg::GetLbPairInformation {
                    token_x: token_x.into(),
                    token_y: token_y.into(),
                    bin_step,
                }
                .do_query(&LB_FACTORY)
                .await
                .inspect(|response| trace!("{:?}", response))
                .and_then(|response| {
                    Ok(serde_json::from_str::<lb_factory::LbPairInformationResponse>(&response)?)
                })
                .map(|x| x.lb_pair_information)
                .unwrap()
            })
        },
    );

    let query = use_query_map();
    let price_by = move || price_by.get().unwrap_or("radius".to_string());

    let (token_x_amount, set_token_x_amount) = signal("0".to_string());
    let (token_y_amount, set_token_y_amount) = signal("0".to_string());
    let (liquidity_shape, set_liquidity_shape) = signal("uniform".to_string());

    // This is so I can change the default liquidity shape above and still load the correct preset
    let liquidity_configuration_preset = match liquidity_shape.get_untracked().as_ref() {
        "uniform" => SPOT_UNIFORM.clone(),
        "curve" => CURVE.clone(),
        "bid-ask" => BID_ASK.clone(),
        "wide" => WIDE.clone(),
        _ => panic!("invalid liquidity shape"),
    };

    let (liquidity_configuration, set_liquidity_configuration) =
        signal(liquidity_configuration_preset);

    let (target_price, set_target_price) = signal("Loading...".to_string());
    let (radius, set_radius) = signal("5".to_string());

    Effect::new(move || {
        if let Some(id) = active_id.get() {
            let price: f64 = get_price_from_id(id, basis_points());
            set_target_price.set(price.to_string());
        }
    });

    // TODO: account for decimals
    fn amount_min(input: &str) -> u32 {
        let number: f64 = input.parse().expect("Error parsing float");
        let adjusted_value = number * 0.95 * number / 1_000_000.0;
        adjusted_value.round() as u32
    }

    // Might be useful to have this re-run regularly at the top-level and provide a context
    // let latest_block = Resource::new(
    //     move || endpoint.get(),
    //     move |endpoint| {
    //         SendWrapper::new(async move {
    //             let tendermint = TendermintQuerier::new(Client::new(endpoint));
    //             let latest_block = tendermint.get_latest_block().await;
    //
    //             latest_block
    //                 .and_then(|block| Ok(block.header.height))
    //                 .inspect(|height| debug!("{:#?}", height))
    //                 .map_err(Into::<crate::Error>::into)
    //         })
    //     },
    // );

    let liquidity_parameters = move || {
        // By using the information returned by query, we can be sure it is correct
        let lb_pair_information = lb_pair_information
            .get()
            .expect("Unverified LB Pair information");
        // FIXME: The tokens are not always in the expected order!
        let token_x = lb_pair_information.lb_pair.token_x;
        let token_y = lb_pair_information.lb_pair.token_y;
        let bin_step = lb_pair_information.bin_step;

        let amount_x = token_x_amount.get();
        let amount_y = token_y_amount.get();
        let shape = liquidity_shape.get();
        let target_price = target_price.get();
        let radius = radius.get();

        let amount_x_min = amount_min(&amount_x);
        let amount_y_min = amount_min(&amount_y);

        let target_price = target_price.parse::<f64>().expect("Invalid price format");
        let target_bin = get_id_from_price(target_price, bin_step);

        // TODO: figure out how to transform inputs into a LiquidityConfig dynamically
        fn configure_liquidity_by_range(
            min_price: f64,
            max_price: f64,
            bin_step: u16,
            shape: LiquidityShape,
        ) -> LiquidityConfiguration {
            let start_bin = get_id_from_price(min_price, bin_step);
            todo!()
        }
        fn configure_liquidity_by_radius(
            target_bin: u32,
            radius: u32,
            shape: LiquidityShape,
        ) -> LiquidityConfiguration {
            todo!()
        }

        let liquidity_configuration = liquidity_configuration.get();
        let delta_ids = liquidity_configuration.delta_ids();
        let distribution_x = liquidity_configuration.distribution_x(18);
        let distribution_y = liquidity_configuration.distribution_y(18);
        let deadline = Uint64::MIN;

        let liquidity_parameters = LiquidityParameters {
            token_x,
            token_y,
            bin_step,
            amount_x: Uint128::from_str(&amount_x).unwrap(),
            amount_y: Uint128::from_str(&amount_y).unwrap(),
            amount_x_min: Uint128::from(amount_x_min),
            amount_y_min: Uint128::from(amount_y_min),
            active_id_desired: get_id_from_price(target_price, bin_step),
            // TODO: write a function to convert price slippage into id slippage
            id_slippage: 10,
            delta_ids,
            distribution_x,
            distribution_y,
            to: "todo".to_string(),
            refund_to: "todo".to_string(),
            deadline,
        };

        debug!("{:#?}", liquidity_parameters);

        liquidity_parameters
    };

    let add_liquidity_action = Action::new(move |liquidity_parameters: &LiquidityParameters| {
        // TODO: Use the dynamic versions instead.
        // let url = endpoint.get();
        // let chain_id = chain_id.get();
        let url = GRPC_URL;
        let chain_id = CHAIN_ID;
        let mut liquidity_parameters = liquidity_parameters.clone();
        SendWrapper::new(async move {
            if liquidity_parameters.amount_x.is_zero() && liquidity_parameters.amount_y.is_zero() {
                alert("Amounts must not be 0!");
                return Err(Error::generic("Amounts must not be 0!"));
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

            // Recheck the latest block height to update the deadline.
            let tendermint = TendermintQuerier::new(Client::new(url.to_string()));
            let latest_block_time = tendermint
                .get_latest_block()
                .await
                .map(|block| block.header.time.unix_timestamp() as u64)
                .map_err(Error::from)?;

            liquidity_parameters.deadline = (latest_block_time + 100).into();
            liquidity_parameters.to = key.bech32_address.clone().into();
            liquidity_parameters.refund_to = key.bech32_address.clone().into();

            let lb_router_contract = &LB_ROUTER;

            let msg = MsgExecuteContractRaw {
                // sender: AccountId::new("secret", &key.address)?,
                sender: AccountId::from_str(key.bech32_address.as_ref())?,
                contract: AccountId::from_str(lb_router_contract.address.as_ref())?,
                msg: lb_router::ExecuteMsg::AddLiquidity {
                    liquidity_parameters,
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
            let add_liquidity_response = serde_json::from_slice::<AddLiquidityResponse>(&data)?;
            debug!("hello");

            debug!("X: {}", add_liquidity_response.amount_x_added);
            debug!("Y: {}", add_liquidity_response.amount_y_added);

            Ok(())
        })
    });

    let add_liquidity = move |_: MouseEvent| {
        let _ = add_liquidity_action.dispatch(liquidity_parameters());
    };

    view! {
        <div class="container max-w-xs py-2 space-y-2">
            <div class="text-xl font-semibold">Deposit Liquidity</div>
            <div class="flex items-center gap-2">
                <div class="basis-1/3 text-md text-ellipsis overflow-hidden">{token_a}</div>
                <input
                    class="p-1 basis-2/3"
                    type="number"
                    placeholder="Enter Amount"
                    on:change=move |ev| set_token_x_amount.set(event_target_value(&ev))
                />
            </div>
            <div class="flex items-center gap-2">
                <div class="basis-1/3 text-md text-ellipsis overflow-hidden">{token_b}</div>
                <input
                    class="p-1 basis-2/3"
                    type="number"
                    placeholder="Enter Amount"
                    on:change=move |ev| set_token_y_amount.set(event_target_value(&ev))
                />
            </div>

            <div class="text-xl font-semibold !mt-6">Choose Liquidity Shape</div>
            <select
                class="block p-1"
                on:change=move |ev| {
                    let shape = event_target_value(&ev);
                    let preset = match shape.as_ref() {
                        "uniform" => SPOT_UNIFORM.clone(),
                        "curve" => CURVE.clone(),
                        "bid-ask" => BID_ASK.clone(),
                        _ => panic!("Invalid liquidity shape"),
                    };
                    set_liquidity_shape.set(shape);
                    set_liquidity_configuration.set(preset);
                    set_radius.set("5".to_string());
                }
            >
                <option value="uniform">"Spot/Uniform"</option>
                <option value="curve">"Curve"</option>
                <option value="bid-ask">"Bid-Ask"</option>
            </select>

            <div class="flex items-center gap-2 !mt-6">
                <div class="text-xl font-semibold mr-auto">Price</div>
                <button on:click=move |_| {
                    set_price_by.set(Some("range".to_string()));
                }>"By Range"</button>
                <button on:click=move |_| {
                    set_price_by.set(Some("radius".to_string()));
                }>"By Radius"</button>
            </div>

            <Show when=move || price_by() == "range">
                <div class="font-mono">"todo!()"</div>
            </Show>
            <Show when=move || price_by() == "radius">
                <div class="flex items-center gap-2">
                    <div class="basis-1/3">"Target Price:"</div>
                    <input
                        class="p-1 basis-2/3"
                        type="decimal"
                        placeholder="Enter Target Price"
                        min="0"
                        prop:value=move || target_price.get()
                        on:change=move |ev| set_target_price.set(event_target_value(&ev))
                    />
                </div>
                <div class="flex items-center gap-2">
                    <div class="basis-1/3">"Radius:"</div>
                    <input
                        class="p-1 basis-2/3"
                        type="number"
                        placeholder="Enter Bin Radius"
                        min="0"
                        prop:value=move || radius.get()
                        on:change=move |ev| set_radius.set(event_target_value(&ev))
                    />
                </div>

                <button class="w-full p-1 !mt-6" on:click=add_liquidity>
                    "Add Liquidity"
                </button>
            </Show>
        </div>
    }
}
