use cosmwasm_std::Uint128;
use leptos::prelude::window;

pub fn alert(msg: impl AsRef<str>) {
    let _ = window().alert_with_message(msg.as_ref());
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
