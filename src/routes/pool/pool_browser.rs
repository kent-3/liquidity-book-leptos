use crate::{
    constants::Querier,
    liquidity_book::{
        constants::addrs::{LB_CONTRACTS, LB_PAIR},
        contract_interfaces::{lb_factory, lb_pair::LbPair},
    },
    routes::pool::LB_FACTORY,
};
use cosmwasm_std::ContractInfo;
use leptos::prelude::*;
use leptos_router::components::A;
use send_wrapper::SendWrapper;
use shade_protocol::swap::core::TokenType;
use tracing::{debug, info, trace};

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

    // TODO: query for the pools
    let pools = vec![pair_1];

    let number_of_lb_pairs: Resource<u32> = Resource::new(
        move || (),
        move |_| {
            SendWrapper::new(async move {
                lb_factory::QueryMsg::GetNumberOfLbPairs {}
                    .do_query(&LB_FACTORY)
                    .await
                    .inspect(|response| debug!("{:?}", response))
                    .and_then(|response| {
                        Ok(serde_json::from_str::<lb_factory::NumberOfLbPairsResponse>(
                            &response,
                        )?)
                    })
                    .map(|x| x.lb_pair_number)
                    .unwrap()
            })
        },
    );

    // TODO: this is super inefficient. but I don't see a better way...
    let all_lb_pairs: Resource<Vec<LbPair>> = Resource::new(
        move || (),
        move |_| {
            SendWrapper::new(async move {
                let i = number_of_lb_pairs.await;
                let mut pairs: Vec<LbPair> = Vec::with_capacity(i as usize);

                for index in 0..i {
                    pairs.push(
                        lb_factory::QueryMsg::GetLbPairAtIndex { index }
                            .do_query(&LB_FACTORY)
                            .await
                            .inspect(|response| debug!("{:?}", response))
                            .and_then(|response| {
                                Ok(serde_json::from_str::<lb_factory::LbPairAtIndexResponse>(
                                    &response,
                                )?)
                            })
                            .map(|x| x.lb_pair)
                            .unwrap(),
                    )
                }

                pairs
            })
        },
    );

    view! {
        <div class="text-3xl font-bold">"Pool"</div>
        <div class="text-sm text-neutral-400">"Provide liquidity and earn fees."</div>

        <h3 class="mb-1">"Existing Pools - " {number_of_lb_pairs}</h3>
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
