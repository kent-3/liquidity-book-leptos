use crate::{
    constants::*,
    error::Error,
    keplr::{tokens::ContractInfo, Keplr, Key},
};
use leptos::prelude::*;
use send_wrapper::SendWrapper;
use std::{collections::HashMap, ops::Deref};
use tonic_web_wasm_client::Client;
use tracing::{debug, trace};

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Endpoint {
    pub url: RwSignal<String>,
}

impl Endpoint {
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: RwSignal::new(url.into()),
        }
    }
}

impl Default for Endpoint {
    fn default() -> Self {
        Self {
            url: RwSignal::new(GRPC_URL.to_string()),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct WasmClient {
    pub client: RwSignal<Client>,
    pub url: RwSignal<String>,
}

impl WasmClient {
    pub fn new() -> Self {
        Self {
            client: RwSignal::new(Client::new(GRPC_URL.to_string())),
            url: RwSignal::new(GRPC_URL.to_string()),
        }
    }
}

impl std::ops::Deref for WasmClient {
    type Target = RwSignal<Client>;

    fn deref(&self) -> &Self::Target {
        &self.client
    }
}

#[derive(Clone, Debug)]
pub struct TokenMap(pub HashMap<String, ContractInfo>);

impl TokenMap {
    pub fn new() -> Self {
        let json = include_str!(concat!(env!("OUT_DIR"), "/token_map.json"));
        let token_map: HashMap<String, ContractInfo> =
            serde_json::from_str(json).expect("Failed to deserialize token_map");

        Self(token_map)
    }
}

impl std::ops::Deref for TokenMap {
    type Target = HashMap<String, ContractInfo>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<HashMap<String, ContractInfo>> for TokenMap {
    fn as_ref(&self) -> &HashMap<String, ContractInfo> {
        &self.0
    }
}

#[derive(Copy, Clone)]
pub struct KeplrSignals {
    pub enabled: RwSignal<bool>,
    pub key: AsyncDerived<Result<Key, Error>, LocalStorage>,
}

impl KeplrSignals {
    pub fn new() -> Self {
        let enabled = RwSignal::new(false);
        let key = AsyncDerived::new_unsync(move || async move {
            if enabled.get() {
                trace!("Updating Keplr key (derived signal)");
                Keplr::get_key(CHAIN_ID).await.map_err(Into::into)
            } else {
                Err(Error::KeplrDisabled)
            }
        });

        Self { enabled, key }
    }
}
