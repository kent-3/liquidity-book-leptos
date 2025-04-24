use ethnum::U256;
use liquidity_book::libraries::{constants::PRECISION, price_helper::PriceHelper};
use tracing::debug;

pub fn get_id_from_price(price: f64, bin_step: u16) -> u32 {
    let price = price * PRECISION as f64;
    let price = U256::from(price as u128);
    let price = PriceHelper::convert_decimal_price_to128x128(price).unwrap();
    PriceHelper::get_id_from_price(price, bin_step).unwrap()
}

pub fn get_price_from_id(id: String, bin_step: u16) -> f64 {
    let id = id.parse::<u32>().unwrap();
    let bin_step = bin_step as f64;
    (1.0 + bin_step / 10_000.0).powf((id as f64) - 8_388_608.0)
}

pub fn parse_to_basis_points(value: &str) -> u32 {
    let parsed: f64 = value.parse().unwrap_or(0.0);
    let basis_points = parsed * 100.0;
    basis_points.round() as u32
}

pub fn parse_to_decimal_price(value: &str) -> U256 {
    let parts: Vec<&str> = value.split('.').collect();
    let whole_part: u128 = parts[0].parse().unwrap_or(0);
    let decimal_part: u128 = if parts.len() > 1 {
        let decimal_str = format!("{:0<18}", parts[1]); // Pad to 18 decimals
        decimal_str[..18.min(decimal_str.len())]
            .parse()
            .unwrap_or(0)
    } else {
        0
    };

    U256::from(whole_part * PRECISION + decimal_part)
}

pub fn u128_to_string_with_precision(value: u128) -> String {
    let whole_part = value / PRECISION;
    let decimal_part = value % PRECISION;

    // Scale the decimal_part to 6 decimal places (instead of 18)
    // PRECISION is 10^18, so divide by 10^12 to get 6 decimal places
    let scaled_decimal = decimal_part / 1_000_000_000_000;

    // Format to exactly 6 decimal places
    let decimal_str = format!("{:06}", scaled_decimal);

    // Trim trailing zeros
    let trimmed_decimal = decimal_str.trim_end_matches('0');

    if trimmed_decimal.is_empty() {
        whole_part.to_string()
    } else {
        format!("{}.{}", whole_part, trimmed_decimal)
    }
}
