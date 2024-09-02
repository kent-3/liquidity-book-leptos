use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[derive(Debug)]
    pub type Wallet;

    #[wasm_bindgen(constructor, js_namespace = ["window", "secretjs"])]
    pub fn new() -> Wallet;
}
