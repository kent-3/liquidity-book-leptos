use ammber_core::{state::*, types::Coin, Error, CHAIN_ID};
use keplr::Keplr;
use leptos::prelude::*;
use rsecret::query::bank::BankQuerier;
use send_wrapper::SendWrapper;
use tonic_web_wasm_client::Client;
use tracing::{debug, error, info};

#[component]
pub fn Home() -> impl IntoView {
    info!("rendering <Home/>");

    let endpoint = use_context::<Endpoint>().expect("endpoint context missing!");
    let keplr = use_context::<KeplrSignals>().expect("keplr signals context missing!");
    let token_map = use_context::<TokenMap>().expect("tokens context missing!");

    let viewing_keys = Resource::new(
        move || keplr.key.track(),
        move |_| {
            let tokens = token_map.clone();
            SendWrapper::new(async move {
                if keplr.enabled.get_untracked() {
                    debug!("gathering viewing_keys");
                    let mut keys = Vec::new();
                    for (_, token) in tokens.iter() {
                        let key_result =
                            Keplr::get_secret_20_viewing_key(CHAIN_ID, &token.contract_address)
                                .await;

                        if let Ok(key) = key_result {
                            keys.push((token.name.clone(), token.contract_address.clone(), key));
                        }
                    }
                    debug!("Found {} viewing keys.", keys.len());
                    keys
                } else {
                    vec![]
                }
            })
        },
    );

    let viewing_keys_list = move || {
        Suspend::new(async move {
            viewing_keys
                .await
                .into_iter()
                .map(|(name, address, key)| {
                    view! {
                        <li>
                            <strong>{name}</strong>
                            ", "
                            {address}
                            ": "
                            {key}
                        </li>
                    }
                })
                .collect_view()
        })
    };

    let user_balance = Resource::new(
        move || keplr.key.track(),
        move |_| {
            let client = Client::new(endpoint.get().to_string());
            SendWrapper::new(async move {
                let bank = BankQuerier::new(client);
                let key = keplr.key.await?;

                bank.balance(key.bech32_address, "uscrt")
                    .await
                    .map(|balance| Coin::from(balance.balance.unwrap()))
                    .map_err(Error::from)
                    .inspect(|coin| debug!("{coin:?}"))
                    .inspect_err(|err| error!("{err:?}"))
            })
        },
    );

    // TODO: move all static resources like this (query response is always the same) to a separate
    // module. Implement caching with local storage. They can all use a random account for the
    // EncryptionUtils, since they don't depend on user address.

    // let enigma_utils = EnigmaUtils::new(None, "secret-4").unwrap();
    // let contract_address = "secret1s09x2xvfd2lp2skgzm29w2xtena7s8fq98v852";
    // let code_hash = "9a00ca4ad505e9be7e6e6dddf8d939b7ec7e9ac8e109c8681f10db9cacb36d42";
    // let token_info = Resource::new(
    //     || (),
    //     move |_| {
    //         debug!("loading token_info resource");
    //         let compute =
    //             ComputeQuerier::new(Client::new(endpoint.get()), enigma_utils.clone().into());
    //         SendWrapper::new(async move {
    //             let query = QueryMsg::TokenInfo {};
    //             compute
    //                 .query_secret_contract(contract_address, code_hash, query)
    //                 .await
    //                 .map_err(Error::generic)
    //         })
    //     },
    // );

    view! {
        <div class="p-2 max-w-lg">
            <div class="text-3xl font-bold mb-4">"Introduction"</div>
            <p>
                "This project presents an efficient Automated Market Maker (AMM)
                protocol, modeled after the Liquidity Book protocol used by Trader Joe ("
                <a
                    href="https://docs.traderjoexyz.com/concepts/concentrated-liquidity"
                    target="_blank"
                    rel="noopener noreferrer"
                >
                    "Liquidity Book docs"
                </a>"). The protocol retains the key features of its predecessor, such as:"
            </p>
            <ul>
                <li>
                    <strong>"No Slippage: "</strong>
                    <span>"Enabling token swaps with zero slippage within designated bins"</span>
                </li>
                <li>
                    <strong>"Adaptive Pricing: "</strong>
                    <span>
                        "Offering Liquidity Providers extra dynamic fees during periods of
                        increased market volatility"
                    </span>
                </li>
                <li>
                    <strong>"Enhanced Capital Efficiency: "</strong>
                    <span>
                        "Facilitating high-volume trading with minimal liquidity requirements"
                    </span>
                </li>
                <li>
                    <strong>"Customizable Liquidity: "</strong>
                    <span>
                        "Liquidity providers can customize their liquidity distributions
                        based on their strategy"
                    </span>
                </li>
            </ul>
        </div>
    }
}
