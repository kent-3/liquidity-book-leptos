use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ChainInfo {
    pub chain_id: String,
    pub chain_name: String,
    pub rpc: String,
    pub rest: String,
    pub bip44: Bip44,
    pub bech32_config: Bech32Config,
    pub currencies: Vec<Currency>,
    pub fee_currencies: Vec<FeeCurrency>,
    pub stake_currency: Currency,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Bip44 {
    pub coin_type: u32,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Bech32Config {
    pub bech32_prefix_acc_addr: String,
    pub bech32_prefix_acc_pub: String,
    pub bech32_prefix_val_addr: String,
    pub bech32_prefix_val_pub: String,
    pub bech32_prefix_cons_addr: String,
    pub bech32_prefix_cons_pub: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Currency {
    pub coin_denom: String,
    pub coin_minimal_denom: String,
    pub coin_decimals: u8,
    pub coin_gecko_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FeeCurrency {
    pub coin_denom: String,
    pub coin_minimal_denom: String,
    pub coin_decimals: u8,
    pub coin_gecko_id: String,
    pub gas_price_step: GasPriceStep,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GasPriceStep {
    pub low: f64,
    pub average: f64,
    pub high: f64,
}
