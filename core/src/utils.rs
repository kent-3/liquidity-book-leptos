use crate::{
    constants::Token,
    error::Error,
    support::{chain_query, COMPUTE_QUERIER},
    TOKEN_MAP,
};
use cosmwasm_std::{Addr, ContractInfo};
use leptos::prelude::window;

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

pub fn get_token_decimals(address: &str) -> Result<u8, Error> {
    TOKEN_MAP
        .get(address)
        .map(|token| token.decimals)
        .ok_or(Error::UnknownToken)
}

// pub fn display_token_amount(amount: impl Into<u128>, decimals: impl Into<u32>) -> String {
//     let value = amount.into();
//     let factor = 10u128.pow(decimals.into());
//
//     let integer_part = value / factor;
//     let fractional_part = value % factor;
//
//     format!(
//         "{}.{:0width$}",
//         integer_part,
//         fractional_part,
//         width = 3 as usize
//     )
// }
//
// pub fn parse_token_amount(amount: &str, decimals: u32) -> u128 {
//     let factor = 10u128.pow(decimals);
//     let parsed: f64 = amount.parse().unwrap_or(0.0);
//
//     let scaled = parsed * factor as f64;
//     scaled.round() as u128
// }

pub fn display_token_amount(amount: impl Into<u128>, decimals: impl Into<u32>) -> String {
    let amount = amount.into();
    let decimals = decimals.into();
    let factor = 10u128.pow(decimals);

    let integer_part = amount / factor;
    let fractional_part = amount % factor;

    if decimals == 0 {
        return integer_part.to_string();
    }

    // Adjust the width dynamically based on decimals
    let fractional_str = format!("{:0width$}", fractional_part, width = decimals as usize);

    format!("{}.{}", integer_part, fractional_str)

    // Trim trailing zeros for a cleaner display
    // let trimmed_fractional = fractional_str.trim_end_matches('0');
    //
    // if trimmed_fractional.is_empty() {
    //     integer_part.to_string()
    // } else {
    //     format!("{}.{}", integer_part, trimmed_fractional)
    // }
}

pub fn parse_token_amount(amount: impl AsRef<str>, decimals: impl Into<u32>) -> u128 {
    let amount = amount.as_ref();
    let decimals = decimals.into();
    let factor = 10u128.pow(decimals);

    // Split by '.' to manually handle the fractional part
    let parts: Vec<&str> = amount.split('.').collect();
    let whole_part: u128 = parts[0].parse().unwrap_or(0);

    let fractional_part: u128 = if parts.len() > 1 {
        let mut decimal_str = parts[1].to_string();

        // Pad or truncate the fractional part to match the precision
        if decimal_str.len() > decimals as usize {
            decimal_str.truncate(decimals as usize);
        } else {
            decimal_str.push_str(&"0".repeat(decimals as usize - decimal_str.len()));
        }
        decimal_str.parse().unwrap_or(0)
    } else {
        0
    };

    whole_part * factor + fractional_part
}

// TODO: utilites that involve chain queries should probably go somewhere else

pub async fn addr_2_contract(contract_address: impl Into<String>) -> Result<ContractInfo, Error> {
    let contract_address = contract_address.into();

    if let Some(token) = TOKEN_MAP.get(&contract_address) {
        Ok(ContractInfo {
            address: Addr::unchecked(token.contract_address.clone()),
            code_hash: token.code_hash.clone(),
        })
    } else {
        COMPUTE_QUERIER
            .code_hash_by_contract_address(&contract_address)
            .await
            .map_err(Error::from)
            .map(|code_hash| ContractInfo {
                address: Addr::unchecked(contract_address),
                code_hash,
            })
    }
}

pub async fn addr_2_symbol(address: String) -> String {
    if let Some(token) = TOKEN_MAP.get(&address) {
        if let Some(ref display_name) = token.display_name {
            return display_name.clone();
        } else {
            return token.symbol.clone();
        }
    }
    let contract = addr_2_contract(&address).await.unwrap();

    chain_query::<secret_toolkit_snip20::TokenInfoResponse>(
        contract.address.to_string(),
        contract.code_hash,
        secret_toolkit_snip20::QueryMsg::TokenInfo {},
    )
    .await
    .map(|x| x.token_info.symbol)
    .unwrap_or(address)
}

/// Queries the chain for the code hash and token info if not in the token map.
pub async fn addr_2_token(address: impl Into<String>) -> Token {
    let contract_address = address.into();

    if let Some(token) = TOKEN_MAP.get(&contract_address) {
        return token.clone();
    }

    let code_hash = COMPUTE_QUERIER
        .code_hash_by_contract_address(&contract_address)
        .await
        .expect("code_hash_by_contract_address query failed");

    let token_info = chain_query::<secret_toolkit_snip20::TokenInfoResponse>(
        contract_address.clone(),
        code_hash.clone(),
        secret_toolkit_snip20::QueryMsg::TokenInfo {},
    )
    .await
    .map(|response| response.token_info)
    .expect("token_info query failed");

    Token {
        contract_address,
        code_hash,
        decimals: token_info.decimals,
        name: token_info.name,
        symbol: token_info.symbol,
        display_name: None,
        denom: None,
        version: None,
    }
}
