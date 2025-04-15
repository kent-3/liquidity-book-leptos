use ammber_core::Error;
use cosmwasm_std::ContractInfo;
use ethnum::U256;
use leptos::prelude::*;
use liquidity_book::interfaces::lb_pair::{LbPair, ReservesResponse, StaticFeeParametersResponse};
use reactive_stores::{Field, Store};
use serde::{Deserialize, Serialize};

// TODO: use a single struct for pool context?
// Currently providing all these separately:
// token symbols, lb_pair, active_id, total_reserves, static_fee_parameters
//
// Flow:
//     1) Get token addresses from the URL params
//     2) Convert them into ContractInfo (use addr_2_token helper function)
//     3) Query the LbPair from the given token contracts and bin step
//         (cache this in local storage - it's not likely to ever change)
//     4) Query the LbPair contract for the active_id, total_reserves, and static_fee_parameters
//         (These could be batched)

// #[derive(Debug, Store, Serialize, Deserialize)]
// pub struct PoolState {
// token_x: ContractInfo,
// token_y: ContractInfo,
// bin_step: u16,
// lb_pair: ContractInfo,
// active_id: u32,
// target_price: U256,
// total_reserves: ReservesResponse,
// static_fee_parameters: StaticFeeParametersResponse,
// }

#[derive(Store, Clone)]
pub struct PoolState {
    pub lb_pair: Result<ContractInfo, Error>,
    pub active_id: Result<u32, Error>,
}
