use crate::components::SecretQuery;
use crate::utils::shorten_address;
use crate::{
    constants::{CHAIN_ID, GRPC_URL},
    error::Error,
    liquidity_book::constants::addrs::{LB_FACTORY_CONTRACT, LB_PAIR_CONTRACT},
    prelude::TOKEN_MAP,
    state::*,
};
use cosmwasm_std::{Addr, ContractInfo};
use leptos::prelude::*;
use leptos_router::hooks::{query_signal_with_options, use_location, use_navigate};
use leptos_router::NavigateOptions;
use leptos_router::{
    components::A,
    hooks::{use_params, use_params_map, use_query_map},
    nested_router::Outlet,
};
use rsecret::query::compute::ComputeQuerier;
use secretrs::utils::EnigmaUtils;
use send_wrapper::SendWrapper;
use shade_protocol::{
    liquidity_book::{
        lb_factory,
        lb_pair::{
            ActiveIdResponse, BinResponse, LBPairInformation, NextNonEmptyBinResponse,
            TotalSupplyResponse,
        },
    },
    swap::core::TokenType,
};
use tonic_web_wasm_client::Client as WebWasmClient;
use tracing::{debug, info, trace};

#[component]
pub fn PoolManager() -> impl IntoView {
    info!("rendering <PoolManager/>");

    let navigate = use_navigate();
    let location = use_location();

    let endpoint = use_context::<Endpoint>().expect("endpoint context missing!");
    let keplr = use_context::<KeplrSignals>().expect("keplr signals context missing!");
    let token_map = use_context::<TokenMap>().expect("tokens context missing!");

    let params = use_params_map();
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
    let basis_points = move || params.read().get("bps").expect("Missing bps URL param");

    // not sure if possible
    fn token_symbol_converter(address: String) -> AsyncDerived<String> {
        AsyncDerived::new(move || {
            let address = address.clone();
            SendWrapper::new(async move {
                let address = Addr::unchecked(address);
                let code_hash =
                    "0bbaa17a6bd4533f5dc3eae14bfd1152891edaabcc0d767f611bb70437b3a159".to_string();
                let contract = ContractInfo { address, code_hash };
                let name = secret_toolkit_snip20::QueryMsg::TokenInfo {}
                    .do_query(&contract)
                    .await
                    .inspect(|response| trace!("{:?}", response))
                    .and_then(|response| Ok(serde_json::from_str::<serde_json::Value>(&response)?))
                    .map(|x| x.get("token_info").unwrap().to_owned())
                    .map(|x| {
                        serde_json::from_value::<secret_toolkit_snip20::TokenInfo>(x.clone())
                            .unwrap()
                    })
                    .map(|x| x.symbol)
                    .unwrap();

                name
            })
        })
    }

    let token_a_symbol: AsyncDerived<String> = AsyncDerived::new(move || {
        SendWrapper::new(async move {
            let address = Addr::unchecked(token_a());
            let code_hash =
                "0bbaa17a6bd4533f5dc3eae14bfd1152891edaabcc0d767f611bb70437b3a159".to_string();
            let contract = ContractInfo { address, code_hash };
            let symbol = secret_toolkit_snip20::QueryMsg::TokenInfo {}
                .do_query(&contract)
                .await
                .inspect(|response| trace!("{:?}", response))
                .and_then(|response| Ok(serde_json::from_str::<serde_json::Value>(&response)?))
                .map(|x| x.get("token_info").unwrap().to_owned())
                .map(|x| {
                    serde_json::from_value::<secret_toolkit_snip20::TokenInfo>(x.clone()).unwrap()
                })
                .map(|x| x.symbol)
                .unwrap();

            symbol
        })
    });

    let token_b_symbol: AsyncDerived<String> = AsyncDerived::new(move || {
        SendWrapper::new(async move {
            let address = Addr::unchecked(token_b());
            let code_hash =
                "0bbaa17a6bd4533f5dc3eae14bfd1152891edaabcc0d767f611bb70437b3a159".to_string();
            let contract = ContractInfo { address, code_hash };
            let symbol = secret_toolkit_snip20::QueryMsg::TokenInfo {}
                .do_query(&contract)
                .await
                .inspect(|response| trace!("{:?}", response))
                .and_then(|response| Ok(serde_json::from_str::<serde_json::Value>(&response)?))
                .map(|x| x.get("token_info").unwrap().to_owned())
                .map(|x| serde_json::from_value::<secret_toolkit_snip20::TokenInfo>(x).unwrap())
                .map(|x| x.symbol)
                .unwrap();

            symbol
        })
    });

    // whenever the key store changes, this will re-set 'is_keplr_enabled' to true, triggering a
    // reload of everything subscribed to that signal
    let keplr_keystorechange_handle =
        window_event_listener_untyped("keplr_keystorechange", move |_| {
            keplr.enabled.set(true);
        });

    on_cleanup(move || {
        info!("cleaning up <PoolManager/>");
        keplr_keystorechange_handle.remove()
    });

    // let resource = Resource::new(
    //     move || (token_a(), token_b(), basis_points()),
    //     move |(token_a, token_b, basis_points)| {
    //         SendWrapper::new(async move {
    //             let encryption_utils = secretrs::EncryptionUtils::new(None, CHAIN_ID).unwrap();
    //             // TODO: revisit this. url is not needed, EncryptionUtils should be a trait
    //             let options = CreateQuerierOptions {
    //                 url: "https://grpc.mainnet.secretsaturn.net",
    //                 chain_id: CHAIN_ID,
    //                 encryption_utils,
    //             };
    //             let compute = ComputeQuerier::new(wasm_client.get(), options);
    //             // TODO:
    //             let query = format!("{}, {}, {}", token_a, token_b, basis_points);
    //             debug!("{query}");
    //
    //             let result = compute.address_by_label("amber-24").await;
    //             result.map_err(Into::<crate::Error>::into)
    //         })
    //     },
    // );
    // let (pending, set_pending) = signal(false);

    use crate::liquidity_book::contract_interfaces::lb_pair::QueryMsg;
    use crate::liquidity_book::Querier;

    // TODO: change contract_interfaces to not use u128 in response types.

    #[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
    pub struct ReservesResponse {
        pub reserve_x: String,
        pub reserve_y: String,
    }

    #[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
    pub struct BinResponse {
        pub bin_id: u32,
        pub bin_reserve_x: String,
        pub bin_reserve_y: String,
    }

    async fn addr_2_contract(contract_address: impl Into<String>) -> Result<ContractInfo> {
        let contract_address = contract_address.into();
        let client = WebWasmClient::new(GRPC_URL.to_string());
        let encryption_utils = EnigmaUtils::new(None, CHAIN_ID).unwrap();
        let compute = ComputeQuerier::new(client, encryption_utils.into());
        let contract_info = compute
            .code_hash_by_contract_address(contract_address.clone())
            .await
            .map(|code_hash| ContractInfo {
                address: Addr::unchecked(contract_address),
                code_hash,
            })?;

        Ok(contract_info)
    }

    // TODO: Move these to a separate module. IDK if it's worth splitting up the query functions
    // from the resources.

    // TODO: Each response can be either the specific expected response struct, or any of the potential
    // error types within the contract. Figure out how to handle this.

    async fn query_lb_pair_information(
        token_x: ContractInfo,
        token_y: ContractInfo,
        bin_step: u16,
    ) -> Result<lb_factory::LbPairInformationResponse, Error> {
        lb_factory::QueryMsg::GetLbPairInformation {
            token_x: token_x.into(),
            token_y: token_y.into(),
            bin_step,
        }
        .do_query(&LB_FACTORY_CONTRACT)
        .await
        .inspect(|response| trace!("{:?}", response))
        .and_then(|response| {
            Ok(serde_json::from_str::<lb_factory::LbPairInformationResponse>(&response)?)
        })
        // serde_json::from_str::<lb_factory::LbPairInformationResponse>(&response)
        //     .expect("Failed to deserialize LbPairInformationResponse!")
    }

    async fn query_reserves(lb_pair_contract: &ContractInfo) -> Result<ReservesResponse, Error> {
        QueryMsg::GetReserves {}
            .do_query(lb_pair_contract)
            .await
            .inspect(|response| trace!("{:?}", response))
            .and_then(|response| Ok(serde_json::from_str::<ReservesResponse>(&response)?))
        // serde_json::from_str::<ReservesResponse>(&response)
        //     .expect("Failed to deserialize ReservesResponse!")
    }

    async fn query_active_id(lb_pair_contract: &ContractInfo) -> Result<u32, Error> {
        QueryMsg::GetActiveId {}
            .do_query(lb_pair_contract)
            .await
            .inspect(|response| trace!("{:?}", response))
            .and_then(|response| Ok(serde_json::from_str::<ActiveIdResponse>(&response)?))
            .map(|x| x.active_id)
        // .expect("Failed to deserialize ActiveIdResponse!")
    }

    async fn query_bin_reserves(
        lb_pair_contract: &ContractInfo,
        id: u32,
    ) -> Result<BinResponse, Error> {
        QueryMsg::GetBinReserves { id }
            .do_query(lb_pair_contract)
            .await
            .inspect(|response| trace!("{:?}", response))
            .and_then(|response| Ok(serde_json::from_str::<BinResponse>(&response)?))
    }

    async fn query_next_non_empty_bin(
        lb_pair_contract: &ContractInfo,
        id: u32,
    ) -> Result<u32, Error> {
        QueryMsg::GetNextNonEmptyBin {
            swap_for_y: true,
            id,
        }
        .do_query(lb_pair_contract)
        .await
        .inspect(|response| trace!("{:?}", response))
        .and_then(|response| Ok(serde_json::from_str::<NextNonEmptyBinResponse>(&response)?))
        .map(|x| x.next_id)
    }

    // TODO: Figure out why this number is so huge, for example:
    //       12243017097593870802128434755484640756287535340
    async fn query_total_supply(lb_pair_contract: &ContractInfo, id: u32) -> Result<String, Error> {
        QueryMsg::TotalSupply { id }
            .do_query(lb_pair_contract)
            .await
            .inspect(|response| trace!("{:?}", response))
            .and_then(|response| Ok(serde_json::from_str::<TotalSupplyResponse>(&response)?))
            .map(|x| x.total_supply.to_string())
    }

    let lb_pair_information: Resource<LBPairInformation> = Resource::new(
        move || (token_a(), token_b(), basis_points()),
        move |(token_a, token_b, basis_points)| {
            SendWrapper::new(async move {
                // let token_x = addr_2_contract(token_a).await.unwrap();
                // let token_y = addr_2_contract(token_b).await.unwrap();
                let token_x = ContractInfo {
                    address: Addr::unchecked(token_a),
                    code_hash: "0bbaa17a6bd4533f5dc3eae14bfd1152891edaabcc0d767f611bb70437b3a159"
                        .to_string(),
                };
                let token_y = ContractInfo {
                    address: Addr::unchecked(token_b),
                    code_hash: "0bbaa17a6bd4533f5dc3eae14bfd1152891edaabcc0d767f611bb70437b3a159"
                        .to_string(),
                };
                let bin_step = basis_points.parse::<u16>().unwrap();
                query_lb_pair_information(token_x, token_y, bin_step)
                    .await
                    .map(|x| x.lb_pair_information)
                    .unwrap()
            })
        },
    );

    // Potentially nicer, but not necessary.
    // let lb_pair_contract = AsyncDerived::new(move || {
    //     SendWrapper::new(async move { lb_pair_information.await.lb_pair.contract })
    // });

    let total_reserves = Resource::new(
        move || lb_pair_information.track(),
        move |_| {
            SendWrapper::new(async move {
                let lb_pair_contract = lb_pair_information.await.lb_pair.contract;
                query_reserves(&lb_pair_contract).await
            })
        },
    );

    // prevents scrolling to the top of the page each time a query param changes
    let nav_options = NavigateOptions {
        scroll: false,
        ..Default::default()
    };
    let (_, set_active_id) = query_signal_with_options::<String>("active_id", nav_options.clone());

    let active_id = Resource::new(
        move || lb_pair_information.track(),
        move |_| {
            SendWrapper::new(async move {
                let lb_pair_contract = lb_pair_information.await.lb_pair.contract;
                query_active_id(&lb_pair_contract)
                    .await
                    .inspect(|id| set_active_id.set(Some(id.to_string())))
            })
        },
    );

    let bin_reserves = Resource::new(
        move || (lb_pair_information.track(), active_id.track()),
        move |_| {
            SendWrapper::new(async move {
                let lb_pair_contract = lb_pair_information.await.lb_pair.contract;
                let id = active_id.await?;
                query_bin_reserves(&lb_pair_contract, id).await
            })
        },
    );

    let next_non_empty_bin = Resource::new(
        move || (lb_pair_information.track(), active_id.track()),
        move |_| {
            SendWrapper::new(async move {
                let lb_pair_contract = lb_pair_information.await.lb_pair.contract;
                let id = active_id.await?;
                query_next_non_empty_bin(&lb_pair_contract, id).await
            })
        },
    );
    let bin_total_supply = Resource::new(
        move || (lb_pair_information.track(), active_id.track()),
        move |_| {
            SendWrapper::new(async move {
                let lb_pair_contract = lb_pair_information.await.lb_pair.contract;
                let id = active_id.await?;
                query_total_supply(&lb_pair_contract, id)
                    .await
                    .map(|x| format!("{x:?}"))
            })
        },
    );

    view! {
        <a
            href="/trader-crow-leptos/pool"
            class="block text-neutral-200/50 text-sm font-bold cursor-pointer no-underline"
        >
            "🡨 Back to pools list"
        </a>
        <div class="flex flex-wrap py-2 items-center gap-x-4 gap-y-2">

            <Suspense fallback=move || {
                view! { <div class="text-3xl font-bold">{token_a}" / "{token_b}</div> }
            }>
                <div class="text-3xl font-bold">
                    {move || Suspend::new(async move { token_a_symbol.await })}" / "
                    {move || Suspend::new(async move { token_b_symbol.await })}
                </div>
            </Suspense>

            <div class="flex items-center gap-x-4">
                <div class="text-md font-bold p-1 outline outline-1 outline-offset-2 outline-neutral-500/50">
                    {basis_points}" bps"
                </div>
                <a href="about:blank" target="_blank" rel="noopener">
                    <div class="text-md font-bold p-1 outline outline-1 outline-offset-2 outline-neutral-500/50">
                        {move || {
                            lb_pair_information
                                .get()
                                .map(|x| shorten_address(x.lb_pair.contract.address))
                        }}" ↗"
                    </div>
                </a>
            </div>
        </div>

        // <Suspense fallback=|| view! { <p>"Loading..."</p> }>
        // // you can `.await` resources to avoid dealing with the `None` state
        // <p>
        // "User ID: "
        // {move || Suspend::new(async move {
        // match resource.await {
        // Ok(response) => response,
        // Err(_) => "error".to_string(),
        // }
        // })}
        // </p>
        // or you can still use .get() to access resources in things like component props
        // <For
        // each=move || resource.get().and_then(Result::ok).unwrap_or_default()
        // key=|resource| resource.id
        // let:post
        // >
        // // ...
        // </For>
        // </Suspense>

        <details class="text-neutral-300 font-bold">
            <summary class="text-lg cursor-pointer">Pool Details</summary>
            <ul class="my-1 font-normal text-base text-neutral-200 ">
                <Suspense fallback=|| view! { <div>"Loading Total Reserves..."</div> }>
                    <li>
                        "Total Reserves: "<span tabindex="0" class="cursor-pointer text-white peer">
                            "🛈"
                        </span>
                        <li class="list-none text-sm font-bold text-violet-400 peer-focus:block hidden">
                            "🛈 Reserves may be in reverse order"
                        </li>
                        {move || Suspend::new(async move {
                            let reserves = total_reserves.await.unwrap();
                            view! {
                                <li class="pl-4 list-none">"reserve_x: "{reserves.reserve_x}</li>
                                <li class="pl-4 list-none">"reserve_y: "{reserves.reserve_y}</li>
                            }
                        })}
                    </li>
                </Suspense>
                <Suspense fallback=|| view! { <div>"Loading Active ID..."</div> }>
                    <li>
                        "Active Bin ID: " {move || Suspend::new(async move { active_id.await })}
                    </li>
                </Suspense>
                <Suspense fallback=|| view! { <div>"Loading Bin Reserves..."</div> }>
                    <li>
                        "Active Bin Reserves: "
                        {move || Suspend::new(async move {
                            let reserves = bin_reserves.await.unwrap();
                            view! {
                                <li class="pl-4 list-none">
                                    "bin_reserve_x: "{reserves.bin_reserve_x}
                                </li>
                                <li class="pl-4 list-none">
                                    "bin_reserve_y: "{reserves.bin_reserve_y}
                                </li>
                            }
                        })}
                    </li>
                </Suspense>
                <Suspense fallback=|| view! { <div>"Loading Next Non-Empty Bin..."</div> }>
                    <li>
                        "Next Non-Empty Bin ID: "
                        {move || Suspend::new(async move { next_non_empty_bin.await })}
                    </li>
                </Suspense>
                <SecretQuery query=bin_total_supply />
            </ul>
        </details>

        <div class="flex gap-4 py-2">
            // This works to preserve the query params when navigating to nested routes.
            <button on:click=move |_| {
                let pathname = location.pathname.get();
                let query_params = location.search.get();
                let new_route = format!("{pathname}/add/?{query_params}");
                navigate(&new_route, Default::default())
            }>"Add Liquidity"</button>

            <A href="remove">
                <button>"Remove Liquidity"</button>
            </A>

            <A href="">
                <button>Nevermind</button>
            </A>
        </div>

        <Outlet />
    }
}
