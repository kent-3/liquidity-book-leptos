use crate::liquidity_book::{
    constants::liquidity_config::{BID_ASK, CURVE, SPOT_UNIFORM, WIDE},
    contract_interfaces::{
        lb_factory,
        lb_pair::{self, LiquidityParameters},
    },
};
use leptos::prelude::*;
use leptos_router::{
    components::A,
    hooks::{use_params, use_params_map, use_query_map},
};
use shade_protocol::c_std::Uint128;
use std::str::FromStr;
use tracing::{debug, info};

#[component]
pub fn AddLiquidity() -> impl IntoView {
    info!("rendering <AddLiquidity/>");

    let params = use_params_map();
    let token_a = move || params.read().get("token_a").unwrap_or("foo".to_string());
    let token_b = move || params.read().get("token_b").unwrap_or("bar".to_string());
    let basis_points = move || params.read().get("bps").unwrap_or("100".to_string());
    // TODO: use a map to translate token name/address to ContractInfo

    // TODO: use a lb_factory query to get the lb_pair contract info
    // let lb_pair_information = Resource::new(
    //     || (),
    //     move |_| {
    //         SendWrapper::new(async move {
    //             let query = lb_factory::QueryMsg::GetLbPairInformation {
    //                 token_x: token_a(),
    //                 token_y: token_b(),
    //                 bin_step: basis_points(),
    //             };
    //         })
    //     },
    // );

    let query = use_query_map();
    let price = move || query.read().get("price").unwrap_or("radius".to_string());

    let (token_x_amount, set_token_x_amount) = signal("0".to_string());
    let (token_y_amount, set_token_y_amount) = signal("0".to_string());
    let (liquidity_shape, set_liquidity_shape) = signal("uniform".to_string());

    // TODO: This doesn't really make sense currently...
    // When the shape is selected, the liquidity configuration should be set based on the
    // constants, but then be customized by user inputs.
    let liquidity_config = move || match liquidity_shape.get().as_ref() {
        "uniform" => SPOT_UNIFORM,
        "curve" => CURVE,
        "bid-ask" => BID_ASK,
        _ => panic!("Invalid liquidity shape"),
    };

    let (target_price, set_target_price) = signal("1.00".to_string());
    let (radius, set_radius) = signal("5".to_string());

    // Effect::new(move || debug!("{:?}", token_x_amount.get()));
    // Effect::new(move || debug!("{:?}", token_y_amount.get()));
    // Effect::new(move || debug!("{:?}", liquidity_shape.get()));

    fn get_id_from_price(price: f64, bin_step: f64) -> u32 {
        ((price.ln() / (1.0 + bin_step / 10_000.0).ln()).trunc() as u32) + 8_388_608
    }

    fn adjust_value(input: &str) -> String {
        let number: f64 = input.parse().expect("Error parsing float");
        let adjusted_value = number * 0.95;
        adjusted_value.to_string()
    }

    // TODO: figure out how to transform inputs into a LiquidityConfig dynamically
    let add_liquidity = move |_| {
        let amount_x = token_x_amount.get();
        let amount_y = token_y_amount.get();
        let shape = liquidity_shape.get();
        let target_price = target_price.get();
        let radius = radius.get();

        let amount_x_min = adjust_value(&amount_x);
        let amount_y_min = adjust_value(&amount_y);

        let price: f64 = target_price.parse::<f64>().expect("Invalid price format");
        let bin_step = basis_points().parse::<f64>().unwrap();
        let active_bin = get_id_from_price(price, bin_step);

        debug!("inputs: {:?}, {:?}, {:?}", price, bin_step, active_bin);

        // let token_x = token_x.get();
        // let token_y = token_y.get();
        let bin_step = basis_points().parse::<u16>().unwrap();

        let liquidity_parameters = LiquidityParameters {
            token_x: todo!(),
            token_y: todo!(),
            bin_step,
            amount_x: Uint128::from_str(&amount_x).unwrap(),
            amount_y: Uint128::from_str(&amount_y).unwrap(),
            amount_x_min: Uint128::from_str(&amount_x_min).unwrap(),
            amount_y_min: Uint128::from_str(&amount_y_min).unwrap(),
            active_id_desired: 2u32.pow(23),
            id_slippage: todo!(),
            delta_ids: SPOT_UNIFORM.delta_ids(),
            distribution_x: SPOT_UNIFORM.distribution_x(),
            distribution_y: SPOT_UNIFORM.distribution_y(),
            deadline: 999999999999999,
        };
    };

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
                on:change=move |ev| set_liquidity_shape.set(event_target_value(&ev))
            >
                // on:input=move |_ev| navigate("&shape=curve", nav_options.clone())
                <option value="uniform">"Spot/Uniform"</option>
                <option value="curve">"Curve"</option>
                <option value="bid-ask">"Bid-Ask"</option>
            </select>
            <div class="flex items-center gap-2 !mt-6">
                <div class="text-xl font-semibold mr-auto">Price</div>
                <A href="?price=range">
                    <button>By Range</button>
                </A>
                <A href="?price=radius">
                    <button>By Radius</button>
                </A>
            </div>
            <Show when=move || price() == "range">
                <div class="font-mono">"todo!()"</div>
            </Show>
            <Show when=move || price() == "radius">
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

                <button class="w-full p-1 !mt-6" on:click=add_liquidity>"Add Liquidity"</button>
            </Show>
        </div>
    }
}
