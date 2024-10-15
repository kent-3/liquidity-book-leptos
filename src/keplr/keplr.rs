use super::Error;
use async_trait::async_trait;
use base64::prelude::{Engine as _, BASE64_STANDARD};
use keplr_sys::*;
use rsecret::{
    secret_client::TBD,
    wallet::{
        wallet_amino::{AccountData, AminoSignResponse, StdSignDoc},
        wallet_proto::{DirectSignResponse, SignDoc},
        Signer,
    },
};
use secretrs::tx::SignMode;
use secretrs::utils::encryption::SecretUtils;
use send_wrapper::SendWrapper;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::{Map, Value};
use std::rc::Rc;
use tracing::debug;
use web_sys::{
    console,
    js_sys::{self, JsString},
    wasm_bindgen::JsValue,
};

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
        let accounts = signer.get_accounts().await.map_err(Error::generic)?;
        let account = accounts[0].clone();

        Ok(account)
    }

    pub fn get_offline_signer(chain_id: &str) -> KeplrOfflineSigner {
        get_offline_signer(chain_id).into()
    }

    pub fn get_offline_signer_only_amino(chain_id: &str) -> KeplrOfflineSignerOnlyAmino {
        get_offline_signer_only_amino(chain_id).into()
    }

    // TODO: not sure if this is correct
    // pub async fn get_offline_signer_auto(chain_id: &str) -> Result<Box<dyn Signer>, Error> {
    //     let key = Self::get_key(chain_id).await?;
    //     let signer: Box<dyn Signer> = match key.is_nano_ledger {
    //         true => Box::new(Self::get_offline_signer_only_amino(chain_id)),
    //         false => Box::new(Self::get_offline_signer(chain_id)),
    //     };
    //     Ok(signer)
    // }

    pub fn get_enigma_utils(chain_id: &str) -> EnigmaUtils {
        get_enigma_utils(chain_id).into()
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

use rsecret::wallet::Error as SignerError;

#[async_trait(?Send)]
impl Signer for KeplrOfflineSigner {
    // pub fn chain_id(&self) -> String {
    //     self.inner
    //         .chain_id()
    //         .as_string()
    //         .expect("chain_id field is missing!")
    // }

    async fn get_accounts(&self) -> Result<Vec<AccountData>, SignerError> {
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
        .map_err(SignerError::custom)
    }

    async fn get_sign_mode(&self) -> Result<SignMode, SignerError> {
        Ok(SignMode::Direct)
    }

    async fn sign_amino<T: Serialize + DeserializeOwned + Send + Sync>(
        &self,
        signer_address: &str,
        sign_doc: StdSignDoc<T>,
    ) -> Result<AminoSignResponse<T>, SignerError> {
        todo!()
    }

    async fn sign_permit<T: Serialize + DeserializeOwned + Send + Sync>(
        &self,
        signer_address: &str,
        sign_doc: StdSignDoc<T>,
    ) -> Result<AminoSignResponse<T>, SignerError> {
        todo!()
    }

    async fn sign_direct(
        &self,
        signer_address: &str,
        sign_doc: secretrs::tx::SignDoc,
    ) -> Result<DirectSignResponse, SignerError> {
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
                .map_err(Error::javascript)
                .map_err(SignerError::custom);

            debug!("{:?}", js_result);
            js_result
        })
        .await
    }
}

/// Sorts a JSON object by its keys recursively.
pub(crate) fn sort_object(value: &Value) -> Value {
    match value {
        Value::Object(map) => {
            let mut sorted_map = Map::new();
            for (key, val) in map {
                sorted_map.insert(key.clone(), sort_object(val));
            }
            Value::Object(sorted_map)
        }
        Value::Array(vec) => Value::Array(vec.iter().map(sort_object).collect()),
        _ => value.clone(),
    }
}

/// Returns a JSON string with objects sorted by key, used for Amino signing.
fn json_sorted_stringify(value: &Value) -> String {
    serde_json::to_string(&sort_object(value)).unwrap()
}

/// Serializes a `StdSignDoc` object to a sorted and UTF-8 encoded JSON string
pub(crate) fn serialize_std_sign_doc<T: Serialize>(sign_doc: &StdSignDoc<T>) -> Vec<u8> {
    let value = serde_json::to_value(sign_doc).unwrap();
    json_sorted_stringify(&value).as_bytes().to_vec()
}

#[derive(Debug, Clone)]
pub struct EnigmaUtils {
    inner: SendWrapper<Rc<keplr_sys::EnigmaUtils>>,
}

impl From<keplr_sys::EnigmaUtils> for EnigmaUtils {
    fn from(value: keplr_sys::EnigmaUtils) -> Self {
        Self {
            inner: SendWrapper::new(Rc::new(value)),
        }
    }
}

// use futures::executor::block_on;
use js_sys::Uint8Array;
use leptos::{prelude::logging::console_log, wasm_bindgen::JsCast};
use secretrs::utils::Error as EncryptionError;

#[async_trait(?Send)]
impl SecretUtils for EnigmaUtils {
    async fn encrypt<M: Serialize + Sync>(
        &self,
        contract_code_hash: &str,
        msg: &M,
    ) -> Result<Vec<u8>, EncryptionError> {
        let msg = serde_wasm_bindgen::to_value(msg).expect("wasm_bindgen error");

        let result = SendWrapper::new({
            async move {
                self.inner
                    .encrypt(contract_code_hash.to_string(), msg)
                    .await
            }
        })
        .await;

        let result = Uint8Array::new(&result.expect("problem encrypting with Keplr enigmaUtils"));

        Ok(result.to_vec())
    }

    async fn decrypt(
        &self,
        nonce: &[u8; 32],
        ciphertext: &[u8],
    ) -> Result<Vec<u8>, EncryptionError> {
        let result =
        // NOTE: the order of inputs is reversed in Keplr's decrypt method.
            SendWrapper::new({ async move { self.inner.decrypt(ciphertext, nonce).await } }).await;

        let result = Uint8Array::new(&result.expect("problem decrypting with Keplr enigmaUtils"));

        Ok(result.to_vec())
    }

    async fn get_pubkey(&self) -> [u8; 32] {
        let key = self.inner.get_pubkey().await;
        debug!("{:?}", key);
        let key = Uint8Array::try_from(key).unwrap();
        let mut array = [0u8; 32];
        key.copy_to(&mut array);
        debug!("{:?}", array);
        array
    }

    async fn get_tx_encryption_key(&self, nonce: &[u8; 32]) -> [u8; 32] {
        let key = self.inner.get_tx_encryption_key(nonce).await;
        debug!("{:?}", key);
        let key = Uint8Array::try_from(key).unwrap();
        let mut array = [0u8; 32];
        key.copy_to(&mut array);
        debug!("{:?}", array);
        array
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

#[async_trait(?Send)]
impl Signer for KeplrOfflineSignerOnlyAmino {
    // pub fn chain_id(&self) -> String {
    //     self.inner
    //         .chain_id()
    //         .as_string()
    //         .expect("chain_id field is missing!")
    // }

    async fn get_accounts(&self) -> Result<Vec<AccountData>, SignerError> {
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
            .map_err(SignerError::custom)
    }

    async fn get_sign_mode(&self) -> Result<SignMode, SignerError> {
        Ok(SignMode::LegacyAminoJson)
    }

    async fn sign_amino<T: Serialize + DeserializeOwned + Send + Sync>(
        &self,
        signer_address: &str,
        sign_doc: StdSignDoc<T>,
    ) -> Result<AminoSignResponse<T>, SignerError> {
        let sign_doc = serde_wasm_bindgen::to_value(&sign_doc).expect("serde_wasm_bindgen problem");
        debug!("{:#?}", sign_doc);
        let js_result = self
            .inner
            .sign_amino(signer_address.into(), sign_doc)
            .await
            .map(|js_value| {
                serde_wasm_bindgen::from_value::<AminoSignResponse<T>>(js_value)
                    .expect("Problem deserializing AminoSignResponse")
            })
            .map_err(Error::javascript)
            .map_err(SignerError::custom);

        js_result
    }

    async fn sign_permit<T: Serialize + DeserializeOwned + Send + Sync>(
        &self,
        signer_address: &str,
        sign_doc: StdSignDoc<T>,
    ) -> Result<AminoSignResponse<T>, SignerError> {
        todo!()
    }

    async fn sign_direct(
        &self,
        signer_address: &str,
        sign_doc: secretrs::tx::SignDoc,
    ) -> Result<DirectSignResponse, SignerError> {
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
