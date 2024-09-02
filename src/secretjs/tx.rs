use js_sys::Object;
use wasm_bindgen::prelude::*;

pub fn create_tx_options(
    gas_limit: Option<u32>,
    gas_price_in_fee_denom: Option<u32>,
    fee_denom: Option<&str>,
    fee_granter: Option<&str>,
    memo: Option<&str>,
) -> JsValue {
    let tx_options = Object::new();

    if let Some(gas_limit) = gas_limit {
        let _ = js_sys::Reflect::set(
            &tx_options,
            &JsValue::from_str("gasLimit"),
            &JsValue::from_f64(gas_limit as f64),
        );
    }

    if let Some(gas_price_in_fee_denom) = gas_price_in_fee_denom {
        let _ = js_sys::Reflect::set(
            &tx_options,
            &JsValue::from_str("gasPriceInFeeDenom"),
            &JsValue::from_f64(gas_price_in_fee_denom as f64),
        );
    }

    if let Some(fee_denom) = fee_denom {
        let _ = js_sys::Reflect::set(
            &tx_options,
            &JsValue::from_str("feeDenom"),
            &JsValue::from_str(fee_denom),
        );
    }

    if let Some(fee_granter) = fee_granter {
        let _ = js_sys::Reflect::set(
            &tx_options,
            &JsValue::from_str("feeGranter"),
            &JsValue::from_str(fee_granter),
        );
    }

    if let Some(memo) = memo {
        let _ = js_sys::Reflect::set(
            &tx_options,
            &JsValue::from_str("memo"),
            &JsValue::from_str(memo),
        );
    }

    tx_options.into()
}

pub struct TxOptionsBuilder {
    gas_limit: Option<u32>,
    gas_price_in_fee_denom: Option<u32>,
    fee_denom: Option<String>,
    fee_granter: Option<String>,
    memo: Option<String>,
}

impl TxOptionsBuilder {
    pub fn new() -> Self {
        TxOptionsBuilder {
            gas_limit: None,
            gas_price_in_fee_denom: None,
            fee_denom: None,
            fee_granter: None,
            memo: None,
        }
    }

    pub fn gas_limit(mut self, gas_limit: u32) -> Self {
        self.gas_limit = Some(gas_limit);
        self
    }

    pub fn gas_price_in_fee_denom(mut self, gas_price_in_fee_denom: u32) -> Self {
        self.gas_price_in_fee_denom = Some(gas_price_in_fee_denom);
        self
    }

    pub fn fee_denom(mut self, fee_denom: String) -> Self {
        self.fee_denom = Some(fee_denom);
        self
    }

    pub fn fee_granter(mut self, fee_granter: String) -> Self {
        self.fee_granter = Some(fee_granter);
        self
    }

    pub fn memo(mut self, memo: String) -> Self {
        self.memo = Some(memo);
        self
    }

    pub fn build(self) -> JsValue {
        let tx_options = Object::new();

        if let Some(gas_limit) = self.gas_limit {
            let _ = js_sys::Reflect::set(
                &tx_options,
                &JsValue::from_str("gasLimit"),
                &JsValue::from(gas_limit),
            );
        }

        if let Some(gas_price_in_fee_denom) = self.gas_price_in_fee_denom {
            let _ = js_sys::Reflect::set(
                &tx_options,
                &JsValue::from_str("gasPriceInFeeDenom"),
                &JsValue::from_f64(gas_price_in_fee_denom as f64),
            );
        }

        if let Some(fee_denom) = self.fee_denom {
            let _ = js_sys::Reflect::set(
                &tx_options,
                &JsValue::from_str("feeDenom"),
                &JsValue::from_str(&fee_denom),
            );
        }

        if let Some(fee_granter) = self.fee_granter {
            let _ = js_sys::Reflect::set(
                &tx_options,
                &JsValue::from_str("feeGranter"),
                &JsValue::from_str(&fee_granter),
            );
        }

        if let Some(memo) = self.memo {
            let _ = js_sys::Reflect::set(
                &tx_options,
                &JsValue::from_str("memo"),
                &JsValue::from_str(&memo),
            );
        }

        tx_options.into()
    }
}
