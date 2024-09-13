/// Configurations for Adding Liquidity Presets
use std::sync::LazyLock;

#[derive(Debug)]
pub struct LiquidityPreset {
    delta_ids: Vec<i32>,
    distribution_x: Vec<f32>,
    distribution_y: Vec<f32>,
}

// 1) Spot (Uniform)
pub const SPOT_UNIFORM: LazyLock<LiquidityPreset> = LazyLock::new(|| LiquidityPreset {
    delta_ids: vec![-5, -4, -3, -2, -1, 0, 1, 2, 3, 4, 5],
    distribution_x: vec![
        0.0, 0.0, 0.0, 0.0, 0.0, 0.090909, 0.181818, 0.181818, 0.181818, 0.181818, 0.181818,
    ],
    distribution_y: vec![
        0.181818, 0.181818, 0.181818, 0.181818, 0.181818, 0.090909, 0.0, 0.0, 0.0, 0.0, 0.0,
    ],
});

// 2) Curve
pub const CURVE: LazyLock<LiquidityPreset> = LazyLock::new(|| LiquidityPreset {
    delta_ids: vec![-5, -4, -3, -2, -1, 0, 1, 2, 3, 4, 5],
    distribution_x: vec![0.0, 0.0, 0.0, 0.0, 0.0, 0.18, 0.3, 0.24, 0.16, 0.08, 0.04],
    distribution_y: vec![0.04, 0.08, 0.16, 0.24, 0.3, 0.18, 0.0, 0.0, 0.0, 0.0, 0.0],
});

// 3) Bid-Ask
pub const BID_ASK: LazyLock<LiquidityPreset> = LazyLock::new(|| LiquidityPreset {
    delta_ids: vec![-5, -4, -3, -2, -1, 0, 1, 2, 3, 4, 5],
    distribution_x: vec![0.0, 0.0, 0.0, 0.0, 0.0, 0.04, 0.12, 0.16, 0.2, 0.24, 0.24],
    distribution_y: vec![0.24, 0.24, 0.2, 0.16, 0.12, 0.04, 0.0, 0.0, 0.0, 0.0, 0.0],
});

// 4) Wide
pub const WIDE: LazyLock<LiquidityPreset> = LazyLock::new(|| LiquidityPreset {
    delta_ids: vec![
        -25, -24, -23, -22, -21, -20, -19, -18, -17, -16, -15, -14, -13, -12, -11, -10, -9, -8, -7,
        -6, -5, -4, -3, -2, -1, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18,
        19, 20, 21, 22, 23, 24, 25,
    ],
    distribution_x: vec![
        0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
        0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0196, 0.0392, 0.0392, 0.0392, 0.0392, 0.0392, 0.0392,
        0.0392, 0.0392, 0.0392, 0.0392, 0.0392, 0.0392, 0.0392, 0.0392, 0.0392, 0.0392, 0.0392,
        0.0392, 0.0392, 0.0392, 0.0392, 0.0392, 0.0392, 0.0392, 0.0392,
    ],
    distribution_y: vec![
        0.0392, 0.0392, 0.0392, 0.0392, 0.0392, 0.0392, 0.0392, 0.0392, 0.0392, 0.0392, 0.0392,
        0.0392, 0.0392, 0.0392, 0.0392, 0.0392, 0.0392, 0.0392, 0.0392, 0.0392, 0.0392, 0.0392,
        0.0392, 0.0392, 0.0392, 0.0196, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
        0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
    ],
});
