use super::Error;
use async_trait::async_trait;
use base64::prelude::{Engine as _, BASE64_STANDARD};
use keplr_sys::*;
use rsecret::wallet::*;
use secretrs::tx::SignMode;
use send_wrapper::SendWrapper;
use serde::{Deserialize, Serialize};
use std::rc::Rc;
use tracing::debug;
use web_sys::{
    console,
    js_sys::{self, JsString},
    wasm_bindgen::JsValue,
};

pub use rsecret::wallet::AccountData;

#[derive(Serialize, Deserialize, Clone, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Key {
    /// Name of the selected key store.
    pub name: String,
    pub algo: String,
    pub pub_key: Vec<u8>,
    pub address: Vec<u8>,
    pub bech32_address: String,
    pub ethereum_hex_address: String,
    // Indicate whether the selected account is from the nano ledger.
    // Because current cosmos app in the nano ledger doesn't support the direct (proto) format msgs,
    // this can be used to select the amino or direct signer.
    pub is_nano_ledger: bool,
    pub is_keystone: bool,
}

impl std::fmt::Debug for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Key")
            .field("name", &self.name)
            .field("algo", &self.algo)
            .field("pub_key", &BASE64_STANDARD.encode(&self.pub_key)) // Convert pub_key to base64
            .field("address", &BASE64_STANDARD.encode(&self.address)) // Convert address to base64
            .field("bech32_address", &self.bech32_address)
            .field("ethereum_hex_address", &self.ethereum_hex_address)
            .field("is_nano_ledger", &self.is_nano_ledger)
            .field("is_keystone", &self.is_keystone)
            .finish()
    }
}

pub struct Keplr {}

impl Keplr {
    pub fn debug() {
        KEPLR.with(console::log_1)
    }

    pub fn is_available() -> bool {
        web_sys::window()
            .and_then(|window| js_sys::Reflect::get(&window, &JsValue::from_str("keplr")).ok())
            .map_or(false, |keplr| !keplr.is_undefined() && !keplr.is_null())
    }

    pub async fn ping() -> Result<(), Error> {
        ping().await.map_err(Into::into)
    }

    pub async fn enable(chain_ids: Vec<String>) -> Result<(), Error> {
        enable(chain_ids).await.map_err(Into::into)
    }

    pub async fn get_key(chain_id: &str) -> Result<Key, Error> {
        get_key(chain_id)
            .await
            .and_then(|key| Ok(serde_wasm_bindgen::from_value::<Key>(key)?))
            .map_err(Into::into)
    }

    pub async fn get_account(chain_id: &str) -> Result<AccountData, Error> {
        let signer = Self::get_offline_signer_only_amino(chain_id);
        let accounts = signer.get_accounts().await?;
        // .map_err(|_| Error::KeplrUnavailable)?;
        // let accounts = js_sys::Array::from(&accounts);
        let account = accounts[0].clone();

        // let account: AccountData = serde_wasm_bindgen::from_value(account)?;

        Ok(account)
    }

    pub fn get_offline_signer(chain_id: &str) -> KeplrOfflineSigner {
        get_offline_signer(chain_id).into()
    }

    pub fn get_offline_signer_only_amino(chain_id: &str) -> KeplrOfflineSignerOnlyAmino {
        get_offline_signer_only_amino(chain_id).into()
    }

    // TODO: this doesn't help... idk what to do here
    pub async fn get_offline_signer_auto(
        chain_id: &str,
    ) -> Result<Box<dyn Signer<Error = Error>>, Error> {
        let key = Self::get_key(chain_id).await?;
        let signer: Box<dyn Signer<Error = Error>> = match key.is_nano_ledger {
            true => Box::new(Self::get_offline_signer_only_amino(chain_id)),
            false => Box::new(Self::get_offline_signer(chain_id)),
        };
        Ok(signer)
    }

    pub fn get_enigma_utils(chain_id: &str) -> EnigmaUtils {
        get_enigma_utils(chain_id)
    }

    pub async fn suggest_token(
        chain_id: &str,
        contract_address: &str,
        viewing_key: Option<&str>,
    ) -> Result<(), Error> {
        suggest_token(chain_id, contract_address, viewing_key)
            .await
            .map_err(Into::into)
    }

    pub async fn get_secret_20_viewing_key(
        chain_id: &str,
        contract_address: &str,
    ) -> Result<String, Error> {
        get_secret_20_viewing_key(chain_id, contract_address)
            .await
            .map(|foo| JsString::from(foo).into())
            .map_err(Into::into)
    }

    pub fn disable(chain_id: &str) {
        disable(chain_id)
    }

    pub fn disable_origin() {
        disable_origin()
    }
}

#[derive(Debug, Clone)]
pub struct KeplrOfflineSigner {
    inner: SendWrapper<Rc<keplr_sys::KeplrOfflineSigner>>,
}

impl From<keplr_sys::KeplrOfflineSigner> for KeplrOfflineSigner {
    fn from(value: keplr_sys::KeplrOfflineSigner) -> Self {
        Self {
            inner: SendWrapper::new(Rc::new(value)),
        }
    }
}

#[async_trait]
impl Signer for KeplrOfflineSigner {
    type Error = super::Error;

    // pub fn chain_id(&self) -> String {
    //     self.inner
    //         .chain_id()
    //         .as_string()
    //         .expect("chain_id field is missing!")
    // }

    async fn get_accounts(&self) -> Result<Vec<AccountData>, Self::Error> {
        SendWrapper::new(async move {
            self.inner
                .get_accounts()
                .await
                .map_err(|_| Error::KeplrUnavailable)
                .map(|val| js_sys::Array::from(&val))
                .and_then(|accounts| {
                    accounts
                        .iter()
                        .map(|account| serde_wasm_bindgen::from_value(account).map_err(Into::into))
                        .collect::<Result<Vec<AccountData>, Error>>()
                })
        })
        .await
    }

    async fn get_sign_mode(&self) -> Result<SignMode, Self::Error> {
        Ok(SignMode::Direct)
    }

    async fn sign_amino(
        &self,
        signer_address: &str,
        sign_doc: StdSignDoc,
    ) -> Result<AminoSignResponse, Self::Error> {
        todo!()
    }

    async fn sign_permit(
        &self,
        signer_address: &str,
        sign_doc: StdSignDoc,
    ) -> Result<AminoSignResponse, Self::Error> {
        todo!()
    }

    async fn sign_direct(
        &self,
        signer_address: &str,
        sign_doc: secretrs::tx::SignDoc,
    ) -> Result<DirectSignResponse, Self::Error> {
        let sign_doc: SignDoc = sign_doc.into();
        let sign_doc = serde_wasm_bindgen::to_value(&sign_doc).expect("serde_wasm_bindgen problem");

        SendWrapper::new(async move {
            let js_result = self
                .inner
                .sign_direct(signer_address.into(), sign_doc)
                .await
                .map(|js_value| {
                    serde_wasm_bindgen::from_value::<DirectSignResponse>(js_value)
                        .expect("Problem deserializing DirectSignResponse")
                })
                .map_err(Into::into);

            debug!("{:?}", js_result);
            js_result
        })
        .await
    }
}

#[derive(Debug, Clone)]
pub struct KeplrOfflineSignerOnlyAmino {
    inner: SendWrapper<Rc<keplr_sys::KeplrOfflineSignerOnlyAmino>>,
}

impl From<keplr_sys::KeplrOfflineSignerOnlyAmino> for KeplrOfflineSignerOnlyAmino {
    fn from(value: keplr_sys::KeplrOfflineSignerOnlyAmino) -> Self {
        Self {
            inner: SendWrapper::new(Rc::new(value)),
        }
    }
}

#[async_trait]
impl Signer for KeplrOfflineSignerOnlyAmino {
    type Error = super::Error;

    // pub fn chain_id(&self) -> String {
    //     self.inner
    //         .chain_id()
    //         .as_string()
    //         .expect("chain_id field is missing!")
    // }

    async fn get_accounts(&self) -> Result<Vec<AccountData>, Self::Error> {
        SendWrapper::new(async move {
            self.inner
                .get_accounts()
                .await
                .map_err(|_| Error::KeplrUnavailable)
                .map(|val| js_sys::Array::from(&val))
                .and_then(|accounts| {
                    accounts
                        .iter()
                        .map(|account| serde_wasm_bindgen::from_value(account).map_err(Into::into))
                        .collect::<Result<Vec<AccountData>, Error>>()
                })
        })
        .await
    }

    async fn get_sign_mode(&self) -> Result<SignMode, Self::Error> {
        Ok(SignMode::LegacyAminoJson)
    }

    async fn sign_amino(
        &self,
        signer_address: &str,
        sign_doc: StdSignDoc,
    ) -> Result<AminoSignResponse, Self::Error> {
        todo!()
    }

    async fn sign_permit(
        &self,
        signer_address: &str,
        sign_doc: StdSignDoc,
    ) -> Result<AminoSignResponse, Self::Error> {
        todo!()
    }

    async fn sign_direct(
        &self,
        signer_address: &str,
        sign_doc: secretrs::tx::SignDoc,
    ) -> Result<DirectSignResponse, Self::Error> {
        unimplemented!()
    }
}

pub mod suggest_chain_types {
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct SuggestingChainInfo {
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
}
