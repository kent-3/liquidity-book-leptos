use crate::liquidity_book::{
    constants::addrs::{LB_CONTRACTS, LB_PAIR},
    contract_interfaces::lb_pair::LbPair,
};
use cosmwasm_std::{Addr, ContractInfo, Uint128, Uint256};
use leptos::prelude::*;
use leptos_router::components::A;
use shade_protocol::swap::core::{TokenAmount, TokenType};
use tracing::{debug, info};

#[component]
pub fn PoolBrowser() -> impl IntoView {
    info!("rendering <PoolBrowser/>");

    on_cleanup(move || {
        info!("cleaning up <PoolBrowser/>");
    });

    let pair_1 = LbPair {
        token_x: TokenType::CustomToken {
            contract_addr: LB_CONTRACTS.snip25.address.clone(),
            token_code_hash: LB_CONTRACTS.snip25.code_hash.clone(),
        },
        token_y: TokenType::CustomToken {
            contract_addr: LB_CONTRACTS.snip20.address.clone(),
            token_code_hash: LB_CONTRACTS.snip20.code_hash.clone(),
        },
        bin_step: 100,
        contract: ContractInfo {
            address: LB_PAIR.address.clone(),
            code_hash: LB_PAIR.code_hash.clone(),
        },
    };

    let pair_2 = LbPair {
        token_x: TokenType::CustomToken {
            contract_addr: Addr::unchecked("foo"),
            token_code_hash: "code_hash".to_string(),
        },
        token_y: TokenType::CustomToken {
            contract_addr: Addr::unchecked("bar"),
            token_code_hash: "code_hash".to_string(),
        },
        bin_step: 100,
        contract: ContractInfo {
            address: Addr::unchecked("a second LB Pair"),
            code_hash: "tbd".to_string(),
        },
    };

    // TODO: query for the pools
    let pools = vec![pair_1, pair_2];

    // let resource = use_context::<LocalResource<String>>().expect("Context missing!");

    view! {
        <div class="text-3xl font-bold">"Pool"</div>
        <div class="text-sm text-neutral-400">"Provide liquidity and earn fees."</div>

        <h3 class="mb-1">"Existing Pools"</h3>
        <ul>
            {pools
                .into_iter()
                .map(|n| {
                    view! {
                        <li>
                            <a href=format!(
                                "/trader-crow-leptos/pool/{}/{}/{}",
                                match n.token_x {
                                    TokenType::CustomToken { contract_addr, .. } => {
                                        contract_addr.to_string()
                                    }
                                    TokenType::NativeToken { denom } => denom,
                                },
                                match n.token_y {
                                    TokenType::CustomToken { contract_addr, .. } => {
                                        contract_addr.to_string()
                                    }
                                    TokenType::NativeToken { denom } => denom,
                                },
                                n.bin_step,
                            )>{n.contract.address.to_string()}</a>
                        </li>
                    }
                })
                .collect_view()}
        </ul>

        // <h3>{move || resource.get()}</h3>

        <div class="mt-4">
            <A href="/trader-crow-leptos/pool/create">
                <button class="p-1">"Create New Pool"</button>
            </A>
        </div>
    }
}
