use crate::chain_query;
use crate::support::COMPUTE_QUERIER;
use crate::{error::Error, prelude::*, state::*, support::Querier};
use ammber_sdk::{
    constants::liquidity_config::{
        LiquidityConfigurations, LiquidityShape, BID_ASK, CURVE, SPOT_UNIFORM, WIDE,
    },
    contract_interfaces::{
        lb_factory::{self, LbPairInformation},
        lb_router::{self, AddLiquidityResponse, LiquidityParameters},
    },
    utils::*,
};
use cosmwasm_std::{Addr, ContractInfo, Uint128, Uint64};
use ethnum::U256;
use keplr::Keplr;
use leptos::{logging::*, prelude::*};
use leptos_router::{
    hooks::{query_signal, query_signal_with_options, use_params, use_params_map, use_query_map},
    NavigateOptions,
};
use liquidity_book::libraries::{PriceHelper, U128x128Math};
use rsecret::{
    query::tendermint::TendermintQuerier,
    secret_client::{CreateTxSenderOptions, TxDecrypter},
    tx::ComputeServiceClient,
    TxOptions,
};
use secret_toolkit_snip20::TokenInfoResponse;
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

    // TODO: move these to support/utils
    async fn addr_2_contract(contract_address: impl Into<String>) -> Result<ContractInfo, Error> {
        let contract_address = contract_address.into();

        if let Some(token) = TOKEN_MAP.get(&contract_address) {
            Ok(ContractInfo {
                address: Addr::unchecked(token.contract_address.clone()),
                code_hash: token.code_hash.clone(),
            })
        } else {
            COMPUTE_QUERIER
                .code_hash_by_contract_address(&contract_address)
                .await
                .map_err(Error::from)
                .map(|code_hash| ContractInfo {
                    address: Addr::unchecked(contract_address),
                    code_hash,
                })
        }
    }
    async fn token_symbol_convert(address: String) -> String {
        if let Some(token) = TOKEN_MAP.get(&address) {
            return token.symbol.clone();
        }
        let contract = addr_2_contract(&address).await.unwrap();

        chain_query::<TokenInfoResponse>(
            contract.address.to_string(),
            contract.code_hash,
            secret_toolkit_snip20::QueryMsg::TokenInfo {},
        )
        .await
        .map(|x| x.token_info.symbol)
        .unwrap_or(address)
    }

    let token_a_symbol =
        AsyncDerived::new_unsync(move || async move { token_symbol_convert(token_a()).await });

    let token_b_symbol =
        AsyncDerived::new_unsync(move || async move { token_symbol_convert(token_b()).await });

    // prevents scrolling to the top of the page each time a query param changes
    let nav_options = NavigateOptions {
        scroll: false,
        ..Default::default()
    };

    let (active_id, _) = query_signal::<u32>("active_id");
    let (price_by, set_price_by) =
        query_signal_with_options::<String>("price_by", nav_options.clone());

    // NOTE: By using the information returned by lb-factory query, we can be sure it is correct.
    // In theory, we could have an off-chain database of (token_x, token_y, bin_step) -> LbPairInformation
    // to reduce the number of chain queries.
    // TODO: Figure out how to obtain the token code hashes efficiently.
    let lb_pair_information = Resource::new(
        move || (token_a(), token_b(), bin_step()), // when any of these values change, the Resource is reloaded
        move |(token_a, token_b, bin_step)| {
            SendWrapper::new(async move {
                // Assume token_x has the code_hash from the current deployment.
                let token_x = ContractInfo {
                    address: Addr::unchecked(token_a),
                    code_hash: LB_AMBER.code_hash.clone(),
                };
                // Assume token_y is sSCRT
                let token_y = ContractInfo {
                    address: Addr::unchecked(token_b),
                    code_hash: LB_SSCRT.code_hash.clone(),
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

    let (amount_x, set_amount_x) = signal("0.0".to_string());
    let (amount_y, set_amount_y) = signal("0.0".to_string());
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

    let (radius, set_radius) = signal(5);
    let (range, set_range) = signal((8_388_608, 8_388_608));

    // TODO: wherever the inputs are for these, need to convert it to/from basis points
    let (amount_slippage, set_amount_slippage) = signal(1); // idk why this is necessary
    let (price_slippage, set_price_slippage) = signal(200); // for if the active bin id moves

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
        // get all the signals

        let amount_x = amount_x.get();
        let amount_y = amount_y.get();

        let amount_slippage = amount_slippage.get();

        let shape = liquidity_shape.get();
        let radius = radius.get();
        let range = range.get();

        let target_bin = target_bin();

        let Some(lb_pair_information) = lb_pair_information.get() else {
            return Err(Error::generic("lb pair information is missing!"));
        };

        // end of signal getting

        let token_x = lb_pair_information.lb_pair.token_x;
        let token_y = lb_pair_information.lb_pair.token_y;
        let bin_step = lb_pair_information.bin_step;

        let decimals_x = get_token_decimals(token_x.address().as_str())?;
        let decimals_y = get_token_decimals(token_y.address().as_str())?;

        let amount_x = parse_token_amount(amount_x, decimals_x);
        let amount_y = parse_token_amount(amount_y, decimals_y);

        // slippage is expressed in basis points (1 = 0.01%)
        let amount_x_min = amount_x * (10_000 - amount_slippage) / 10_000;
        let amount_y_min = amount_y * (10_000 - amount_slippage) / 10_000;

        let liq = match price_by().as_str() {
            "radius" => LiquidityConfigurations::by_radius(target_bin, radius, shape),
            "range" => LiquidityConfigurations::by_range(target_bin, range, shape),
            _ => unimplemented!(),
        };

        let liquidity_parameters = LiquidityParameters {
            token_x,
            token_y,
            bin_step,
            amount_x: Uint128::new(amount_x),
            amount_y: Uint128::new(amount_y),
            amount_x_min: Uint128::new(amount_x_min),
            amount_y_min: Uint128::new(amount_y_min),
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
            let url = NODE;
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
        <div class="max-w-md space-y-2">
            <div class="text-xl font-semibold">Deposit Liquidity</div>
            <div class="flex items-center gap-2">
                <input
                    class="p-1 w-full"
                    type="number"
                    placeholder="Enter Amount"
                    on:change=move |ev| set_amount_x.set(event_target_value(&ev))
                />
                <div class="text-sm p-2">{move || token_a_symbol.get()}</div>
            </div>
            <div class="flex items-center gap-2">
                <input
                    class="p-1 w-full"
                    type="number"
                    placeholder="Enter Amount"
                    on:change=move |ev| set_amount_y.set(event_target_value(&ev))
                />
                <div class="text-sm p-2">{move || token_b_symbol.get()}</div>
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
                    set_radius.set(5);
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
                // <div class="flex items-center gap-2">
                // <div class="basis-1/3">"Target Price:"</div>
                // <input
                // class="p-1 basis-2/3"
                // type="decimal"
                // placeholder="Enter Target Price"
                // min="0"
                // prop:value=move || target_price.get()
                // on:change=move |ev| set_target_price.set(event_target_value(&ev))
                // />
                // </div>
                // <div class="flex items-center gap-2">
                // <div class="basis-1/3">"Radius:"</div>
                // <input
                // class="p-1 basis-2/3"
                // type="number"
                // placeholder="Enter Bin Radius"
                // min="0"
                // prop:value=move || radius.get()
                // on:change=move |ev| {
                // set_radius
                // .set(event_target_value(&ev).parse::<u32>().unwrap_or_default())
                // }
                // />
                // </div>

                <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
                    <div>
                        <label class="block mb-1 text-xs" for="target-price">
                            "Target Price:"
                        </label>
                        <input
                            id="target-price"
                            class="p-1 w-full box-border"
                            type="decimal"
                            placeholder="Enter Target Price"
                            min="0"
                            prop:value=move || target_price.get()
                            on:change=move |ev| set_target_price.set(event_target_value(&ev))
                        />
                    </div>
                    <div>
                        <label class="block mb-1 text-xs" for="radius">
                            "Radius (number of bins):"
                        </label>
                        <input
                            id="radius"
                            class="p-1 w-full box-border"
                            type="number"
                            placeholder="Enter Bin Radius"
                            min="0"
                            prop:value=move || radius.get()
                            on:change=move |ev| {
                                set_radius
                                    .set(event_target_value(&ev).parse::<u32>().unwrap_or_default())
                            }
                        />
                    </div>
                    <div>
                        <label class="block mb-1 text-xs" for="range-min">
                            "Range Min:"
                        </label>
                        <input
                            id="range-min"
                            class="p-1 w-full box-border"
                            type="decimal"
                            placeholder="Range Min"
                            disabled
                        />
                    // prop:value=move || range_min.get()
                    </div>
                    <div>
                        <label class="block mb-1 text-xs" for="range-max">
                            "Range Max:"
                        </label>
                        <input
                            id="range-max"
                            class="p-1 w-full box-border"
                            type="decimal"
                            placeholder="Range Max"
                            disabled
                        />
                    // prop:value=move || range_max.get()
                    </div>
                    <div>
                        <label class="block mb-1 text-xs" for="num-bins">
                            "Num Bins:"
                        </label>
                        <input
                            id="num-bins"
                            class="p-1 w-full box-border"
                            type="number"
                            placeholder="Number of Bins"
                            min="0"
                            disabled
                        />
                    // prop:value=move || num_bins.get()
                    </div>
                    <div>
                        <label class="block mb-1 text-xs" for="pct-range">
                            "Pct Range:"
                        </label>
                        <input
                            id="pct-range"
                            class="p-1 w-full box-border"
                            type="decimal"
                            placeholder="Percentage Range"
                            disabled
                        />
                    // prop:value=move || pct_range.get()
                    </div>
                </div>

                <button class="w-full p-1 !mt-6" on:click=add_liquidity>
                    "Add Liquidity"
                </button>
            </Show>
        </div>
    }
}
