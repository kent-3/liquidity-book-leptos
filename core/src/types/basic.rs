use serde::{Deserialize, Serialize};

// TODO: include the decimals somehow, and use that in the Display trait
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Coin {
    pub denom: String,
    pub amount: String,
}

impl From<secretrs::proto::cosmos::base::v1beta1::Coin> for Coin {
    fn from(value: secretrs::proto::cosmos::base::v1beta1::Coin) -> Self {
        Self {
            denom: value.denom,
            amount: value.amount,
        }
    }
}

impl std::fmt::Display for Coin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.amount, self.denom)
    }
}
