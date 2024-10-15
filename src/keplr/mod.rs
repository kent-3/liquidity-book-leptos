use base64::prelude::{Engine as _, BASE64_STANDARD};
use rsecret::wallet::{wallet_amino::AccountData, Signer};
use serde::{Deserialize, Serialize};
use tracing::{debug, trace};
use web_sys::{console, js_sys, wasm_bindgen::JsValue};

mod enigma_utils;
mod error;
pub mod experimental;
mod signers;
pub mod tokens;

pub use enigma_utils::EnigmaUtils;
pub use error::Error;
pub use signers::{KeplrOfflineSigner, KeplrOfflineSignerOnlyAmino};

pub struct Keplr {}

impl Keplr {
    pub fn debug() {
        keplr_sys::KEPLR.with(console::log_1)
    }

    pub fn is_available() -> bool {
        web_sys::window()
            .and_then(|window| js_sys::Reflect::get(&window, &JsValue::from_str("keplr")).ok())
            .map_or(false, |keplr| !keplr.is_undefined() && !keplr.is_null())
    }

    pub async fn ping() -> Result<(), Error> {
        keplr_sys::ping().await.map_err(Error::js)
    }

    pub async fn enable(chain_ids: Vec<String>) -> Result<(), Error> {
        keplr_sys::enable(chain_ids).await.map_err(Error::js)
    }

    pub async fn get_key(chain_id: &str) -> Result<Key, Error> {
        keplr_sys::get_key(chain_id)
            .await
            .and_then(|key| Ok(serde_wasm_bindgen::from_value::<Key>(key)?))
            .map_err(Error::js)
    }

    pub async fn get_account(chain_id: &str) -> Result<AccountData, Error> {
        let signer = Self::get_offline_signer_only_amino(chain_id);
        let accounts = signer.get_accounts().await.map_err(Error::generic)?;
        let account = accounts[0].clone();

        Ok(account)
    }

    pub fn get_offline_signer(chain_id: &str) -> KeplrOfflineSigner {
        keplr_sys::get_offline_signer(chain_id).into()
    }

    pub fn get_offline_signer_only_amino(chain_id: &str) -> KeplrOfflineSignerOnlyAmino {
        keplr_sys::get_offline_signer_only_amino(chain_id).into()
    }

    // TODO: Not sure how to do this. `match` arms have incompatible types
    // pub async fn get_offline_signer_auto<T: Signer>(chain_id: &str) -> Result<T, Error> {
    //     let key = Self::get_key(chain_id).await?;
    //     let signer = match key.is_nano_ledger {
    //         true => Self::get_offline_signer_only_amino(chain_id),
    //         false => Self::get_offline_signer(chain_id),
    //     };
    //     Ok(signer)
    // }

    pub fn get_enigma_utils(chain_id: &str) -> EnigmaUtils {
        keplr_sys::get_enigma_utils(chain_id).into()
    }

    pub async fn suggest_token(
        chain_id: &str,
        contract_address: &str,
        viewing_key: Option<&str>,
    ) -> Result<(), Error> {
        keplr_sys::suggest_token(chain_id, contract_address, viewing_key)
            .await
            .map_err(Error::js)
    }

    pub async fn get_secret_20_viewing_key(
        chain_id: &str,
        contract_address: &str,
    ) -> Result<String, Error> {
        keplr_sys::get_secret_20_viewing_key(chain_id, contract_address)
            .await
            .map(|key| js_sys::JsString::from(key).into())
            .map_err(Error::js)
    }

    pub fn disable(chain_id: &str) {
        keplr_sys::disable(chain_id)
    }

    pub fn disable_origin() {
        keplr_sys::disable_origin()
    }

    pub async fn suggest_chain(chain_info: experimental::ChainInfo) -> Result<(), Error> {
        keplr_sys::suggest_chain(serde_wasm_bindgen::to_value(&chain_info)?)
            .await
            .map_err(Error::js)
    }
}

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
