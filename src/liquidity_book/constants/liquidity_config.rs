/// Configurations for Adding Liquidity Presets
use std::sync::LazyLock;

use super::super::curves::configure_liquidity_by_radius;
use cosmwasm_std::Uint64;
use leptos::attr::IntoAttributeValue;
use liquidity_book::libraries::math::liquidity_configurations::PRECISION;

// TODO: decide if we keep 10^18 PRECISION or use the token decimal as PRECISION
// 10^18 is because ethereum. Token decimals could work, but requires an extra step. And I'm not
// sure if that would affect the contract implementation (I don't think so).
// UPDATE: the distributions are expressed as a percentage, so the precision can be static. Let's
// keep it 10^18 to match the original.

#[derive(Debug, Clone, PartialEq)]
pub struct LiquidityConfigurations {
    delta_ids: Vec<i64>,
    distribution_x: Vec<f64>,
    distribution_y: Vec<f64>,
}

impl LiquidityConfigurations {
    pub fn new(delta_ids: Vec<i64>, distribution_x: Vec<f64>, distribution_y: Vec<f64>) -> Self {
        Self {
            delta_ids,
            distribution_x,
            distribution_y,
        }
    }

    pub fn delta_ids(&self) -> Vec<i64> {
        self.delta_ids.clone()
    }

    pub fn distribution_x(&self) -> Vec<Uint64> {
        self.distribution_x
            .iter()
            .map(|el| (el * PRECISION as f64).trunc() as u64)
            .map(Uint64::new)
            .collect()
    }

    pub fn distribution_y(&self) -> Vec<Uint64> {
        self.distribution_y
            .iter()
            .map(|el| (el * PRECISION as f64).trunc() as u64)
            .map(Uint64::new)
            .collect()
    }
}

impl LiquidityConfigurations {
    pub fn by_radius(
        target_bin: u32,
        radius: u32,
        shape: LiquidityShape,
    ) -> LiquidityConfigurations {
        configure_liquidity_by_radius(target_bin, radius, shape)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum LiquidityShape {
    SpotUniform,
    Curve,
    BidAsk,
    Wide,
}

impl IntoAttributeValue for LiquidityShape {
    type Output = String;
    fn into_attribute_value(self) -> Self::Output {
        self.to_string()
    }
}

impl From<&str> for LiquidityShape {
    fn from(value: &str) -> Self {
        match value {
            "uniform" => LiquidityShape::SpotUniform,
            "curve" => LiquidityShape::Curve,
            "bid-ask" => LiquidityShape::BidAsk,
            "wide" => LiquidityShape::Wide,
            _ => panic!("Invalid liquidity shape"), // can handle this with Result or Option instead
        }
    }
}

impl From<String> for LiquidityShape {
    fn from(value: String) -> Self {
        match value.as_str() {
            "uniform" => LiquidityShape::SpotUniform,
            "curve" => LiquidityShape::Curve,
            "bid-ask" => LiquidityShape::BidAsk,
            "wide" => LiquidityShape::Wide,
            _ => panic!("Invalid liquidity shape"), // can handle this with Result or Option instead
        }
    }
}

impl ToString for LiquidityShape {
    fn to_string(&self) -> String {
        match self {
            LiquidityShape::SpotUniform => "uniform".to_string(),
            LiquidityShape::Curve => "curve".to_string(),
            LiquidityShape::BidAsk => "bid-ask".to_string(),
            LiquidityShape::Wide => "wide".to_string(),
        }
    }
}

// 1) Spot (Uniform)
pub const SPOT_UNIFORM: LazyLock<LiquidityConfigurations> =
    LazyLock::new(|| LiquidityConfigurations {
        delta_ids: vec![-5, -4, -3, -2, -1, 0, 1, 2, 3, 4, 5],
        distribution_x: vec![
            0.0, 0.0, 0.0, 0.0, 0.0, 0.090909, 0.181818, 0.181818, 0.181818, 0.181818, 0.181818,
        ],
        distribution_y: vec![
            0.181818, 0.181818, 0.181818, 0.181818, 0.181818, 0.090909, 0.0, 0.0, 0.0, 0.0, 0.0,
        ],
    });

// 2) Curve
pub const CURVE: LazyLock<LiquidityConfigurations> = LazyLock::new(|| LiquidityConfigurations {
    delta_ids: vec![-5, -4, -3, -2, -1, 0, 1, 2, 3, 4, 5],
    distribution_x: vec![0.0, 0.0, 0.0, 0.0, 0.0, 0.18, 0.3, 0.24, 0.16, 0.08, 0.04],
    distribution_y: vec![0.04, 0.08, 0.16, 0.24, 0.3, 0.18, 0.0, 0.0, 0.0, 0.0, 0.0],
});

// 3) Bid-Ask
pub const BID_ASK: LazyLock<LiquidityConfigurations> = LazyLock::new(|| LiquidityConfigurations {
    delta_ids: vec![-5, -4, -3, -2, -1, 0, 1, 2, 3, 4, 5],
    distribution_x: vec![0.0, 0.0, 0.0, 0.0, 0.0, 0.04, 0.12, 0.16, 0.2, 0.24, 0.24],
    distribution_y: vec![0.24, 0.24, 0.2, 0.16, 0.12, 0.04, 0.0, 0.0, 0.0, 0.0, 0.0],
});

// 4) Wide
pub const WIDE: LazyLock<LiquidityConfigurations> = LazyLock::new(|| LiquidityConfigurations {
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
