use leptos::prelude::*;
use leptos_router::components::A;
use shade_protocol::{
    c_std::{Addr, ContractInfo, Uint128, Uint256},
    contract_interfaces::liquidity_book::lb_pair::LBPair,
    swap::core::{TokenAmount, TokenType},
};
use tracing::{debug, info};

#[component]
pub fn PoolBrowser() -> impl IntoView {
    info!("rendering <PoolBrowser/>");

    on_cleanup(move || {
        info!("cleaning up <PoolBrowser/>");
    });

    let pair_1 = LBPair {
        token_x: TokenType::CustomToken {
            contract_addr: Addr::unchecked("secret1cm5tnqda0vecc6tx0vlp7fpfwj5uu85xn3vu2n"),
            token_code_hash: "0bbaa17a6bd4533f5dc3eae14bfd1152891edaabcc0d767f611bb70437b3a159"
                .to_string(),
        },
        token_y: TokenType::CustomToken {
            contract_addr: Addr::unchecked("secret1072wn6au3m002vsz8ej7upnudm3evuz7vujjwl"),
            token_code_hash: "0bbaa17a6bd4533f5dc3eae14bfd1152891edaabcc0d767f611bb70437b3a159"
                .to_string(),
        },
        bin_step: 100,
        contract: ContractInfo {
            address: Addr::unchecked("secret1pt5nd3fuevamy5lqcv53jqsvytspmknanf5c28"),
            code_hash: "9768cfd5753a7fa2b51b30a3fc41632df2b3bc31801dece2d6111f321a3e4252"
                .to_string(),
        },
    };

    let pair_2 = LBPair {
        token_x: TokenType::CustomToken {
            contract_addr: Addr::unchecked("bar"),
            token_code_hash: "code_hash".to_string(),
        },
        token_y: TokenType::CustomToken {
            contract_addr: Addr::unchecked("baz"),
            token_code_hash: "code_hash".to_string(),
        },
        bin_step: 100,
        contract: ContractInfo {
            address: Addr::unchecked("a second LB Pair"),
            code_hash: "9768cfd5753a7fa2b51b30a3fc41632df2b3bc31801dece2d6111f321a3e4252"
                .to_string(),
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
