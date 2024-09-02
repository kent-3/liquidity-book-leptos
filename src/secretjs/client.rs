use crate::keplr::{EnigmaUtils, KeplrOfflineSigner};
use js_sys::{Object, Promise};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[derive(Debug, Clone)]
    pub type SecretNetworkClient;
    pub type Querier;
    pub type ComputeQuerier;

    #[wasm_bindgen(constructor, js_namespace = ["window", "secretjs"])]
    pub fn new(options: &JsValue) -> SecretNetworkClient;

    #[wasm_bindgen(method, getter)]
    pub fn query(this: &SecretNetworkClient) -> Querier;

    #[wasm_bindgen(method, getter)]
    pub fn address(this: &SecretNetworkClient) -> String;

    #[wasm_bindgen(method, getter)]
    pub fn compute(this: &Querier) -> ComputeQuerier;

    #[wasm_bindgen(method, js_name = queryContract)]
    pub fn query_contract(this: &ComputeQuerier, __namedParameters: &JsValue) -> Promise;
}

// prefer using the builder
pub fn create_client_options(
    url: &str,
    chain_id: &str,
    encryption_utils: Option<&EnigmaUtils>,
    wallet: Option<&KeplrOfflineSigner>,
    wallet_address: Option<&str>,
) -> JsValue {
    let client_options = Object::new();

    let _ = js_sys::Reflect::set(
        &client_options,
        &JsValue::from_str("url"),
        &JsValue::from_str(url),
    );
    let _ = js_sys::Reflect::set(
        &client_options,
        &JsValue::from_str("chainId"),
        &JsValue::from_str(chain_id),
    );
    if let Some(encryption_utils) = encryption_utils {
        let _ = js_sys::Reflect::set(
            &client_options,
            &JsValue::from_str("encryptionUtils"),
            encryption_utils,
        );
    }
    if let Some(wallet) = wallet {
        let _ = js_sys::Reflect::set(&client_options, &JsValue::from_str("wallet"), wallet);
        let _ = js_sys::Reflect::set(
            &client_options,
            &JsValue::from_str("walletAddress"),
            &JsValue::from_str(
                wallet_address.expect("walletAddress is required for signing client"),
            ),
        );
    }

    client_options.into()
}

pub struct ClientOptionsBuilder {
    url: Option<String>,
    chain_id: Option<String>,
    encryption_utils: Option<EnigmaUtils>,
    wallet: Option<KeplrOfflineSigner>,
    wallet_address: Option<String>,
}

impl ClientOptionsBuilder {
    pub fn new() -> Self {
        ClientOptionsBuilder {
            url: None,
            chain_id: None,
            encryption_utils: None,
            wallet: None,
            wallet_address: None,
        }
    }

    pub fn url(mut self, url: &str) -> Self {
        self.url = Some(url.to_string());
        self
    }

    pub fn chain_id(mut self, chain_id: &str) -> Self {
        self.chain_id = Some(chain_id.to_string());
        self
    }

    pub fn encryption_utils(mut self, encryption_utils: EnigmaUtils) -> Self {
        self.encryption_utils = Some(encryption_utils);
        self
    }

    pub fn wallet(mut self, wallet: KeplrOfflineSigner) -> Self {
        self.wallet = Some(wallet);
        self
    }

    pub fn wallet_address(mut self, wallet_address: &str) -> Self {
        self.wallet_address = Some(wallet_address.to_string());
        self
    }

    pub fn build(self) -> JsValue {
        let client_options = Object::new();

        if let Some(url) = self.url {
            let _ = js_sys::Reflect::set(
                &client_options,
                &JsValue::from_str("url"),
                &JsValue::from_str(&url),
            );
        }

        if let Some(chain_id) = self.chain_id {
            let _ = js_sys::Reflect::set(
                &client_options,
                &JsValue::from_str("chainId"),
                &JsValue::from_str(&chain_id),
            );
        }

        if let Some(encryption_utils) = self.encryption_utils {
            let _ = js_sys::Reflect::set(
                &client_options,
                &JsValue::from_str("encryptionUtils"),
                &encryption_utils,
            );
        }

        if let Some(wallet) = self.wallet {
            let _ = js_sys::Reflect::set(&client_options, &JsValue::from_str("wallet"), &wallet);
        }

        if let Some(wallet_address) = self.wallet_address {
            let _ = js_sys::Reflect::set(
                &client_options,
                &JsValue::from_str("walletAddress"),
                &JsValue::from_str(&wallet_address),
            );
        }

        client_options.into()
    }
}
