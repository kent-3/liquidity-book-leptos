pub fn get_id_from_price(price: f64, bin_step: impl Into<f64>) -> u32 {
    ((price.ln() / (1.0 + bin_step.into() / 10_000.0).ln()).trunc() as u32) + 8_388_608
}

pub fn get_price_from_id(id: String, bin_step: String) -> f64 {
    let id = id.parse::<u32>().unwrap();
    let bin_step = bin_step.parse::<f64>().unwrap();
    (1.0 + bin_step / 10_000.0).powf((id as f64) - 8_388_608.0)
}
