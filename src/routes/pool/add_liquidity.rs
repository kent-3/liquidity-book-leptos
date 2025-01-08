use crate::{
    constants::Querier,
    error::Error,
    liquidity_book::{
        constants::{
            addrs::{LB_CONTRACTS, LB_FACTORY, LB_ROUTER},
            liquidity_config::{
                LiquidityConfigurations, LiquidityShape, BID_ASK, CURVE, SPOT_UNIFORM, WIDE,
            },
        },
        contract_interfaces::{
            lb_factory::{self, LbPairInformation},
            lb_router::{self, AddLiquidityResponse, LiquidityParameters},
        },
        curves::configure_liquidity_by_radius,
        utils::*,
    },
    prelude::{CHAIN_ID, GRPC_URL},
    state::*,
    utils::{alert, latest_block},
};
use cosmwasm_std::{Addr, ContractInfo, Uint128, Uint64};
use ethnum::U256;
use keplr::Keplr;
use leptos::{logging::*, prelude::*};
use leptos_router::{
    hooks::{query_signal, query_signal_with_options, use_params, use_params_map, use_query_map},
    NavigateOptions,
};
use liquidity_book::libraries::{constants::PRECISION, PriceHelper, U128x128Math};
use rsecret::{
    query::tendermint::TendermintQuerier,
    secret_client::{CreateTxSenderOptions, TxDecrypter},
    tx::{compute::MsgExecuteContractRaw, ComputeServiceClient},
    TxOptions,
};
use secretrs::{
    compute::{MsgExecuteContract, MsgExecuteContractResponse},
    tx::Msg,
    AccountId,
};
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
    let bin_step = move || {
        params
            .read()
            .get("bps") // bps = basis points
            .and_then(|string| string.parse::<u16>().ok())
            .unwrap_or_default()
    };

    // prevents scrolling to the top of the page each time a query param changes
    let nav_options = NavigateOptions {
        scroll: false,
        ..Default::default()
    };
    let (active_id, _) = query_signal::<u32>("active_id");
    let (price_by, set_price_by) =
        query_signal_with_options::<String>("price_by", nav_options.clone());

    // TODO: Figure out how to obtain the token code hashes efficiently.
    let lb_pair_information = Resource::new(
        move || (token_a(), token_b(), bin_step()),
        move |(token_a, token_b, bin_step)| {
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
                .unwrap_or_default()
            })
        },
    );

    let price_by = move || price_by.get().unwrap_or("radius".to_string());

    let (token_x_amount, set_token_x_amount) = signal("0.0".to_string());
    let (token_y_amount, set_token_y_amount) = signal("0.0".to_string());
    let (liquidity_shape, set_liquidity_shape) = signal(LiquidityShape::SpotUniform);
    let (liquidity_configuration, set_liquidity_configuration) = signal(SPOT_UNIFORM.clone());

    let (target_price, set_target_price) = signal("Loading...".to_string());

    // TODO: I think we need to go straight to 128x128 fixed point number from the float input
    let target_bin = move || {
        let price = target_price.get();
        let price = parse_to_decimal_price(&price);
        let price = PriceHelper::convert_decimal_price_to128x128(price).unwrap_or_default();

        let target_bin = PriceHelper::get_id_from_price(price, bin_step()).unwrap_or_default();

        target_bin
    };

    let (radius, set_radius) = signal("5".to_string());

    let (amount_slippage, set_amount_slippage) = signal(0.01); // idk why this would be necessary

    // TODO: wherever the input for this is, we need to convert it from percentage to basis points
    let (price_slippage, set_price_slippage) = signal(200u32); // for if the active bin id moves

    // id_slippage = price_slippage (in basis points) / bin_step (in basis points)
    //
    let id_slippage = move || price_slippage.get() / bin_step() as u32;

    // old inefficient way to calculate bin id slippage, but might be useful for something else
    // let id_slippage = move || {
    // let target_price = parse_to_decimal_price(&target_price.get());
    // let slippage = parse_to_decimal_price(&price_slippage.get());
    //
    // debug!("{target_price}");
    // debug!("{slippage}");
    //
    // let slippage_price = (PRECISION - slippage) * target_price / PRECISION;
    //
    // debug!("{slippage_price}");
    //
    // let slippage_price =
    //     PriceHelper::convert_decimal_price_to128x128(slippage_price).unwrap_or_default();
    //
    // let slippage_bin =
    //     PriceHelper::get_id_from_price(slippage_price, bin_step()).unwrap_or_default();
    //
    // // TODO: is this too many levels of derived signals?
    // target_bin().abs_diff(slippage_bin)
    // };

    // debug Effects
    Effect::new(move || debug!("amount_slippage: {}", amount_slippage.get()));
    Effect::new(move || debug!("price_slippage: {}", price_slippage.get()));
    Effect::new(move || debug!("target_bin: {}", target_bin()));
    Effect::new(move || debug!("id_slippage: {}", id_slippage()));
    //

    Effect::new(move || {
        active_id
            .get()
            .and_then(|id| PriceHelper::get_price_from_id(id, bin_step()).ok())
            .and_then(|price| PriceHelper::convert128x128_price_to_decimal(price).ok())
            .map(|price| u128_to_string_with_precision(price.as_u128()))
            .map(|price| set_target_price.set(price));
    });

    fn amount_min(input: &str, slippage: f64) -> u128 {
        let number: f64 = input.parse().expect("Error parsing float");
        let adjusted_value = number * (1.0 - slippage);
        adjusted_value.round() as u128
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

    // even tho this is a derived signal, it's not run automatically whenever the signals it
    // uses change (I think). It's only running when something calls it (in this case, on click)
    let liquidity_parameters = move || {
        // By using the information returned by query, we can be sure it is correct
        let Some(lb_pair_information) = lb_pair_information.get() else {
            return Err(Error::generic("LB pair information missing"));
        };
        let token_x = lb_pair_information.lb_pair.token_x;
        let token_y = lb_pair_information.lb_pair.token_y;
        let bin_step = lb_pair_information.bin_step;

        // TODO: need a way to convert token amounts like 1.5 to 1_500_000
        // need one map from token address to token info (the thing with the name, symbol, decimals)
        // use a separate map for address -> code hash (can be used for any contract)
        let amount_x = token_x_amount.get();
        let amount_y = token_y_amount.get();
        let target_price = target_price.get();

        let shape = liquidity_shape.get();
        let radius = radius.get().parse::<u32>().unwrap();
        // let range = range.get();

        let amount_slippage = amount_slippage.get();

        let amount_x_min = amount_min(&amount_x, amount_slippage);
        let amount_y_min = amount_min(&amount_y, amount_slippage);

        let target_price = target_price.parse::<f64>().expect("Invalid price format");
        let target_bin = get_id_from_price(target_price, bin_step);

        let liq = match price_by().as_str() {
            "radius" => LiquidityConfigurations::by_radius(target_bin, radius, shape),
            "range" => todo!(),
            _ => unimplemented!(),
        };

        // let liq = configure_liquidity_by_radius(target_bin, radius, shape);

        // this gets it from the preset configurations
        // let liq = liquidity_configuration.get();

        let liquidity_parameters = LiquidityParameters {
            token_x,
            token_y,
            bin_step,
            amount_x: Uint128::from_str(&amount_x).unwrap(),
            amount_y: Uint128::from_str(&amount_y).unwrap(),
            amount_x_min: Uint128::from(amount_x_min),
            amount_y_min: Uint128::from(amount_y_min),
            active_id_desired: target_bin,
            id_slippage: id_slippage(),
            delta_ids: liq.delta_ids(),
            distribution_x: liq.distribution_x(),
            distribution_y: liq.distribution_y(),
            to: String::new(),
            refund_to: String::new(),
            deadline: Uint64::MIN,
        };

        Ok(liquidity_parameters)
    };

    let add_liquidity_action =
        Action::new_local(move |liquidity_parameters: &LiquidityParameters| {
            // TODO: Use the dynamic versions instead.
            // let url = endpoint.get();
            // let chain_id = chain_id.get();
            let url = GRPC_URL;
            let chain_id = CHAIN_ID;
            let mut liquidity_parameters = liquidity_parameters.clone();

            async move {
                if liquidity_parameters.amount_x.is_zero()
                    && liquidity_parameters.amount_y.is_zero()
                {
                    alert("Amounts must not be 0!");
                    return Err(Error::generic("Amounts must not be 0!"));
                }

                let key = Keplr::get_key(&chain_id).await?;
                keplr.enabled.set(true);
                // let wallet = Keplr::get_offline_signer_only_amino(&chain_id);
                let wallet = Keplr::get_offline_signer(&chain_id);
                let enigma_utils = Keplr::get_enigma_utils(&chain_id).into();

                // TODO: I need to make this type use Strings instead of &'static str, because the
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
                liquidity_parameters.to = key.bech32_address.clone();
                liquidity_parameters.refund_to = key.bech32_address.clone();

                debug!("{liquidity_parameters:#?}");

                let lb_router_contract = &LB_ROUTER;

                // NOTE: here we are encrypting the messages manually so we can broadcast them all
                // together. (The client doesn't have a way to handle this internally yet)

                let increase_x_allowance_msg = MsgExecuteContract {
                    sender: AccountId::from_str(key.bech32_address.as_ref())?,
                    contract: AccountId::from_str(liquidity_parameters.token_x.address().as_str())?,
                    msg: compute_service_client
                        .encrypt(
                            &liquidity_parameters.token_x.code_hash(),
                            &secret_toolkit_snip20::HandleMsg::IncreaseAllowance {
                                spender: lb_router_contract.address.to_string(),
                                amount: liquidity_parameters.amount_x.clone(),
                                expiration: None,
                                padding: None,
                            },
                        )
                        .await?
                        .into_inner(),
                    sent_funds: vec![],
                };

                let increase_y_allowance_msg = MsgExecuteContract {
                    sender: AccountId::from_str(key.bech32_address.as_ref())?,
                    contract: AccountId::from_str(liquidity_parameters.token_y.address().as_str())?,
                    msg: compute_service_client
                        .encrypt(
                            &liquidity_parameters.token_y.code_hash(),
                            &secret_toolkit_snip20::HandleMsg::IncreaseAllowance {
                                spender: lb_router_contract.address.to_string(),
                                amount: liquidity_parameters.amount_y.clone(),
                                expiration: None,
                                padding: None,
                            },
                        )
                        .await?
                        .into_inner(),
                    sent_funds: vec![],
                };

                let add_liquidity_msg = MsgExecuteContract {
                    sender: AccountId::from_str(key.bech32_address.as_ref())?,
                    contract: AccountId::from_str(lb_router_contract.address.as_ref())?,
                    msg: compute_service_client
                        .encrypt(
                            &lb_router_contract.code_hash,
                            &lb_router::ExecuteMsg::AddLiquidity {
                                liquidity_parameters,
                            },
                        )
                        .await?
                        .into_inner(),
                    sent_funds: vec![],
                };

                let tx_options = TxOptions {
                    gas_limit: 1_000_000,
                    ..Default::default()
                };

                // let tx = compute_service_client
                //     .execute_contract(msg, lb_router_contract.code_hash.clone(), tx_options)
                //     .await
                //     .map_err(Error::from)
                //     .inspect(|tx_response| info!("{tx_response:?}"))
                //     .inspect_err(|error| error!("{error}"))?;

                let tx = compute_service_client
                    .broadcast(
                        vec![
                            increase_x_allowance_msg,
                            increase_y_allowance_msg,
                            add_liquidity_msg,
                        ],
                        tx_options,
                    )
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
            }
        });

    let add_liquidity = move |_: MouseEvent| {
        let _ = add_liquidity_action
            .dispatch(liquidity_parameters().expect("invalid liquidity_parameters"));
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
                    set_liquidity_shape.set(shape.into());
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
