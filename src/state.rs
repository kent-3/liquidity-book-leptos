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

#[derive(Clone, Debug, PartialEq)]
pub struct ProviderConfig {
    pub url: String,
    pub chain_id: String,
}

impl ProviderConfig {
    pub fn new(url: impl Into<String>, chain_id: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            chain_id: chain_id.into(),
        }
    }
}

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

impl std::ops::Deref for Endpoint {
    type Target = RwSignal<String>;

    fn deref(&self) -> &Self::Target {
        &self.url
    }
}

impl AsRef<RwSignal<String>> for Endpoint {
    fn as_ref(&self) -> &RwSignal<String> {
        &self.url
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ChainId {
    pub chain_id: RwSignal<String>,
}

impl ChainId {
    pub fn new(chain_id: impl Into<String>) -> Self {
        Self {
            chain_id: RwSignal::new(chain_id.into()),
        }
    }
}

impl Default for ChainId {
    fn default() -> Self {
        Self {
            chain_id: RwSignal::new(CHAIN_ID.to_string()),
        }
    }
}

impl std::ops::Deref for ChainId {
    type Target = RwSignal<String>;

    fn deref(&self) -> &Self::Target {
        &self.chain_id
    }
}

impl AsRef<RwSignal<String>> for ChainId {
    fn as_ref(&self) -> &RwSignal<String> {
        &self.chain_id
    }
}

// TODO: decide between this and the LazyLock approach.
// It's not a signal, and should rarely be updated.
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
    // pub key: RwSignal<Option<Result<Key, Error>>>,
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
        // let key = RwSignal::new(None);

        Self { enabled, key }
    }
}
