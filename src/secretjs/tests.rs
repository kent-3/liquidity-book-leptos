use crate::state::MyAccount;
use js_sys::{Object, JSON};
use leptos::{error::Result, *};
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;

use crate::constants::{CHAIN_ID, LCD_URL};
use crate::keplr;
use crate::secretjs;
use crate::secretjs::{ClientOptionsBuilder, SecretNetworkClient};

#[component]
pub fn SecretJsTests() -> impl IntoView {
    let ctx = use_context::<MyAccount>().expect("there should be a MyAccount context provided");

    let create_random_wallet = move |_| {
        log::debug!("trying to create wallet...");

        let wallet = crate::secretjs::Wallet::new();

        log::debug!("{:#?}", &wallet);
    };

    fn create_readonly_client() {
        log::debug!("trying to create client...");

        let client_options = ClientOptionsBuilder::new()
            .url(LCD_URL)
            .chain_id(CHAIN_ID)
            .build();
        let client = SecretNetworkClient::new(&client_options);

        log::debug!("{:#?}", &client);
    }

    fn create_signing_client() -> SecretNetworkClient {
        log::debug!("trying to create client...");

        let keplr_offline_signer = keplr::get_offline_signer().unwrap();
        let encryption_utils = keplr::get_enigma_utils(CHAIN_ID);
        let client_options = secretjs::ClientOptionsBuilder::new()
            .url(LCD_URL)
            .chain_id(CHAIN_ID)
            .encryption_utils(encryption_utils)
            .wallet(keplr_offline_signer)
            .wallet_address("secret12kw8ja5rgcxq66x3q48m9ec4n7g8a29xayprgy")
            .build();
        let client = SecretNetworkClient::new(&client_options);

        log::debug!("{:#?}", &client);

        client
    }

    async fn do_a_query(client: SecretNetworkClient) -> Result<String> {
        // TODO - make a separate query builder
        let query = Object::new();
        //  query = { token_info: {} }
        let _ = js_sys::Reflect::set(&query, &JsValue::from_str("token_info"), &Object::new());

        let query_contract_request = Object::new();
        //  query_contract_request = { code_hash?: string; contract_address: string; query: query }
        let _ = js_sys::Reflect::set(
            &query_contract_request,
            &JsValue::from_str("code_hash"),
            &JsValue::from_str("5a085bd8ed89de92b35134ddd12505a602c7759ea25fb5c089ba03c8535b3042"),
        );
        let _ = js_sys::Reflect::set(
            &query_contract_request,
            &JsValue::from_str("contract_address"),
            &JsValue::from_str("secret1s09x2xvfd2lp2skgzm29w2xtena7s8fq98v852"),
        );
        let _ = js_sys::Reflect::set(&query_contract_request, &JsValue::from_str("query"), &query);

        log::debug!("query_contract_request: {:#?}", &query_contract_request);

        let query_promise = client
            .query()
            .compute()
            .query_contract(&query_contract_request.into());
        let query_js_value = JsFuture::from(query_promise).await;

        match query_js_value {
            Ok(js_value) => {
                log::debug!("{js_value:#?}");
                Ok(JSON::stringify(&js_value).unwrap().into())
            }
            Err(js_error) => {
                log::debug!("{js_error:#?}");
                Ok(JSON::stringify(&js_error).unwrap().into())
            }
        }
    }

    let query_action = create_action(|input: &SecretNetworkClient| {
        let input = input.to_owned();
        async move { do_a_query(input).await }
    });
    let dispatch_query_action = move |_| query_action.dispatch(ctx.my_client.get());
    let pending_query = query_action.pending();
    let query_response = query_action.value();

    let create_readonly_client = move |_| {
        create_readonly_client();
    };

    let create_signing_client = move |_| {
        create_signing_client();
    };

    view! {
        <h2>"SecretJS Tests"</h2>

        <button on:click=create_random_wallet>"create_random_wallet"</button>
        <br />
        <button on:click=create_readonly_client>"create_readonly_client"</button>
        <br />
        <button on:click=create_signing_client>"create_signing_client"</button>
        <br />
        <button on:click=dispatch_query_action>"dispatch_query_action"</button>
        <br />
        <Show when=move || query_response().is_some() fallback=|| ()>
            <p>
                "Response: "
                <code>
                    {move || {
                        match query_response() {
                            Some(resp) => resp.unwrap(),
                            None => "".to_string(),
                        }
                    }}
                </code>
            </p>
        </Show>
    }
}
