use crate::{constants::*, error::Error};
use keplr::{tokens::KeplrToken, Keplr, Key};
use leptos::prelude::*;
use reactive_stores::{Field, Store};
use std::sync::Arc;
use std::{collections::HashMap, ops::Deref};
use tracing::{debug, trace};

// #[derive(Clone, Debug, PartialEq, Store)]
// pub struct ProviderConfig {
//     pub url: String,
//     pub chain_id: String,
// }
//
// impl ProviderConfig {
//     pub fn new(url: impl Into<String>, chain_id: impl Into<String>) -> Self {
//         Self {
//             url: url.into(),
//             chain_id: chain_id.into(),
//         }
//     }
// }

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Endpoint {
    pub url: RwSignal<&'static str>,
}

impl Endpoint {
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: RwSignal::new(Box::leak(url.into().into_boxed_str())),
        }
    }
}

impl Default for Endpoint {
    fn default() -> Self {
        Self {
            url: RwSignal::new(NODE),
        }
    }
}

impl Deref for Endpoint {
    type Target = RwSignal<&'static str>;
    fn deref(&self) -> &Self::Target {
        &self.url
    }
}

impl AsRef<RwSignal<&'static str>> for Endpoint {
    fn as_ref(&self) -> &RwSignal<&'static str> {
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

impl Deref for ChainId {
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
// UPDATE: We can do both. Have a static compiled one to use as a base, and one that can be added
// to at runtime.
#[derive(Clone, Debug)]
pub struct TokenMap(pub Arc<HashMap<String, Token>>);

impl TokenMap {
    pub fn new(token_map: Arc<HashMap<String, Token>>) -> Self {
        Self(token_map)
    }
}

impl Deref for TokenMap {
    type Target = HashMap<String, Token>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<HashMap<String, Token>> for TokenMap {
    fn as_ref(&self) -> &HashMap<String, Token> {
        &self.0
    }
}

// TODO: probably should change this. I'm not sure the derived signal works as intended
#[derive(Copy, Clone)]
pub struct KeplrSignals {
    pub enabled: RwSignal<bool>,
    pub key: AsyncDerived<Result<Key, Error>, LocalStorage>,
    // pub key: RwSignal<Option<Result<Key, Error>>>,
}

// TODO: use runtime chain_id instead of static
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
