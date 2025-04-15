use crate::{SwapDetails, SwapSettings};
use ammber_components::{LoadingModal, Secret20Balance, Spinner2};
use ammber_core::{
    constants::{contracts::*, CHAIN_ID, NODE, SYMBOL_TO_ADDR, TOKEN_MAP},
    state::{Endpoint, KeplrSignals, TokenMap},
    utils::{display_token_amount, parse_token_amount},
    Error,
};
use ammber_sdk::contract_interfaces::{
    lb_quoter::Quote,
    lb_router::{self, Path},
};
use codee::string::FromToStringCodec;
use cosmwasm_std::{to_binary, Addr, Uint128, Uint64};
use keplr::Keplr;
use leptos::{ev, html, logging::*, prelude::*, tachys::dom::window};
use leptos_router::{hooks::query_signal_with_options, NavigateOptions};
use leptos_use::storage::use_local_storage;
use liquidity_book::core::TokenType;
use lucide_leptos::{ArrowDownUp, Settings2};
use rsecret::{
    secret_client::CreateTxSenderOptions,
    tx::{compute::MsgExecuteContractRaw, ComputeServiceClient},
    TxOptions,
};
use secretrs::AccountId;
use std::str::FromStr;
use tracing::{debug, info};
use web_sys::js_sys::Date;

#[component]
pub fn Swap() -> impl IntoView {
    info!("rendering <Swap/>");

    on_cleanup(move || {
        info!("cleaning up <Swap/>");
    });

    let endpoint = use_context::<Endpoint>().expect("endpoint context missing!");
    let keplr = use_context::<KeplrSignals>().expect("keplr signals context missing!");
    let _token_map = use_context::<TokenMap>().expect("tokens context missing!");

    // prevents scrolling to the top of the page each time a query param changes
    let nav_options = NavigateOptions {
        scroll: false,
        ..Default::default()
    };

    let (token_x, set_token_x) = query_signal_with_options::<String>("from", nav_options.clone());
    let (token_y, set_token_y) = query_signal_with_options::<String>("to", nav_options.clone());

    let token_x_info =
        Signal::derive(move || token_x.get().and_then(|ref address| TOKEN_MAP.get(address)));
    let token_y_info =
        Signal::derive(move || token_y.get().and_then(|ref address| TOKEN_MAP.get(address)));

    let (amount_x, set_amount_x) = signal(String::default());
    let (amount_y, set_amount_y) = signal(String::default());
    let (swap_for_y, set_swap_for_y) = signal(true);

    // slippage is in basis points. smallest supported slippage = 0.01%
    let (slippage, set_slippage, _) = use_local_storage::<u16, FromToStringCodec>("swap_slippage");
    let (deadline, set_deadline, _) = use_local_storage::<u64, FromToStringCodec>("swap_deadline");

    if slippage.get() == 0 {
        set_slippage.set(50u16);
    }
    if deadline.get() == 0 {
        set_deadline.set(5u64);
    }

    // TODO: come up with cool keyboard shortcuts
    // let handle = window_event_listener(ev::keypress, |ev| {
    //     // ev is typed as KeyboardEvent automatically,
    //     // so .code() can be called
    //     let code = ev.code();
    //     log!("code = {code:?}");
    // });
    // on_cleanup(move || handle.remove());

    // TODO: all this settings stuff can probably go in the setting component itself?

    let settings_dialog_ref = NodeRef::<html::Dialog>::new();

    let handle = window_event_listener(ev::keydown, move |ev| {
        if let Some(dialog) = settings_dialog_ref.get() {
            if ev.key() == "Escape" {
                dialog.close();
            }
        }
    });

    on_cleanup(move || handle.remove());

    let toggle_swap_settings = move |_: ev::MouseEvent| match settings_dialog_ref.get() {
        Some(dialog) => match dialog.open() {
            false => {
                _ = dialog.show();
            }
            true => {
                dialog.close();
            }
        },
        None => {
            _ = window().alert_with_message("Something is wrong!");
        }
    };

    // --

    let select_x_node_ref = NodeRef::<html::Select>::new();
    let select_y_node_ref = NodeRef::<html::Select>::new();

    Effect::new(move || {
        if let Some(token_x) = token_x.get() {
            if let Some(select_x) = select_x_node_ref.get() {
                select_x.set_value(&token_x)
            }
        }
    });
    Effect::new(move || {
        if let Some(token_y) = token_y.get() {
            if let Some(select_y) = select_y_node_ref.get() {
                select_y.set_value(&token_y)
            }
        }
    });

    // TODO: handle the tokens_for_exact_tokens scenario. possibly use a separate Action
    let get_quote: Action<(String, String, String), Result<Quote, Error>> = Action::new(
        move |(token_x, token_y, amount_in): &(String, String, String)| {
            let token_x = token_x.to_owned();
            let token_y = token_y.to_owned();
            let amount_in = amount_in.to_owned();

            async move {
                let Some(token_x) = TOKEN_MAP.get(&token_x) else {
                    return Err(Error::generic("No token X selected!"));
                };
                let Some(token_y) = TOKEN_MAP.get(&token_y) else {
                    return Err(Error::generic("No token Y selected!"));
                };

                let amount_in = parse_token_amount(amount_in, token_x.decimals);

                // let token_x_code_hash = TOKEN_MAP
                //     .get(&token_x_address)
                //     .map(|t| t.code_hash.clone())
                //     .ok_or(Error::UnknownToken)
                //     .inspect_err(|error| error!("{:?}", error))?;
                // let token_y_code_hash = TOKEN_MAP
                //     .get(&token_y_address)
                //     .map(|t| t.code_hash.clone())
                //     .ok_or(Error::UnknownToken)
                //     .inspect_err(|error| error!("{:?}", error))?;

                let token_x = TokenType::CustomToken {
                    contract_addr: Addr::unchecked(token_x.contract_address.to_owned()),
                    token_code_hash: token_x.code_hash.to_owned(),
                };
                let token_y = TokenType::CustomToken {
                    contract_addr: Addr::unchecked(token_y.contract_address.to_owned()),
                    token_code_hash: token_y.code_hash.to_owned(),
                };

                let route = vec![token_x, token_y];
                let amount_in = Uint128::from(amount_in);

                LB_QUOTER
                    .find_best_path_from_amount_in(route, amount_in)
                    .await
            }
        },
    );

    let handle_quote = move |_| {
        let (Some(token_x), Some(token_y)) = (token_x.get(), token_y.get()) else {
            return;
        };
        _ = get_quote.dispatch((token_x, token_y, amount_x.get()))
    };

    // Updates the amount_y input whenever the quote changes
    Effect::new(move || {
        if let Some(Ok(quote)) = get_quote.value().get() {
            if let Some(amount_out) = quote.amounts.last() {
                if let Some(token_info) = token_y_info.get() {
                    let amount = display_token_amount(amount_out.u128(), token_info.decimals);
                    set_amount_y.set(amount);
                }
            }
        }
    });

    let _current_quote = move || {
        get_quote
            .value()
            .get()
            .and_then(Result::ok)
            .and_then(|quote| serde_json::to_string_pretty(&quote).ok())
    };

    let _path = move || {
        get_quote
            .value()
            .get()
            .and_then(Result::ok)
            .map(|quote| Path {
                pair_bin_steps: quote.bin_steps,
                versions: quote.versions,
                token_path: quote.route,
            })
    };

    // TODO: how will we recheck the balances after a swap?
    let swap = Action::new_local(move |quote: &Quote| {
        let url = endpoint.get();
        let chain_id = CHAIN_ID;

        let quote = quote.clone();

        async move {
            let Ok(key) = Keplr::get_key(CHAIN_ID).await else {
                return Err(Error::generic("Could not get key from Keplr"));
            };

            let slippage = 10_000 - slippage.get();

            let amount_in = quote
                .amounts
                .first()
                .cloned()
                .expect("quote is missing amount!");
            let amount_out_min = quote
                .amounts
                .last()
                .expect("quote is missing amount!")
                .multiply_ratio(slippage, 10_000u16);
            let path = Path {
                pair_bin_steps: quote.bin_steps,
                versions: quote.versions,
                token_path: quote.route,
            };
            let to = key.bech32_address.clone();
            let deadline = deadline.get() * 60 + (Date::now() / 1000.0) as u64;

            // let wallet = Keplr::get_offline_signer_only_amino(CHAIN_ID);
            let wallet = Keplr::get_offline_signer(chain_id);
            let enigma_utils = Keplr::get_enigma_utils(chain_id).into();

            let options = CreateTxSenderOptions {
                url: NODE,
                chain_id: CHAIN_ID,
                wallet: wallet.into(),
                wallet_address: key.bech32_address.clone().into(),
                enigma_utils,
            };

            // TODO: this isn't making sense... why am I providing a url both here and in the options?
            let wasm_web_client = tonic_web_wasm_client::Client::new(url.to_string());
            let compute_service_client = ComputeServiceClient::new(wasm_web_client, options);

            let swap_msg = lb_router::ExecuteMsg::SwapExactTokensForTokens {
                amount_in: Uint128::from(amount_in),
                amount_out_min: Uint128::from(amount_out_min),
                path: path.clone(),
                to: to.clone(),
                deadline: Uint64::from(deadline),
            };

            debug!("{swap_msg:#?}");

            let send_msg = secret_toolkit_snip20::HandleMsg::Send {
                recipient: LB_ROUTER.address.to_string(),
                recipient_code_hash: Some(LB_ROUTER.code_hash.clone()),
                amount: Uint128::from(amount_in),
                msg: Some(to_binary(&swap_msg)?),
                memo: None,
                padding: None,
            };

            let sender = AccountId::new("secret", &key.address)?;
            let contract = AccountId::from_str(path.token_path[0].address().as_str())?;

            let msg = MsgExecuteContractRaw {
                sender,
                contract: contract.clone(),
                msg: send_msg,
                sent_funds: vec![],
            };

            let tx_options = TxOptions {
                gas_limit: 500_000,
                ..Default::default()
            };

            let tx = compute_service_client
                .execute_contract(msg, path.token_path[0].code_hash(), tx_options)
                .await
                .inspect(|tx_response| info!("{tx_response:?}"))
                .inspect_err(|error| error!("{error}"))?;

            if tx.code != 0 {
                error!("{}", tx.raw_log);
            }

            // TODO: trigger rechecking of balances here somehow

            Ok(())
        }
    });

    let handle_swap = move |_| {
        _ = swap.dispatch(
            get_quote
                .value()
                .get()
                .and_then(Result::ok)
                .expect("you need to get a quote first!"),
        );
    };

    // returns the final amount (the output token)
    let amount_out = Signal::derive(move || {
        get_quote
            .value()
            .get()
            .and_then(Result::ok)
            .and_then(|quote| quote.amounts.last().cloned())
        // .map(|amount| {
        //     display_token_amount(
        //         amount,
        //         token_y_info().map(|info| info.decimals).unwrap_or(0u8),
        //     )
        // })
    });

    // returns the minimum amount out, adjusted for slippage
    let amount_out_min = Signal::derive(move || {
        get_quote
            .value()
            .get()
            .and_then(Result::ok)
            .and_then(|quote| quote.amounts.last().cloned())
            .map(|amount_out| amount_out.multiply_ratio(10_000 - slippage.get(), 10_000u16))
        // .map(|amount| {
        //     display_token_amount(
        //         amount,
        //         token_y_info().map(|info| info.decimals).unwrap_or(0u8),
        //     )
        // })
    });

    // TODO: get the active price, so we can calculate price impact

    // returns the minimum amount out, adjusted for slippage
    let swap_price_ratio = Signal::derive(move || {
        get_quote
            .value()
            .get()
            .and_then(Result::ok)
            .and_then(|quote| {
                let input_token = quote.amounts.first().cloned();
                let output_token = quote.amounts.last().cloned();

                debug!("{:?}", input_token);
                debug!("{:?}", output_token);

                input_token
                    .zip(output_token)
                    .map(|(input, output)| output.u128() as f64 / input.u128() as f64)
                    .inspect(|x| debug!("{:?}", x))
            })
    });

    // let expected_output = RwSignal::new("2.86545 USDC".to_string());
    // let minimum_received = RwSignal::new("2.85112 USDC".to_string());

    // TODO: figure out how to calculate this and enforce 2 decimal places
    let price_impact = RwSignal::new(2.00);
    // absolute inset-0 m-auto -translate-y-[54px]
    view! {
        <LoadingModal when=swap.pending() message="Processing Transaction... (watch the console)" />
        <div class="absolute inset-0 m-auto flex items-center justify-center">
            // <div class="grid gap-4 sm:grid-cols-[minmax(0px,7fr)_minmax(0px,5fr)] grid-cols-1 grid-rows-2 sm:grid-rows-1">
            // <div class="grid gap-4 grid-cols-1 max-w-[550px] w-full">
            <div class="grid gap-4 grid-cols-1 max-w-sm md:-translate-y-[54px]">
                <div class="flex flex-col space-y-3">
                    // buttons above the main swap box
                    <div class="flex items-center justify-evenly gap-0.5 p-[5px] bg-muted rounded-md">
                        <button class="w-full py-1.5 px-3 rounded-sm bg-background text-foreground border-none h-8
                        ">"Swap"</button>
                        <div class="w-full group relative">
                            <button
                                disabled
                                class="!opacity-75 w-full py-1.5 px-3 rounded-sm bg-muted text-muted-foreground border-none h-8
                                "
                            >
                                "Place Order"
                            </button>
                            <div class="w-[100px] text-center absolute bottom-1/2 md:bottom-full left-1/2 -translate-x-1/2 md:mb-2 px-2 py-1 z-50 translate-y-1/2 md:translate-y-0
                            invisible group-hover:visible opacity-0 group-hover:opacity-100 transition-opacity duration-100 ease-in
                            border border-solid border-border
                            bg-popover text-popover-foreground text-xs font-semibold rounded-md whitespace-nowrap">
                                "soon"
                            </div>
                        </div>
                    </div>

                    // TODO: toggle button to show chart or something else. when that's on, switch to grid
                    // layout with grid-cols-[minmax(0px,7fr)_minmax(0px,5fr)]
                    // <div class="container block align-middle sm:row-auto row-start-2 outline outline-2 outline-neutral-700 rounded">
                    // <pre class="px-2 text-xs whitespace-pre-wrap text-neutral-300">{current_quote}</pre>
                    // </div>

                    // Main swap box
                    <div class="row-start-1 md:row-auto rounded-lg shadow-sm
                    bg-card text-card-foreground border border-solid border-border">
                        // card header
                        <div class="p-6 flex justify-between items-center">
                            <h2 class="m-0">Swap</h2>
                            <div class="relative">
                                <button
                                    on:click=toggle_swap_settings
                                    class="inline-flex items-center justify-center
                                    ml-auto w-10 h-10 text-muted-foreground
                                    rounded-md border border-solid border-border"
                                >
                                    <Settings2 size=16 />
                                </button>
                                <SwapSettings
                                    dialog_ref=settings_dialog_ref
                                    toggle_menu=toggle_swap_settings
                                    slippage=(slippage, set_slippage)
                                    deadline=(deadline, set_deadline)
                                />
                            </div>

                        </div>
                        // card body
                        <div class="px-6 pb-6 space-y-4">
                            <div class="space-y-2">
                                <div class="flex items-center justify-between">
                                    <label class="block text-sm font-medium" for="from-token">
                                        "From"
                                    </label>
                                    <Secret20Balance token_address=token_x />
                                </div>
                                <div class="flex justify-between gap-4 h-9">
                                    <input
                                        id="from-token"
                                        type="text"
                                        pattern="^[0-9]*[.,]?[0-9]*$"
                                        inputmode="decimal"
                                        placeholder="0.0"
                                        autocomplete="off"
                                        class="px-3 py-1 w-full text-sm rounded-md font-normal"
                                        prop:value=move || amount_x.get()
                                        on:input=move |ev| {
                                            set_amount_x.set(event_target_value(&ev));
                                            set_amount_y.set("".to_string());
                                            set_swap_for_y.set(true);
                                        }
                                    />
                                    <select
                                        node_ref=select_x_node_ref
                                        class="w-[135px] font-medium py-2 px-4 bg-card rounded-md"
                                        title="Select Token X"
                                        on:input=move |ev| {
                                            let token_x = event_target_value(&ev);
                                            set_token_x.set(None);
                                            set_token_x.set(Some(token_x));
                                        }
                                        prop:value=move || token_x.get().unwrap_or_default()
                                    >
                                        <option value="" disabled selected>
                                            "Select Token"
                                        </option>
                                        <option value=SYMBOL_TO_ADDR.get("SSCRT")>sSCRT</option>
                                        <option value=SYMBOL_TO_ADDR
                                            .get("STKDSCRT")>"stkd-SCRT"</option>
                                        <option value=SYMBOL_TO_ADDR.get("AMBER")>AMBER</option>
                                        <option value=SYMBOL_TO_ADDR.get("SHD")>SHD</option>
                                    </select>
                                </div>
                            </div>
                            <div class="flex items-center gap-0.5 w-full">
                                <hr class="w-full" />
                                <button
                                    type="button"
                                    aria-label="change swap direction"
                                    class="inline-flex items-center justify-center rounded-full border-0 min-w-[1.5rem] h-6 p-0
                                    hover:text-primary"
                                    on:click=move |_| {
                                        let x = token_x.get();
                                        let y = token_y.get();
                                        set_token_x.set(y);
                                        set_token_y.set(x);
                                    }
                                >
                                    <ArrowDownUp size=15 />
                                </button>
                                <hr class="w-full" />
                            </div>
                            <div class="space-y-2">
                                <div class="flex justify-between leading-none">
                                    <label class="block text-sm font-medium" for="to-token">
                                        "To"
                                    </label>
                                    <Secret20Balance token_address=token_y />
                                </div>
                                <div class="flex justify-between gap-4 h-9">
                                    <input
                                        disabled
                                        id="to-token"
                                        type="text"
                                        pattern="^[0-9]*[.,]?[0-9]*$"
                                        inputmode="decimal"
                                        placeholder="0.0"
                                        autocomplete="off"
                                        class="px-3 py-1 w-full text-sm font-normal rounded-md disabled:cursor-not-allowed"
                                        prop:value=move || amount_y.get()
                                        on:change=move |ev| {
                                            set_amount_y.set(event_target_value(&ev));
                                            set_amount_x.set("".to_string());
                                            set_swap_for_y.set(false);
                                        }
                                    />
                                    <select
                                        node_ref=select_y_node_ref
                                        title="Select Token Y"
                                        class="w-[135px] font-medium py-2 px-4 bg-card rounded-md"
                                        prop:value=move || token_y.get().unwrap_or_default()
                                        on:change=move |ev| {
                                            let token_y = event_target_value(&ev);
                                            set_token_y.set(None);
                                            set_token_y.set(Some(token_y));
                                        }
                                    >
                                        <option value="" disabled selected>
                                            "Select Token"
                                        </option>
                                        <option value=SYMBOL_TO_ADDR.get("SSCRT")>sSCRT</option>
                                        <option value=SYMBOL_TO_ADDR
                                            .get("STKDSCRT")>"stkd-SCRT"</option>
                                        <option value=SYMBOL_TO_ADDR.get("AMBER")>AMBER</option>
                                        <option value=SYMBOL_TO_ADDR.get("SHD")>SHD</option>
                                    </select>
                                </div>
                            </div>

                            <div class="flex flex-row items-center gap-2">
                                <button
                                    class="py-1.5 px-6 bg-secondary text-secondary-foreground rounded-md h-9"
                                    disabled=move || {
                                        token_x.get().is_none() || token_y.get().is_none()
                                            || amount_x.get().is_empty() || get_quote.pending().get()
                                    }
                                    on:click=handle_quote
                                >
                                    "Estimate Swap"
                                </button>
                                <Show when=move || get_quote.pending().get()>
                                    <Spinner2 size="h-6 w-6" />
                                </Show>
                            </div>

                            // Swap Details
                            <Show when=move || {
                                get_quote.value().get().is_some_and(|quote| quote.is_ok())
                            }>
                                <SwapDetails
                                    price_ratio=swap_price_ratio
                                    expected_output=amount_out
                                    minimum_received=amount_out_min
                                    price_impact
                                />
                            </Show>

                        // <Show when=move || amount_out().is_some() fallback=|| ()>
                        // <p>"Amount out: " {amount_out}</p>
                        // </Show>
                        </div>

                        // card footer
                        <div class="px-6 pb-6">
                            <button
                                class="w-full py-2 px-6 bg-primary active:brightness-90 text-primary-foreground text-sm font-medium rounded-md"
                                disabled=move || {
                                    !keplr.enabled.get()
                                        || get_quote.value().get().and_then(Result::ok).is_none()
                                }
                                on:click=handle_swap
                            >
                                "Swap"
                            </button>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}
