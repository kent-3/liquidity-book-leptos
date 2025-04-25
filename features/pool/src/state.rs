use ammber_core::{prelude::Token, Error};
use cosmwasm_std::{Addr, ContractInfo, Uint128};
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

// TODO: decide if any of these should be Result types
#[derive(Debug, Store, Serialize, Deserialize)]
pub struct PoolState {
    token_x: Token,
    token_y: Token,
    bin_step: u16,
    lb_pair: LbPair,
    active_id: u32,
    target_price: U256,
    total_reserves: ReservesResponse,
    static_fee_parameters: StaticFeeParametersResponse,
}

impl Default for PoolState {
    fn default() -> Self {
        PoolState {
            token_x: Token::default(),
            token_y: Token::default(),
            bin_step: 0u16,
            lb_pair: LbPair {
                token_x: liquidity_book::core::TokenType::CustomToken {
                    contract_addr: Addr::unchecked(""),
                    token_code_hash: String::new(),
                },
                token_y: liquidity_book::core::TokenType::CustomToken {
                    contract_addr: Addr::unchecked(""),
                    token_code_hash: String::new(),
                },
                bin_step: 0u16,
                contract: ContractInfo {
                    address: Addr::unchecked(""),
                    code_hash: String::new(),
                },
            },
            active_id: 8388608u32,
            target_price: U256::ZERO,
            total_reserves: ReservesResponse {
                reserve_x: Uint128::zero(),
                reserve_y: Uint128::zero(),
            },
            static_fee_parameters: StaticFeeParametersResponse {
                base_factor: 0u16,
                filter_period: 0u16,
                decay_period: 0u16,
                reduction_factor: 0u16,
                variable_fee_control: 0u32,
                protocol_share: 0u16,
                max_volatility_accumulator: 0u32,
            },
        }
    }
}

// #[derive(Store, Clone)]
// pub struct PoolState {
//     pub lb_pair: Result<ContractInfo, Error>,
//     pub active_id: Result<u32, Error>,
// }
