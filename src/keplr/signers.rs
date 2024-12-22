use super::Error;
use async_trait::async_trait;
use rsecret::wallet::{
    wallet_amino::{AccountData, AminoSignResponse, StdSignDoc},
    wallet_proto::{DirectSignResponse, SignDoc},
    Error as SignerError, Signer,
};
use secretrs::tx::SignMode;
use send_wrapper::SendWrapper;
use serde::{de::DeserializeOwned, Serialize};
use serde_json::{Map, Value};
use std::rc::Rc;
use tracing::{debug, trace};
use web_sys::js_sys;

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

#[async_trait(?Send)]
impl Signer for KeplrOfflineSigner {
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

        let js_result = self
            .inner
            .sign_direct(signer_address.into(), sign_doc)
            .await
            .map(|js_value| {
                serde_wasm_bindgen::from_value::<DirectSignResponse>(js_value)
                    .expect("Problem deserializing DirectSignResponse")
            })
            .map_err(Error::js)
            .map_err(SignerError::custom);

        debug!("{:?}", js_result);

        js_result
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
            .map_err(Error::js)
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

/// Sorts a JSON object by its keys recursively.
fn sort_object(value: &Value) -> Value {
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
fn serialize_std_sign_doc<T: Serialize>(sign_doc: &StdSignDoc<T>) -> Vec<u8> {
    let value = serde_json::to_value(sign_doc).unwrap();
    json_sorted_stringify(&value).as_bytes().to_vec()
}
