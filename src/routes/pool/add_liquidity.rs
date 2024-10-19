use crate::{
    liquidity_book::{
        constants::liquidity_config::{
            LiquidityConfiguration, LiquidityShape, BID_ASK, CURVE, SPOT_UNIFORM, WIDE,
        },
        contract_interfaces::{
            lb_factory,
            // TODO: rename all LB* to Lb*
            lb_pair::{self, LBPair, LBPairInformation, LiquidityParameters},
        },
    },
    state::*,
};
use lb_libraries::math::liquidity_configurations::LiquidityConfigurations;
use leptos::prelude::*;
use leptos_router::{
    components::A,
    hooks::{query_signal_with_options, use_params, use_params_map, use_query_map},
    NavigateOptions,
};
use rsecret::query::tendermint::TendermintQuerier;
use send_wrapper::SendWrapper;
use shade_protocol::{
    c_std::{Addr, ContractInfo, Uint128},
    liquidity_book::lb_factory::{LbPairInformationResponse, QueryMsg::GetLbPairInformation},
    swap::core::TokenType,
};
use std::str::FromStr;
use tonic_web_wasm_client::Client;
use tracing::{debug, info};

#[component]
pub fn AddLiquidity() -> impl IntoView {
    info!("rendering <AddLiquidity/>");

    let endpoint = use_context::<Endpoint>().expect("endpoint context missing!");
    let keplr = use_context::<KeplrSignals>().expect("keplr signals context missing!");
    let token_map = use_context::<TokenMap>().expect("tokens context missing!");

    let params = use_params_map();
    let token_a = move || params.read().get("token_a").unwrap_or("foo".to_string());
    let token_b = move || params.read().get("token_b").unwrap_or("bar".to_string());
    let basis_points = move || params.read().get("bps").unwrap_or("100".to_string());

    // prevents scrolling to the top of the page each time a query param changes
    let nav_options = NavigateOptions {
        scroll: false,
        ..Default::default()
    };

    let (price_option, set_price_option) =
        query_signal_with_options::<String>("price", nav_options.clone());

    // TODO: use a lb_factory query to get the lb_pair contract info
    let lb_pair_information = Resource::new(
        move || (token_a(), token_b(), basis_points()),
        move |(token_a, token_b, basis_points)| {
            // TODO: Map token address to contract info and more
            let token_x = TokenType::CustomToken {
                contract_addr: Addr::unchecked(&token_a),
                token_code_hash: "foo".to_string(),
            };
            let token_y = TokenType::CustomToken {
                contract_addr: Addr::unchecked(&token_b),
                token_code_hash: "bar".to_string(),
            };
            let bin_step = basis_points
                .parse::<u16>()
                .expect("Invalid basis_points value");

            SendWrapper::new(async move {
                // TODO: do actual query
                // TODO: this query should just take token addresses, I think. It can take in
                // strings and give back structured data to be used in the execute msg.
                let query = lb_factory::QueryMsg::GetLbPairInformation {
                    token_x: token_x.clone(),
                    token_y: token_y.clone(),
                    bin_step,
                };

                let LbPairInformationResponse {
                    lb_pair_information,
                } = LbPairInformationResponse {
                    lb_pair_information: LBPairInformation {
                        bin_step: 100,
                        info: LBPair {
                            token_x,
                            token_y,
                            bin_step: 100,
                            contract: ContractInfo {
                                address: Addr::unchecked("secretxyz"),
                                code_hash: "lb_pair_code_hash".to_string(),
                            },
                        },
                        created_by_owner: true,
                        ignored_for_routing: false,
                    },
                };

                lb_pair_information
            })
        },
    );

    let query = use_query_map();
    let price = move || price_option.get().unwrap_or("by_radius".to_string());

    let (token_x_amount, set_token_x_amount) = signal("0".to_string());
    let (token_y_amount, set_token_y_amount) = signal("0".to_string());
    let (liquidity_shape, set_liquidity_shape) = signal("uniform".to_string());

    let liquidity_configuration_preset = match liquidity_shape.get_untracked().as_ref() {
        "uniform" => SPOT_UNIFORM.clone(),
        "curve" => CURVE.clone(),
        "bid-ask" => BID_ASK.clone(),
        "wide" => WIDE.clone(),
        _ => panic!("invalid liquidity shape"),
    };

    let (liquidity_configuration, set_liquidity_configuration) =
        signal(liquidity_configuration_preset);

    let (target_price, set_target_price) = signal("1.00".to_string());
    let (radius, set_radius) = signal("5".to_string());

    fn get_id_from_price(price: f64, bin_step: impl Into<f64>) -> u32 {
        ((price.ln() / (1.0 + bin_step.into() / 10_000.0).ln()).trunc() as u32) + 8_388_608
    }

    fn amount_min(input: &str) -> u32 {
        let number: f64 = input.parse().expect("Error parsing float");
        let adjusted_value = number * 0.95 * 1_000_000.0;
        adjusted_value.round() as u32
    }

    let latest_block = Resource::new(
        move || endpoint.get(),
        move |endpoint| {
            SendWrapper::new(async move {
                let tendermint = TendermintQuerier::new(Client::new(endpoint));
                let latest_block = tendermint.get_latest_block().await;

                latest_block
                    .and_then(|block| Ok(block.header.height))
                    .inspect(|height| debug!("{:#?}", height))
                    .map_err(|e| crate::Error::from(e))
            })
        },
    );

    let liquidity_parameters = move |_| {
        // By using the information returned by query, we can be sure it is correct
        let lb_pair_information = lb_pair_information
            .get()
            .expect("Unverified LB Pair information");
        // TODO: awkward naming... lb_pair_information.info is type LbPair
        let token_x = lb_pair_information.info.token_x;
        let token_y = lb_pair_information.info.token_y;
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
        let distribution_x = liquidity_configuration.distribution_x(6);
        let distribution_y = liquidity_configuration.distribution_y(6);

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
            deadline: latest_block
                .get()
                .map(|res| res.ok())
                .flatten()
                .unwrap()
                .value()
                + 100,
            // deadline: 999999999999999,
        };

        debug!("{:#?}", liquidity_parameters);

        // liquidity_parameters
    };

    let add_liquidity = Action::new(move |_: &()| {
        let url = endpoint.get();
        SendWrapper::new(async move {
            let tendermint = TendermintQuerier::new(Client::new(url));
            let latest_block = tendermint.get_latest_block().await;

            latest_block
                .and_then(|block| Ok(block.header.height))
                .inspect(|height| debug!("{:#?}", height))
        })
    });

    // liquidityParameters:
    // 	active_id_desired: 2 ** 23,
    // 	amount_x: amountX.toFixed(0),
    // 	amount_y: amountY.toFixed(0),
    // 	amount_x_min: (0.95 * amountX).toFixed(0),
    // 	amount_y_min: (0.95 * amountY).toFixed(0),
    // 	bin_step: binStep,
    // 	deadline: 999999999999999,
    // 	delta_ids: [-5, -4, -3, -2, -1, 0, 1, 2, 3, 4, 5].map((el) => el * binStep),
    // 	distribution_x: [
    // 		0, 0, 0, 0, 0, 0.090909, 0.181818, 0.181818, 0.181818, 0.181818, 0.181818
    // 	].map((el) => el * 1e18),
    // 	distribution_y: [
    // 		0.181818, 0.181818, 0.181818, 0.181818, 0.181818, 0.090909, 0, 0, 0, 0, 0
    // 	].map((el) => el * 1e18),
    // 	id_slippage: 10,
    // 	token_x: tokenX,
    // 	token_y: tokenY

    // let navigate = leptos_router::hooks::use_navigate();
    // let nav_options = NavigateOptions {
    //     resolve: true,
    //     replace: true,
    //     scroll: false,
    //     state: leptos_router::location::State::new(None),
    // };

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
                <div class="basis-1/3 text-md text-ellipsis line-clamp-1">{token_b}</div>
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
                    set_target_price.set("1.00".to_string());
                }
            >
                // on:input=move |_ev| navigate("&shape=curve", nav_options.clone())
                <option value="uniform">"Spot/Uniform"</option>
                <option value="curve">"Curve"</option>
                <option value="bid-ask">"Bid-Ask"</option>
            </select>
            <div class="flex items-center gap-2 !mt-6">
                <div class="text-xl font-semibold mr-auto">Price</div>
                <button on:click=move |_| {
                    set_price_option.set(Some("by_range".to_string()));
                }>"By Range"</button>
                <button on:click=move |_| {
                    set_price_option.set(Some("by_radius".to_string()));
                }>"By Radius"</button>
            </div>
            <Show when=move || price() == "by_range">
                <div class="font-mono">"todo!()"</div>
            </Show>
            <Show when=move || price() == "by_radius">
                <div class="flex items-center gap-2">
                    <div class="basis-1/3">"Target Price:"</div>
                    <input
                        class="p-1 basis-2/3"
                        type="number"
                        placeholder="Enter Target Price"
                        value="1.00"
                        min="0"
                        // prop:value=move|| target_price.get()
                        on:change=move |ev| set_target_price.set(event_target_value(&ev))
                    />
                </div>
                <div class="flex items-center gap-2">
                    <div class="basis-1/3">"Radius:"</div>
                    <input
                        class="p-1 basis-2/3"
                        type="number"
                        placeholder="Enter Bin Radius"
                        value="5"
                        min="0"
                        // prop:value=move|| radius.get()
                        on:change=move |ev| set_radius.set(event_target_value(&ev))
                    />
                </div>
                // <div class="grid grid-cols-2 items-center gap-2" >
                // <div class="">"Target Price:"</div>
                // <input
                // class="p-1"
                // type="number"
                // placeholder="Enter Amount"
                // on:change=move |ev| set_target_price.set(event_target_value(&ev))
                // />
                // <div class="">"Radius:"</div>
                // <input
                // class="p-1"
                // type="number"
                // placeholder="Enter Amount"
                // on:change=move |ev| set_radius.set(event_target_value(&ev))
                // />
                // </div>

                <button class="w-full p-1 !mt-6" on:click=liquidity_parameters>
                    "Add Liquidity"
                </button>
                <button class="w-full p-1 !mt-6" on:click=move |_| _ = add_liquidity.dispatch(())>
                    "get latest block"
                </button>
            </Show>
        </div>
    }
}
