#![allow(unused)]

use ammber_core::state::*;
use ammber_core::support::{chain_query, ILbPair, Querier, COMPUTE_QUERIER};
use ammber_core::{prelude::*, Error};
use ammber_sdk::contract_interfaces::{
    lb_pair::{self, LbPair},
    lb_router,
};
use cosmwasm_std::{Addr, ContractInfo, Uint128, Uint256, Uint64};
use ethnum::U256;
use keplr::Keplr;
use leptos::prelude::*;
use leptos_router::{
    hooks::{use_params, use_params_map},
    NavigateOptions,
};
use liquidity_book::libraries::{
    math::{
        u256x256_math::U256x256MathError,
        uint256_to_u256::{ConvertU256, ConvertUint256},
    },
    PriceHelper,
};
use rsecret::{
    query::tendermint::TendermintQuerier,
    secret_client::{CreateTxSenderOptions, TxDecrypter},
    tx::ComputeServiceClient,
    TxOptions,
};
use secret_toolkit_snip20::TokenInfoResponse;
use secretrs::compute::MsgExecuteContractResponse;
use secretrs::tx::Msg;
use secretrs::{compute::MsgExecuteContract, AccountId};
use std::str::FromStr;
use tonic_web_wasm_client::Client;
use tracing::{debug, error, info};

#[component]
pub fn RemoveLiquidity() -> impl IntoView {
    info!("rendering <RemoveLiquidity/>");

    on_cleanup(move || {
        info!("cleaning up <RemoveLiquidity/>");
    });

    let endpoint = use_context::<Endpoint>().expect("endpoint context missing!");
    let chain_id = use_context::<ChainId>().expect("chain_id context missing!");
    let keplr = use_context::<KeplrSignals>().expect("keplr signals context missing!");
    let token_map = use_context::<TokenMap>().expect("tokens context missing!");

    let params = use_params_map();
    let token_a = move || params.read().get("token_a").unwrap_or_default();
    let token_b = move || params.read().get("token_b").unwrap_or_default();
    let bin_step = move || {
        params
            .read()
            .get("bps") // bps = basis points
            .and_then(|string| string.parse::<u16>().ok())
            .unwrap_or_default()
    };

    // TODO: move these to support/utils
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

    // TODO: pass this info from parent with context
    let token_a_symbol =
        AsyncDerived::new_unsync(move || async move { token_symbol_convert(token_a()).await });
    let token_b_symbol =
        AsyncDerived::new_unsync(move || async move { token_symbol_convert(token_b()).await });

    // prevents scrolling to the top of the page each time a query param changes
    let nav_options = NavigateOptions {
        scroll: false,
        ..Default::default()
    };

    let active_id = use_context::<LocalResource<Result<u32, Error>>>()
        .expect("missing the active_id resource context");
    let lb_pair = use_context::<LocalResource<Result<LbPair, Error>>>()
        .expect("missing the LbPair resource context");

    let (amount_x, set_amount_x) = signal("0.0".to_string());
    let (amount_y, set_amount_y) = signal("0.0".to_string());

    let find_liquidity = Action::new_local(move |_: &()| {
        let url = NODE;
        let chain_id = CHAIN_ID;

        async move {
            let Ok(lb_pair) = lb_pair.await else {
                return Err(Error::generic("lb pair information is missing!"));
            };
            let Ok(id) = active_id.await else {
                return Err(Error::generic("active id is missing!"));
            };

            let mut ids = vec![];
            let radius = 49;

            for i in 0..(radius * 2 + 1) {
                let offset_id = if i < radius {
                    id - (radius - i) as u32 // Subtract for the first half
                } else {
                    id + (i - radius) as u32 // Add for the second half
                };

                ids.push(offset_id);
            }

            debug!("{:?}", ids);

            let key = Keplr::get_key(&chain_id).await?;
            let account = key.bech32_address;

            let accounts = vec![account.clone(); ids.len()];

            let balances: Vec<Uint256> = ILbPair(lb_pair.contract.clone())
                .balance_of_batch(accounts, ids.clone())
                .await?;
            // .and_then(|vec| {
            //     vec.iter()
            //         .map(|el| {
            //             PriceHelper::convert128x128_price_to_decimal(el.uint256_to_u256())
            //         })
            //         .collect::<Result<Vec<U256>, U256x256MathError>>()
            //         .map_err(|e| Error::generic(e.to_string()))
            // .map(|hmm| {
            //     hmm.into_iter()
            //         .map(|el| el.u256_to_uint256())
            //         .collect::<Vec<Uint256>>()
            // })
            // .map(|hmm| {
            //     hmm.into_iter()
            //         .map(|el| el.to_string())
            //         .collect::<Vec<String>>()
            // })
            // });

            debug!("{:?}", balances);

            Ok((ids, balances))
        }
    });

    let liquidity_parameters = move || {
        let amount_x = amount_x.get();
        let amount_y = amount_y.get();

        let binding = lb_pair.get();

        let Some(Ok(lb_pair)) = binding.as_deref() else {
            return Err(Error::generic("lb pair information is missing!"));
        };

        let token_x = &lb_pair.token_x;
        let token_y = &lb_pair.token_y;
        let bin_step = lb_pair.bin_step;

        let decimals_x = get_token_decimals(token_x.address().as_str())?;
        let decimals_y = get_token_decimals(token_y.address().as_str())?;

        let amount_x = parse_token_amount(amount_x, decimals_x);
        let amount_y = parse_token_amount(amount_y, decimals_y);

        let ids: Vec<u32> = vec![];
        let amounts: Vec<Uint256> = vec![]; // these amounts are expressed in lb_token terms
        let Some(Ok((ids, amounts))) = find_liquidity.value().get() else {
            return Err(Error::generic("liquidity is missing!"));
        };

        // This will filter out any bin ids with a corresponding amount of 0
        let (ids, amounts) = ids
            .into_iter()
            .zip(amounts)
            .filter(|(_, amount)| *amount != Uint256::zero())
            .unzip();

        // TODO: determine the expected amounts of x and y that should be returned. I assume there
        // is a formula that involves the lb-pair's bin reserves and the user's share of lb-tokens
        // and the total supply of lb-tokens.

        let message = lb_router::ExecuteMsg::RemoveLiquidity {
            token_x: token_x.into_contract_info().unwrap(),
            token_y: token_y.into_contract_info().unwrap(),
            bin_step,
            amount_x_min: amount_x.into(),
            amount_y_min: amount_y.into(),
            ids,
            amounts,
            to: String::new(),
            deadline: Uint64::MIN,
        };

        Ok(message)
    };

    // Effect::new(move || debug!("{:?}", liquidity_parameters()));

    let remove_liquidity = Action::new_local(move |_: &()| {
        // TODO: Use the dynamic versions instead.
        // let url = endpoint.get();
        // let chain_id = chain_id.get();
        let url = NODE;
        let chain_id = CHAIN_ID;
        let lb_router_contract = &LB_ROUTER;

        async move {
            let amount_x = amount_x.get_untracked();
            let amount_y = amount_y.get_untracked();

            let Ok(lb_pair) = lb_pair.await else {
                return Err(Error::generic("lb pair information is missing!"));
            };

            let token_x = lb_pair.token_x;
            let token_y = lb_pair.token_y;
            let bin_step = lb_pair.bin_step;

            let decimals_x = get_token_decimals(token_x.address().as_str())?;
            let decimals_y = get_token_decimals(token_y.address().as_str())?;

            let amount_x = parse_token_amount(amount_x, decimals_x);
            let amount_y = parse_token_amount(amount_y, decimals_y);

            let ids: Vec<u32> = vec![];
            let amounts: Vec<Uint256> = vec![]; // these amounts are expressed in lb_token terms
            let Some(Ok((ids, amounts))) = find_liquidity.value().get_untracked() else {
                return Err(Error::generic("liquidity is missing!"));
            };

            // This will filter out any bin ids with a corresponding amount of 0
            let (ids, amounts) = ids
                .into_iter()
                .zip(amounts)
                .filter(|(_, amount)| *amount != Uint256::zero())
                .unzip();

            // let Ok(liquidity_parameters) = liquidity_parameters else {
            //     return Err(Error::generic(
            //         "Something is wrong with the liquidity parameters!",
            //     ));
            // };

            let key = Keplr::get_key(&chain_id).await?;
            // let wallet = Keplr::get_offline_signer_only_amino(&chain_id);
            let wallet = Keplr::get_offline_signer(&chain_id);
            let enigma_utils = Keplr::get_enigma_utils(&chain_id).into();

            // TODO: I need to make this type use Strings instead of &'static str, because the
            // values are not static in this application (user is able to set them to anything).
            let options = CreateTxSenderOptions {
                url,
                chain_id,
                wallet: wallet.into(),
                wallet_address: key.bech32_address.clone().into(),
                enigma_utils,
            };
            let wasm_web_client = tonic_web_wasm_client::Client::new(url.to_string());
            let compute_service_client = ComputeServiceClient::new(wasm_web_client, options);

            // Recheck the latest block height to update the deadline.
            let tendermint = TendermintQuerier::new(Client::new(url.to_string()));
            let latest_block_time = tendermint
                .get_latest_block()
                .await
                .map(|block| block.header.time.unix_timestamp() as u64)
                .map_err(Error::from)?;

            let msg = lb_router::ExecuteMsg::RemoveLiquidity {
                token_x: token_x.into_contract_info().unwrap(),
                token_y: token_y.into_contract_info().unwrap(),
                bin_step,
                amount_x_min: amount_x.into(),
                amount_y_min: amount_y.into(),
                ids,
                amounts,
                to: key.bech32_address.clone(),
                deadline: (latest_block_time + 100).into(),
            };

            debug!("{msg:#?}");

            let remove_liquidity_msg = MsgExecuteContract {
                sender: AccountId::from_str(key.bech32_address.as_ref())?,
                contract: AccountId::from_str(lb_router_contract.address.as_ref())?,
                msg: compute_service_client
                    .encrypt(&lb_router_contract.code_hash, &msg)
                    .await?
                    .into_inner(),
                sent_funds: vec![],
            };

            let tx_options = TxOptions {
                gas_limit: 1_000_000,
                ..Default::default()
            };

            let tx = compute_service_client
                .broadcast(vec![remove_liquidity_msg], tx_options)
                .await
                .map_err(Error::from)
                .inspect(|tx_response| info!("{tx_response:?}"))
                .inspect_err(|error| error!("{error}"))?;

            if tx.code != 0 {
                error!("{}", tx.raw_log);
            }

            let data = MsgExecuteContractResponse::from_any(&tx.data[0])
                .inspect_err(|e| error! {"{e}"})?
                .data;

            let remove_liquidity_response =
                serde_json::from_slice::<lb_router::RemoveLiquidityResponse>(&data)?;

            debug!("X received: {}", remove_liquidity_response.amount_x);
            debug!("Y received: {}", remove_liquidity_response.amount_y);

            Ok(())
        }
    });

    view! {
        <div class="space-y-2">
            <div class="text-base font-semibold">"Remove Liquidity"</div>

            <button class="block bg-secondary" on:click=move |_| _ = find_liquidity.dispatch(())>
                "Find Liquidity"
            </button>
            // <pre>{move || find_liquidity.value_local().get().and_then(Result::ok).and_then(Result::ok).unwrap_or_default() }</pre>
            <button
                class="block bg-secondary"
                on:click=move |_| _ = remove_liquidity.dispatch(())
                disabled=move || find_liquidity.value().get().is_none()
            >
                "Remove Liquidity"
            </button>
        </div>
    }
}
