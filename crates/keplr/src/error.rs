use serde::{Deserialize, Serialize};
use web_sys::{js_sys, wasm_bindgen};

#[derive(thiserror::Error, Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Error {
    #[error("Keplr is unavailable!")]
    KeplrUnavailable,

    #[error("{0}")]
    Js(String),

    #[error("Serialization Error: {0}")]
    Serialization(String),

    #[error("{0}")]
    Generic(String),
}

impl Error {
    pub fn js(value: wasm_bindgen::JsValue) -> Self {
        value.into()
    }
    pub fn generic(value: impl std::fmt::Display) -> Self {
        Self::Generic(value.to_string())
    }
}

impl From<wasm_bindgen::JsValue> for Error {
    fn from(error: wasm_bindgen::JsValue) -> Self {
        let message = js_sys::Error::from(error)
            .message()
            .as_string()
            .unwrap_or("unknown JS error".to_string());
        Error::Js(message)
    }
}

impl From<serde_wasm_bindgen::Error> for Error {
    fn from(error: serde_wasm_bindgen::Error) -> Self {
        let message = error.to_string();
        Error::Serialization(message)
    }
}

impl From<&str> for Error {
    fn from(value: &str) -> Self {
        Self::Generic(value.to_string())
    }
}

impl From<String> for Error {
    fn from(value: String) -> Self {
        Self::Generic(value)
    }
}
