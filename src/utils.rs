use cosmwasm_std::Uint128;
use leptos::prelude::window;
use rsecret::query::tendermint::TendermintQuerier;
use tonic_web_wasm_client::Client;

use crate::error::Error;

pub fn alert(msg: impl AsRef<str>) {
    let _ = window().alert_with_message(msg.as_ref());
}

pub fn shorten_address(address: impl ToString) -> String {
    let address = address.to_string();
    if address.len() > 15 {
        format!("{}...{}", &address[..10], &address[address.len() - 5..])
    } else {
        address.to_string() // Return the address as is if it's too short to shorten
    }
}

pub fn humanize_token_amount(amount: impl Into<u128>, decimals: impl Into<u32>) -> String {
    let value = amount.into();
    let factor = 10u128.pow(decimals.into());

    let integer_part = value / factor;
    let fractional_part = value % factor;

    format!(
        "{}.{:0width$}",
        integer_part,
        fractional_part,
        width = 3 as usize
    )
}

pub async fn latest_block(tendermint: TendermintQuerier<Client>) -> Result<u64, Error> {
    tendermint
        .get_latest_block()
        .await
        .map(|block| block.header.height.value())
        .map_err(Error::from)
}
