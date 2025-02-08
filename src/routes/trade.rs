use crate::{
    components::Secret20Balance,
    components::Spinner2,
    constants::{contracts::*, CHAIN_ID, NODE, SYMBOL_TO_ADDR, TOKEN_MAP},
    error::Error,
    state::*,
    utils::{display_token_amount, parse_token_amount},
    LoadingModal,
};
use ammber_sdk::contract_interfaces::{lb_quoter::Quote, lb_router::Path, *};
use codee::string::FromToStringCodec;
use cosmwasm_std::{to_binary, Addr, Uint128, Uint64};
use keplr::Keplr;
use leptos::{ev, html, logging::*, prelude::*, tachys::dom::window};
use leptos_router::{hooks::query_signal_with_options, NavigateOptions};
use leptos_use::storage::use_local_storage;
use liquidity_book::core::TokenType;
use lucide_leptos::{ArrowDownUp, ChevronDown, Info, Settings2, TriangleAlert, X};
use rsecret::{
    secret_client::CreateTxSenderOptions,
    tx::{compute::MsgExecuteContractRaw, ComputeServiceClient},
    TxOptions,
};
use secretrs::AccountId;
use std::str::FromStr;
use tracing::{debug, info};
use web_sys::js_sys::Date;
use web_sys::wasm_bindgen::JsCast;

#[component]
fn KeyboardShortcuts() -> impl IntoView {
    let handle_shortcut = move |ev: web_sys::KeyboardEvent| {
        let target = ev.target();

        // Check if the event is coming from an input field
        if let Some(target) = target.and_then(|t| t.dyn_into::<web_sys::HtmlElement>().ok()) {
            let tag = target.tag_name().to_lowercase();
            if tag == "input" || tag == "textarea" || target.is_content_editable() {
                return; // Don't trigger shortcut inside input fields
            }
        }
        if ev.ctrl_key() {
            // Check if Ctrl is held
            match ev.code().as_str() {
                "Digit1" => log!("Ctrl + 1 pressed → Action 1"),
                "Digit2" => log!("Ctrl + 2 pressed → Action 2"),
                "Digit3" => log!("Ctrl + 3 pressed → Action 3"),
                _ => {}
            }
        }
    };

    // Attach a global keydown listener
    window_event_listener(ev::keydown, handle_shortcut);

    view! {
        <p>
            "Keyboard shortcuts: Press "<kbd>"1"</kbd>", "<kbd>"2"</kbd>", or "<kbd>"3"</kbd>
            " to trigger actions."
        </p>
    }
}

#[component]
pub fn Trade() -> impl IntoView {
    info!("rendering <Pool/>");

    on_cleanup(move || {
        info!("cleaning up <Pool/>");
    });

    let endpoint = use_context::<Endpoint>().expect("endpoint context missing!");
    let keplr = use_context::<KeplrSignals>().expect("keplr signals context missing!");
    let token_map = use_context::<TokenMap>().expect("tokens context missing!");

    // prevents scrolling to the top of the page each time a query param changes
    let nav_options = NavigateOptions {
        scroll: false,
        ..Default::default()
    };

    let (token_x, set_token_x) = query_signal_with_options::<String>("from", nav_options.clone());
    let (token_y, set_token_y) = query_signal_with_options::<String>("to", nav_options.clone());

    let token_x_info = move || token_x.get().and_then(|ref address| TOKEN_MAP.get(address));
    let token_y_info = move || token_y.get().and_then(|ref address| TOKEN_MAP.get(address));

    let (amount_x, set_amount_x) = signal(String::default());
    let (amount_y, set_amount_y) = signal(String::default());
    let (swap_for_y, set_swap_for_y) = signal(true);

    let (slippage, set_slippage, _) = use_local_storage::<u16, FromToStringCodec>("swap_slippage");
    let (deadline, set_deadline, _) =
        use_local_storage::<String, FromToStringCodec>("swap_deadline");

    if slippage.get() == 0 {
        set_slippage.set(50);
    }
    if deadline.get().is_empty() {
        set_deadline.set("5".to_string());
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
        _ = get_quote.dispatch((
            token_x.get().unwrap(),
            token_y.get().unwrap(),
            amount_x.get(),
        ))
    };

    Effect::new(move || {
        if let Some(Ok(quote)) = get_quote.value().get() {
            if let Some(amount_out) = quote.amounts.last() {
                set_amount_y.set(display_token_amount(
                    amount_out.u128(),
                    token_y_info().unwrap().decimals,
                ))
            }
        }
    });

    let current_quote = move || {
        get_quote
            .value()
            .get()
            .and_then(Result::ok)
            .map(|quote| serde_json::to_string_pretty(&quote).unwrap())
    };

    let path = move || {
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

    // TODO: how will we recheck the balances after a successful swap?
    let swap = Action::new_local(move |quote: &Quote| {
        // TODO: Use the dynamic versions instead.
        // let url = endpoint.get();
        // let chain_id = chain_id.get();
        let url = NODE;
        let chain_id = CHAIN_ID;

        let quote = quote.clone();

        async move {
            let Ok(key) = Keplr::get_key(CHAIN_ID).await else {
                return Err(Error::generic("Could not get key from Keplr"));
            };

            // smallest supported slippage = 0.01%
            // let slippage = ((1.0 - slippage.get().parse::<f64>()?) * 10_000.0).round() as u16;
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
            let deadline = deadline.get().parse::<u64>()? * 60 + (Date::now() / 1000.0) as u64;

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

    view! {
        <LoadingModal when=swap.pending() message="Processing Transaction... (watch the console)" />
        <div class="flex mt-2 md:mt-10 justify-center">
            // <div class="grid gap-4 sm:grid-cols-[minmax(0px,7fr)_minmax(0px,5fr)] grid-cols-1 grid-rows-2 sm:grid-rows-1">
            // <div class="grid gap-4 grid-cols-1 max-w-[550px] w-full">
            <div class="grid gap-4 grid-cols-1 max-w-sm">
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
                            <div class="absolute bottom-1/2 md:bottom-full left-1/2 -translate-x-1/2 md:mb-2 px-2 py-1 z-50 translate-y-1/2 md:translate-y-0
                            invisible group-hover:visible opacity-0 group-hover:opacity-100 transition-opacity duration-100 ease-in
                            border border-solid border-border
                            bg-popover text-popover-foreground text-xs font-semibold rounded-md whitespace-nowrap">
                                "soon"
                            // <div class="absolute left-1/2 -translate-x-1/2 top-full -mt-1 w-2 h-2 bg-neutral-500 rotate-45"></div>
                            </div>
                        </div>
                    </div>

                    // <div class="inline-flex items-center gap-0.5 mt-2 mb-4 p-[5px] bg-muted rounded-md">
                    // <A href="manage">
                    // <button tabindex="-1" class="py-1.5 px-3 rounded-sm bg-muted text-muted-foreground border-none h-8 w-[95px]">"Manage"</button>
                    // </A>
                    // <A href="analytics">
                    // <button tabindex="-1" class="py-1.5 px-3 rounded-sm bg-muted text-muted-foreground border-none h-8 w-[95px]">"Analytics"</button>
                    // </A>
                    // </div>

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
                                    <Secret20Balance token_address=token_x.into() />
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
                                        class="w-[135px] font-medium py-2 px-4 text-sm bg-card rounded-md"
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
                            // TODO: switch tokens separator
                            // <div class="flex items-center gap-1 w-full">
                            // <div class="h-0.5 bg-border w-full"></div>
                            // <button
                            // type="button"
                            // aria-label="change swap direction"
                            // class="inline-flex items-center justify-center rounded-full border-0 min-w-[2.5rem] h-10 p-0 bg-transparent
                            // hover:bg-muted transition-colors duration-200 active:bg-transparent"
                            // >
                            // <ArrowUpDown size=16 />
                            // </button>
                            // <div class="h-0.5 bg-border w-full"></div>
                            // </div>
                            <div class="flex items-center gap-0.5 w-full">
                                <hr class="w-full" />
                                <button
                                    type="button"
                                    aria-label="change swap direction"
                                    class="inline-flex items-center justify-center rounded-full border-0 min-w-[1.5rem] h-6 p-0
                                    hover:text-primary"
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
                                    <Secret20Balance token_address=token_y.into() />
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
                                        class="w-[135px] font-medium py-2 px-4 text-sm bg-card rounded-md"
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

#[component]
fn SwapDetails(
    #[prop(into)] price_ratio: Signal<Option<f64>>,
    #[prop(into)] expected_output: Signal<Option<Uint128>>,
    #[prop(into)] minimum_received: Signal<Option<Uint128>>,
    #[prop(into)] price_impact: Signal<f64>,
) -> impl IntoView {
    let (expanded, set_expanded) = signal(false);

    let content_ref = NodeRef::<html::Div>::new();

    let toggle_expand = move |_: ev::MouseEvent| {
        if let Some(content) = content_ref.get() {
            // let full_height = content.get_bounding_client_rect().height();
            let full_height = content.scroll_height();

            if expanded.get() {
                // Ensure the content has an explicit height before collapsing
                content.style(("height", format!("{}px", full_height)));
                request_animation_frame(move || {
                    content.style(("height", "0px"));
                });
            } else {
                // First, set the height explicitly (this fixes the first animation issue)
                content.style(("height", "0px"));
                request_animation_frame(move || {
                    content.style(("height", format!("{}px", full_height)));
                });

                // Reset height to `auto` after transition ends to allow dynamic resizing
                let expanded_signal = expanded.clone();
                window_event_listener_untyped("transitionend", move |_| {
                    if expanded_signal.get() {
                        if let Some(content) = content_ref.get() {
                            content.style(("height", "auto"));
                        }
                    }
                });
            }
        }
        set_expanded.update(|e| *e = !*e);
    };

    view! {
        <div class="flex flex-col w-full rounded-md box-border border border-solid border-border">
            // Header (Click to Toggle)
            <div
                class="min-h-[40px] px-4 flex items-center justify-between cursor-pointer"
                on:click=toggle_expand
            >
                // TODO: toggle between price ratio on click. somehow make this take precedence
                // over the toggle_expand for the whole header.
                // NOTE: This price ratio is based on the expected output (amount_out).
                // TODO: add token symbols to this string.
                <p on:click=move |_| () class="m-0 text-sm text-white font-semibold">
                    {move || price_ratio.get().map(|uint128| uint128.to_string())}
                // "1 AVAX = 35.37513945 USDC"
                </p>
                <div
                    class="flex items-center justify-center transition-transform"
                    class=("rotate-180", move || expanded.get())
                >
                    <ChevronDown size=20 />
                </div>
            </div>

            // Expandable Content
            <div
                node_ref=content_ref
                class="transition-all ease-standard box-border overflow-hidden"
                class=(["opacity-0", "invisible", "h-0"], move || !expanded.get())
                class=(["opacity-100", "visible"], move || expanded.get())
            >
                <div class="w-full box-border p-4 pt-2 flex flex-col gap-2 items-center">
                    <div class="w-full flex flex-row justify-between text-sm">
                        <p class="m-0 text-muted-foreground">"Expected Output:"</p>
                        <p class="m-0 text-foreground font-semibold">
                            {move || expected_output.get().map(|uint128| uint128.to_string())}
                        </p>
                    </div>
                    <div class="w-full flex flex-row justify-between text-sm">
                        <p class="m-0 text-muted-foreground">"Minimum Received:"</p>
                        <p class="m-0 text-foreground font-semibold">
                            {move || minimum_received.get().map(|uint128| uint128.to_string())}
                        </p>
                    </div>
                    <div class="w-full flex flex-row justify-between text-sm">
                        <p class="m-0 text-muted-foreground">"Price Impact:"</p>
                        <p class="m-0 text-foreground font-semibold">
                            {move || price_impact.get()}
                        </p>
                    </div>
                </div>
            </div>

            // Warning (Price Impact, etc)
            <Show when=move || price_impact.get().gt(&2.0)>
                <div class="flex flex-col items-center gap-2 m-2 mt-0">
                    <div class="flex items-center justify-between box-border w-full px-4 py-2 text-sm text-white font-semibold bg-red-500/90 rounded-md">
                        // price impact icon and text
                        <div class="flex flex-row items-center gap-3">
                            <TriangleAlert size=20 />
                            <p class="m-0">"Price Impact Warning"</p>
                        </div>
                        // price impact percentage
                        <p class="m-0">{move || price_impact.get()}"%"</p>
                    </div>
                </div>
            </Show>
        </div>
    }
}

#[component]
fn SwapSettings(
    dialog_ref: NodeRef<html::Dialog>,
    toggle_menu: impl Fn(ev::MouseEvent) + 'static,
    slippage: (Signal<u16>, WriteSignal<u16>),
    deadline: (Signal<String>, WriteSignal<String>),
) -> impl IntoView {
    info!("rendering <SettingsMenu/>");

    view! {
        <div class="floating-menu">
            <dialog
                node_ref=dialog_ref
                class="z-40 mt-1.5 -mr-0 md:-mr-[124px] w-80 h-52 p-0 shadow-md bg-background text-foreground rounded-md border border-solid border-border"
            >
                <div class="relative flex flex-col z-auto">
                    // <div class="absolute right-1.5 top-1.5 flex shrink-0 items-center justify-center w-6 h-6 p-1 box-border rounded-md hover:bg-neutral-700">
                    // <X size=16 />
                    // </div>
                    <div class="flex justify-between items-center p-2 pl-3 text-popover-foreground border-0 border-b border-solid border-border">
                        <p class="m-0">"Settings"</p>
                        <button
                            autofocus
                            on:click=toggle_menu
                            class="appearance-none border-0
                            flex shrink-0 items-center justify-center w-6 h-6 p-1 box-border rounded-md
                            bg-transparent hover:bg-muted transition-colors duration-200 ease-standard
                            "
                        >
                            <X size=16 />
                        </button>
                    </div>
                    <div class="px-3 py-4 box-border">
                        <div class="flex flex-col items-start gap-4 w-full">
                            <div class="flex flex-col items-start gap-2 w-full">
                                <div class="flex flex-row items-center justify-between gap-2 w-full">
                                    <p class="text-muted-foreground text-sm m-0">
                                        "Slippage tolerance"
                                    </p>
                                    <div class="relative group focus-within:group">
                                        <div
                                            tabindex="0"
                                            class="text-foreground focus:outline-none"
                                        >
                                            <Info size=16 />
                                        </div>
                                        <div class="absolute w-[200px] z-50 bottom-full right-0 lg:right-1/2 translate-x-0 lg:translate-x-1/2
                                        bg-popover text-popover-foreground text-xs font-normal rounded-md border border-solid
                                        mb-1 p-2 invisible opacity-0 transition-opacity duration-100 ease-in
                                        group-hover:visible group-hover:opacity-100 group-focus-within:visible group-focus-within:opacity-100">
                                            "Your transaction will revert if the price changes unfavorably by more than this percentage."
                                        </div>
                                    </div>
                                </div>
                                <div class="flex flex-row items-center gap-2">
                                    <div class="flex flex-row items-center gap-1">
                                        <button
                                            on:click=move |_| slippage.1.set(10)
                                            class="h-8 min-w-8 w-16 text-sm font-semibold bg-secondary text-secondary-foreground rounded-md"
                                        >
                                            "0.1%"
                                        </button>
                                        <button
                                            on:click=move |_| slippage.1.set(50)
                                            class="h-8 min-w-8 w-16 text-sm font-semibold bg-secondary text-secondary-foreground rounded-md"
                                        >
                                            "0.5%"
                                        </button>
                                        <button
                                            on:click=move |_| slippage.1.set(100)
                                            class="h-8 min-w-8 w-16 text-sm font-semibold bg-secondary text-secondary-foreground rounded-md"
                                        >
                                            "1%"
                                        </button>
                                    </div>
                                    <div class="w-full relative flex items-center isolate box-border">
                                        <input
                                            class="w-full box-border px-3 h-8 text-sm font-semibold bg-transparent text-popover-foreground rounded-md"
                                            inputmode="decimal"
                                            minlength="1"
                                            maxlength="79"
                                            type="text"
                                            pattern="^[0-9]*[.,]?[0-9]*$"
                                            placeholder="0.5"
                                            prop:value=move || { slippage.0.get() as f64 / 100.0 }
                                            on:input=move |ev| {
                                                let value = event_target_value(&ev)
                                                    .parse::<f64>()
                                                    .unwrap_or_default();
                                                let value = (value * 100.0).round() as u16;
                                                slippage.1.set(value)
                                            }
                                        />
                                        <div class="absolute right-0 top-0 w-8 h-8 z-[2] flex items-center justify-center text-popover-foreground">
                                            "%"
                                        </div>
                                    </div>
                                </div>
                            </div>
                            <div class="flex flex-col items-start gap-2">
                                <p class="text-muted-foreground text-sm m-0">
                                    "Transaction deadline"
                                </p>
                                <div class="w-full relative flex items-center isolate box-border">
                                    <input
                                        class="w-full box-border px-3 h-8 text-sm font-semibold bg-transparent text-popover-foreground rounded-md"
                                        inputmode="decimal"
                                        minlength="1"
                                        maxlength="79"
                                        type="text"
                                        pattern="^[0-9]*[.,]?[0-9]*$"
                                        placeholder="10"
                                        bind:value=deadline
                                    />
                                    <div class="absolute right-0 top-0 min-w-fit h-8 mr-4 z-[2] flex items-center justify-center text-sm text-popover-foreground">
                                        "minutes"
                                    </div>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
            </dialog>
        </div>
    }
}
