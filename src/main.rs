use leptos::prelude::*;
use secret_leptos::App;
use tracing_subscriber::fmt;
use tracing_subscriber_wasm::MakeConsoleWriter;
use web_sys::{js_sys, wasm_bindgen::JsValue};

fn main() {
    fmt()
        .with_writer(MakeConsoleWriter::default().map_trace_level_to(tracing::Level::DEBUG))
        .with_max_level(tracing::Level::DEBUG)
        .without_time()
        .init();
    console_error_panic_hook::set_once();

    // set global variable to signal the wasm is starting
    let window = window();
    let _ = js_sys::Reflect::set(
        &window,
        &JsValue::from_str("myWasmIsReady"),
        &JsValue::from_bool(true),
    );

    mount_to_body(|| view! { <App /> });
}
