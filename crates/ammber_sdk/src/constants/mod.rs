use cosmwasm_std::Uint128;
use std::str::FromStr;

pub mod addrs;
pub mod liquidity_config;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChainId {
    Dev,
    Pulsar,
    Secret,
}

impl ChainId {
    /// Returns the corresponding string for each ChainId
    pub fn as_str(&self) -> &'static str {
        match self {
            ChainId::Dev => "secretdev-1",
            ChainId::Pulsar => "pulsar-3",
            ChainId::Secret => "secret-4",
        }
    }
}

impl FromStr for ChainId {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "secretdev-1" => Ok(ChainId::Dev),
            "pulsar-3" => Ok(ChainId::Pulsar),
            "secret-4" => Ok(ChainId::Secret),
            _ => Err("Invalid chain ID"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TradeType {
    ExactInput,
    ExactOutput,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Rounding {
    RoundDown,
    RoundHalfUp,
    RoundUp,
}

pub const MINIMUM_LIQUIDITY: Uint128 = Uint128::new(1000);
