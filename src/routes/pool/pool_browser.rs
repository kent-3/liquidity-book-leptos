use crate::prelude::*;
use crate::Error;
use ammber_sdk::contract_interfaces::lb_pair::LbPair;
use cosmwasm_std::ContractInfo;
use leptos::prelude::*;
use leptos_router::components::A;
use shade_protocol::swap::core::TokenType;
use tracing::{debug, info};

#[component]
pub fn PoolBrowser() -> impl IntoView {
    info!("rendering <PoolBrowser/>");

    on_cleanup(move || {
        info!("cleaning up <PoolBrowser/>");
    });

    let number_of_lb_pairs = use_context::<crate::NumberOfLbPairs>()
        .expect("missing the NumberOfLbPairsResponse resource context")
        .0;
    let all_lb_pairs =
        use_context::<Resource<Vec<LbPair>>>().expect("missing the Vec<LbPair> resource context");

    view! {
        <div class="text-3xl font-bold">"Pool"</div>
        <div class="text-sm text-neutral-400">"Provide liquidity and earn fees."</div>

        <h3 class="mb-1">"Existing Pools - " {move || number_of_lb_pairs.get()}</h3>
        // crazy, but it works
        <Suspense fallback=|| view! { <div>"Loading..."</div> }>
            <ul>
                {move || Suspend::new(async move {
                    all_lb_pairs
                        .await
                        .into_iter()
                        .map(|n| {
                            view! {
                                <li>
                                    <a href=format!(
                                        "/liquidity-book-leptos/pool/{}/{}/{}",
                                        match n.token_x {
                                            TokenType::CustomToken { ref contract_addr, .. } => {
                                                contract_addr.to_string()
                                            }
                                            TokenType::NativeToken { ref denom } => denom.to_string(),
                                        },
                                        match n.token_y {
                                            TokenType::CustomToken { ref contract_addr, .. } => {
                                                contract_addr.to_string()
                                            }
                                            TokenType::NativeToken { ref denom } => denom.to_string(),
                                        },
                                        n.bin_step,
                                    )>{n.contract.address.to_string()}</a>
                                    " - "
                                    {format!(
                                        "[{}, {}, {} bps]",
                                        TOKEN_MAP
                                            .get(&n.token_x.unique_key())
                                            .map(|t| t.symbol.clone())
                                            .unwrap_or_default(),
                                        TOKEN_MAP
                                            .get(&n.token_y.unique_key())
                                            .map(|t| t.symbol.clone())
                                            .unwrap_or_default(),
                                        n.bin_step,
                                    )}
                                </li>
                            }
                        })
                        .collect_view()
                })}
            </ul>
        </Suspense>

        <div class="mt-4">
            <A href="/liquidity-book-leptos/pool/create">
                <button class="p-1">"Create New Pool"</button>
            </A>
        </div>
    }
}
