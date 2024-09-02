use web_sys::{js_sys, wasm_bindgen};

#[derive(thiserror::Error, serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub enum Error {
    #[error("An error occurred in JavaScript: {0}")]
    JavaScript(String),

    #[error("Serialization Error: {0}")]
    Serialization(String),

    #[error("Keplr is unavailable!")]
    KeplrUnavailable,
}

impl From<wasm_bindgen::JsValue> for Error {
    fn from(error: wasm_bindgen::JsValue) -> Self {
        let message = js_sys::Error::from(error)
            .message()
            .as_string()
            .unwrap_or("unknown JS error".to_string());
        Error::JavaScript(message)
    }
}
impl From<serde_wasm_bindgen::Error> for Error {
    fn from(error: serde_wasm_bindgen::Error) -> Self {
        let message = error.to_string();
        Error::Serialization(message)
    }
}
