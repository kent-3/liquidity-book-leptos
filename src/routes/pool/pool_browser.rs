use crate::prelude::*;
use crate::Error;
use ammber_sdk::contract_interfaces::lb_pair::LbPair;
use leptos::prelude::*;
use leptos_router::components::A;
use liquidity_book::core::TokenType;
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
    let all_lb_pairs = use_context::<LocalResource<Vec<LbPair>>>()
        .expect("missing the Vec<LbPair> resource context");

    // TODO: experiment with leptos_struct_table
    // use leptos_struct_table::*;

    // #[derive(TableRow, Clone, Default, Debug)]
    // #[table(impl_vec_data_provider)]
    // struct Pool {
    //     pub pool_name: String,
    //     pub liquidity: u32,
    //     pub depth_x: u32,
    //     pub depth_y: u32,
    //     pub fees: u32,
    // }
    //
    // let rows = all_lb_pairs
    //     .get()
    //     .map(|vec| {
    //         vec.iter()
    //             .map(|n| Pool {
    //                 pool_name: n.contract.address.to_string(),
    //                 ..Default::default()
    //             })
    //             .collect::<Vec<Pool>>()
    //     })
    //     .unwrap_or_default();

    // Effect::new(move || debug!("{:?}", rows));

    view! {
        <div class="text-3xl font-bold">"Pool"</div>
        <p class="text-sm text-zinc-400">"Provide liquidity and earn fees."</p>

        // <table>
        // <TableContent rows=rows scroll_container="html"/>
        // </table>

        <div class="flex items-center justify-between">
            <h3 class="mb-3">
                "All Pools - " {move || number_of_lb_pairs.get().as_deref().cloned()}
            </h3>

            <div class="">
                <A href="/liquidity-book-leptos/pool/create">
                    <button class="min-w-24 inline-flex justify-center items-center
                    font-medium leading-none py-1.5 px-2
                    border border-solid border-zinc-500 bg-zinc-500 text-white rounded-xs">
                        "Create New Pool"
                    </button>
                </A>
            </div>
        </div>

        <div class="flex flex-col gap-2 md:hidden">
            <Suspense fallback=|| {
                view! { <div>"Loading..."</div> }
            }>
                {move || Suspend::new(async move {
                    all_lb_pairs
                        .await
                        .into_iter()
                        .map(|n| {
                            view! {
                                <div class="block bg-zinc-800 rounded-lg space-y-4 border border-solid border-zinc-700 p-4">
                                    <div class="flex items-center gap-4 text-base font-semibold">
                                        <a
                                            class="no-underline text-white"
                                            href=format!(
                                                "/liquidity-book-leptos/pool/{}/{}/{}/manage",
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
                                            )
                                        >
                                            <div class="">
                                                {format!(
                                                    "{} – {}",
                                                    TOKEN_MAP
                                                        .get(&n.token_x.unique_key())
                                                        .map(|t| t.symbol.clone())
                                                        .unwrap_or_default(),
                                                    TOKEN_MAP
                                                        .get(&n.token_y.unique_key())
                                                        .map(|t| t.symbol.clone())
                                                        .unwrap_or_default(),
                                                )}
                                            </div>
                                        </a>
                                        <div class="text-white text-xs py-1 px-2 rounded-full bg-zinc-700">
                                            {format!("{} bps", n.bin_step)}
                                        </div>

                                    </div>
                                    // TODO: how would I get this data while inside of the iterator?
                                    <div class="flex flex-row justify-between text-sm">
                                        <div class="flex flex-col">
                                            <p class="mb-1 mt-0 text-zinc-400">"Liquidity"</p>
                                            <p class="my-0 font-semibold">"$0.00"</p>
                                        </div>
                                        <div class="flex flex-col">
                                            <p class="mb-1 mt-0 text-zinc-400">"Volume (24H)"</p>
                                            <p class="my-0 font-semibold">"$0.00"</p>
                                        </div>
                                        <div class="flex flex-col">
                                            <p class="mb-1 mt-0 text-zinc-400">"Fees (24H)"</p>
                                            <p class="my-0 font-semibold">"$0.00"</p>
                                        </div>
                                    </div>
                                </div>
                            }
                        })
                        .collect_view()
                })}
            </Suspense>
        </div>

        <div class="hidden md:block box-border p-2 min-w-full border border-solid border-zinc-700 rounded-lg bg-zinc-800">
            <table class="min-w-full -my-2 leading-tight border-separate border-spacing-x-0 border-spacing-y-2">
                <thead class="box-border border-0 border-solid border-spacing-x-0 border-spacing-y-2">
                    <tr class="box-content bg-zinc-700">
                        <th class="px-4 py-2 text-left rounded-l-sm box-content">"Pool Name"</th>
                        <th class="px-4 py-2 text-right box-content">"Volume (24H)"</th>
                        <th class="px-4 py-2 text-right box-content">"Liquidity"</th>
                        <th class="px-4 py-2 text-right rounded-r-sm box-content">"Fees (24H)"</th>
                    </tr>
                </thead>
                // crazy, but it works
                <Suspense fallback=|| view! { <div>"Loading..."</div> }>
                    <tbody>
                        {move || Suspend::new(async move {
                            all_lb_pairs
                                .await
                                .into_iter()
                                .map(|n| {
                                    view! {
                                        <tr>
                                            <td class="px-4 py-2">
                                                <div class="flex items-center gap-4 text-sm font-semibold">
                                                    <a
                                                        class="no-underline text-white"
                                                        href=format!(
                                                            "/liquidity-book-leptos/pool/{}/{}/{}/manage",
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
                                                        )
                                                    >
                                                        <div class="">
                                                            {format!(
                                                                "{} – {}",
                                                                TOKEN_MAP
                                                                    .get(&n.token_x.unique_key())
                                                                    .map(|t| t.symbol.clone())
                                                                    .unwrap_or_default(),
                                                                TOKEN_MAP
                                                                    .get(&n.token_y.unique_key())
                                                                    .map(|t| t.symbol.clone())
                                                                    .unwrap_or_default(),
                                                            )}
                                                        </div>
                                                    </a>
                                                    <div class="text-white text-xs py-1 px-2 rounded-full bg-zinc-700">
                                                        {format!("{} bps", n.bin_step)}
                                                    </div>

                                                </div>
                                            </td>
                                            // TODO: how would I get this data while inside of the iterator?
                                            <td class="px-4 py-2 text-right">"$0.00"</td>
                                            <td class="px-4 py-2 text-right">"$0.00"</td>
                                            <td class="px-4 py-2 text-right">"$0.00"</td>
                                        </tr>
                                    }
                                })
                                .collect_view()
                        })}
                    </tbody>
                </Suspense>

            </table>
        </div>
    }
}
