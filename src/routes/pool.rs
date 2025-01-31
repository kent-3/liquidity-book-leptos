mod pool_analytics;
mod pool_browser;
mod pool_creator;
mod pool_manager;

pub use pool_analytics::PoolAnalytics;
pub use pool_browser::PoolBrowser;
pub use pool_creator::PoolCreator;
pub use pool_manager::{AddLiquidity, PoolManager, RemoveLiquidity};

use leptos::prelude::*;
use leptos_router::nested_router::Outlet;
use tracing::info;

#[component]
pub fn Pools() -> impl IntoView {
    info!("rendering <Pools/>");

    on_cleanup(move || {
        info!("cleaning up <Pools/>");
    });

    // Resources in this component can be shared with all children, so they only run once and are
    // persistent. This is just an example:
    // let resource = LocalResource::new(move || {
    //     SendWrapper::new(async move {
    //         QueryMsg::GetNumberOfLbPairs {}
    //             .do_query(&LB_FACTORY)
    //             .await
    //     })
    // });

    // provide_context(resource);

    view! {
        <div class="pools-group">
            <Outlet />
        </div>
    }
}

// NOTE: If the Router gets complicated enough, it's possible to split it up like this:

// use leptos_router::{
//     components::{ParentRoute, Route},
//     MatchNestedRoutes,
// };
// use leptos_router_macro::path;
//
// #[component]
// pub fn PoolRoutes() -> impl MatchNestedRoutes<Dom> + Clone {
//     view! {
//         <ParentRoute path=path!("/pool") view=Pool>
//             <Route path=path!("/") view=PoolBrowser />
//             <Route path=path!("/create") view=PoolCreator />
//             <ParentRoute path=path!("/:token_a/:token_b/:bps") view=PoolManager>
//                 <Route path=path!("/") view=|| () />
//                 <Route path=path!("/add") view=AddLiquidity />
//                 <Route path=path!("/remove") view=RemoveLiquidity />
//             </ParentRoute>
//         </ParentRoute>
//     }
//     .into_inner()
// }

#[component]
pub fn Pool() -> impl IntoView {
    info!("rendering <Pool/>");

    use crate::{
        error::Error,
        prelude::*,
        support::{chain_query, ILbPair, Querier, COMPUTE_QUERIER},
    };
    use ammber_sdk::contract_interfaces::lb_pair::{BinResponse, LbPair};
    use batch_query::{
        msg_batch_query, parse_batch_query, BatchItemResponseStatus, BatchQueryParams,
        BatchQueryParsedResponse, BatchQueryResponse, BATCH_QUERY_ROUTER,
    };
    use cosmwasm_std::{Addr, ContractInfo};
    use leptos::prelude::*;
    use leptos_router::{components::A, hooks::use_params_map, nested_router::Outlet};
    use lucide_leptos::{ArrowLeft, ExternalLink};
    use secret_toolkit_snip20::TokenInfoResponse;
    use send_wrapper::SendWrapper;
    use serde::Serialize;
    use tracing::{debug, info, trace};

    // TODO: I should provide a context here with all the pool information. that way child
    // components like the Add/Remove liquidity ones can access it. I don't think putting the
    // active_id as a query param in the url is a good idea (it should be updated frequently).

    let params = use_params_map();
    // TODO: decide on calling these a/b or x/y
    let token_a = move || {
        params
            .read()
            .get("token_a")
            .expect("Missing token_a URL param")
    };
    let token_b = move || {
        params
            .read()
            .get("token_b")
            .expect("Missing token_b URL param")
    };
    let basis_points = move || {
        params
            .read()
            .get("bps")
            .and_then(|value| value.parse::<u16>().ok())
            .expect("Missing bps URL param")
    };

    // TODO: these 2 functions feel very convoluted

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

    provide_context((token_a_symbol, token_b_symbol));

    // TODO: how about instead, we have a contract query that can return the nearby liquidity, so
    // we don't have to mess with the complicated batch query router? That might be the purpose of
    // the LiquidityHelper contract (I haven't looked at it yet)

    async fn query_nearby_bins<T: Serialize>(
        queries: Vec<BatchQueryParams<T>>,
    ) -> Result<Vec<BinResponse>, Error> {
        msg_batch_query(queries)
            .do_query(&BATCH_QUERY_ROUTER.pulsar)
            .await
            .inspect(|response| trace!("{:?}", response))
            .and_then(|response| Ok(serde_json::from_str::<BatchQueryResponse>(&response)?))
            .map(parse_batch_query)
            .map(extract_bins_from_batch_response)
    }

    fn extract_bins_from_batch_response(
        batch_response: BatchQueryParsedResponse,
    ) -> Vec<BinResponse> {
        batch_response
            .items
            .into_iter()
            .filter(|item| item.status == BatchItemResponseStatus::SUCCESS)
            .map(|item| {
                serde_json::from_str::<BinResponse>(&item.response)
                    .expect("Invalid BinResponse JSON")
            })
            .collect()
    }

    // SendWrapper required due to addr_2_contract function
    let lb_pair: Resource<Result<LbPair, Error>> = Resource::new(
        move || (token_a(), token_b(), basis_points()),
        |(token_a, token_b, basis_points)| {
            debug!("run lb_pair resource");
            SendWrapper::new(async move {
                let token_x = addr_2_contract(token_a).await.unwrap();
                let token_y = addr_2_contract(token_b).await.unwrap();
                let bin_step = basis_points;

                let storage = window()
                    .local_storage()
                    .expect("local storage not available?")
                    .expect("local storage returned none?");

                let storage_key = format!("{}_{}_{}", token_x.address, token_y.address, bin_step);

                match storage.get_item(&storage_key) {
                    Ok(None) => {
                        let lb_pair = LB_FACTORY
                            .get_lb_pair_information(token_x, token_y, bin_step)
                            .await
                            .map(|lb_pair_information| lb_pair_information.lb_pair)
                            .inspect(|lb_pair| {
                                _ = storage.set_item(
                                    &storage_key,
                                    &serde_json::to_string(&lb_pair).unwrap(),
                                )
                            });

                        // let _ = storage
                        //     .set_item(&storage_key, &serde_json::to_string(&lb_pair).unwrap());

                        lb_pair
                    }
                    Ok(Some(lb_pair)) => Ok(serde_json::from_str(&lb_pair).unwrap()),
                    _ => todo!(),
                }
            })
        },
    );

    // NOTE: For some reason, this always runs twice. To avoid double querying, do nothing if
    //       lb_pair is None, instead of await-ing it. I wonder if it has something to do with all
    //       the SendWrappers involved...
    let active_id = Resource::new(
        move || lb_pair.get(),
        |lb_pair| {
            debug!("run active_id resource");
            async move {
                if let Some(Ok(lb_pair)) = lb_pair {
                    ILbPair(lb_pair.contract).get_active_id().await
                } else {
                    Err(Error::generic("lb_pair resource is not available yet"))
                }
            }
        },
    );

    provide_context(active_id);
    provide_context(lb_pair);

    // TODO: decide if these queries should go here or in the analytics component
    let total_reserves = Resource::new(
        move || lb_pair.get(),
        move |_| async move { ILbPair(lb_pair.await?.contract).get_reserves().await },
    );
    let static_fee_parameters = Resource::new(
        move || lb_pair.get(),
        move |_| async move {
            ILbPair(lb_pair.await?.contract)
                .get_static_fee_parameters()
                .await
        },
    );
    provide_context(total_reserves);
    provide_context(static_fee_parameters);

    view! {
        <a
            href="/liquidity-book-leptos/pool"
            class="inline-flex gap-x-2 items-center text-neutral-500 text-sm font-bold cursor-pointer no-underline"
        >
            <ArrowLeft size=14 />
            "Back to pools list"
        </a>

        // page title with the token symbols
        <div class="flex flex-col md:flex-row py-2 items-start md:items-center gap-x-4 gap-y-2">
            <Suspense fallback=move || {
                view! { <div class="text-3xl font-bold">{token_a}" / "{token_b}</div> }
            }>
                // TODO: add token icons here
                <div class="text-3xl font-bold">
                    {move || Suspend::new(async move { token_a_symbol.await })}" / "
                    {move || Suspend::new(async move { token_b_symbol.await })}
                </div>
            </Suspense>

            <div class="flex items-center gap-x-2 md:pl-4">
                <span class="text-sm text-white inline-flex font-bold px-2 py-1 rounded-full border border-solid border-neutral-700">
                    {basis_points}" bps"
                </span>
                <span class="inline-flex px-2 py-1 rounded-full border border-solid border-neutral-700">
                    <a
                        href="about:blank"
                        target="_blank"
                        rel="noopener"
                        class="no-underline text-white text-sm font-bold"
                    >
                        <div class="flex gap-1 items-center">
                            <div>
                                {move || {
                                    lb_pair
                                        .get()
                                        .and_then(Result::ok)
                                        .map(|x| shorten_address(x.contract.address))
                                }}
                            </div>
                            <ExternalLink size=14 color="white" />
                        </div>
                    </a>
                </span>
            </div>
        </div>

        <div class="flex gap-4 items-center mt-2 mb-6">
            <A href="manage">
                <button class="px-4">"Manage"</button>
            </A>
            <A href="analytics">
                <button class="px-4">"Analytics"</button>
            </A>
        </div>

        <div class="pool-tab-group">
            <Outlet />
        </div>
    }
}
