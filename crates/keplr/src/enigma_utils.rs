use super::Error;
use async_trait::async_trait;
use secretrs::utils::{encryption::SecretUtils, Error as EncryptionError};
use send_wrapper::SendWrapper;
use serde::Serialize;
use tracing::{debug, trace};
use web_sys::js_sys::Uint8Array;

#[derive(Debug)]
pub struct EnigmaUtils {
    inner: SendWrapper<keplr_sys::EnigmaUtils>,
}

impl Clone for EnigmaUtils {
    fn clone(&self) -> Self {
        EnigmaUtils {
            inner: SendWrapper::new(self.inner.clone().into()),
        }
    }
}

impl From<keplr_sys::EnigmaUtils> for EnigmaUtils {
    fn from(value: keplr_sys::EnigmaUtils) -> Self {
        Self {
            inner: SendWrapper::new(value),
        }
    }
}

#[async_trait(?Send)]
impl SecretUtils for EnigmaUtils {
    async fn encrypt<M: Serialize + Sync>(
        &self,
        contract_code_hash: &str,
        msg: &M,
    ) -> Result<Vec<u8>, EncryptionError> {
        let msg = serde_wasm_bindgen::to_value(msg).expect("wasm_bindgen error");

        let result = self
            .inner
            .encrypt(contract_code_hash.to_string(), msg)
            .await
            .map_err(Error::js)
            .map_err(|error| EncryptionError::Other(error.to_string()))?;

        let result = Uint8Array::new(&result);

        Ok(result.to_vec())
    }

    async fn decrypt(
        &self,
        nonce: &[u8; 32],
        ciphertext: &[u8],
    ) -> Result<Vec<u8>, EncryptionError> {
        // NOTE: the order of inputs is reversed in Keplr's decrypt method.
        let result = self
            .inner
            .decrypt(ciphertext, nonce)
            .await
            .map_err(Error::js)
            .map_err(|error| EncryptionError::Other(error.to_string()))?;

        let result = Uint8Array::new(&result);

        Ok(result.to_vec())
    }

    async fn get_pubkey(&self) -> [u8; 32] {
        let key = self.inner.get_pubkey().await;
        trace!("{:?}", key);
        let key = Uint8Array::try_from(key).unwrap();
        let mut array = [0u8; 32];
        key.copy_to(&mut array);
        trace!("{:?}", array);
        array
    }

    async fn get_tx_encryption_key(&self, nonce: &[u8; 32]) -> [u8; 32] {
        let key = self.inner.get_tx_encryption_key(nonce).await;
        trace!("{:?}", key);
        let key = Uint8Array::try_from(key).unwrap();
        let mut array = [0u8; 32];
        key.copy_to(&mut array);
        trace!("{:?}", array);
        array
    }
}
