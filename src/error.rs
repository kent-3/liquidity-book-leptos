// The Serialize and Deserialize traits are derived to ensure that Errors can be
// transmitted to or from a server, which is necessary for them to function as Resources.
#[derive(thiserror::Error, serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub enum Error {
    #[error("{0}")]
    Generic(String),

    #[error("Token not found in the map!")]
    UnknownToken,

    #[error("Serde Error: {0}")]
    Serde(String),

    #[error("Secret error: {0}")]
    Secret(String),

    #[error("Keplr error: {0}")]
    Keplr(String),

    #[error("Keplr is not enabled!")]
    KeplrDisabled,
}

impl Error {
    pub fn generic(message: impl ToString) -> Self {
        let message = message.to_string();
        Error::Generic(message)
    }
    pub fn serde(message: impl ToString) -> Self {
        let message = message.to_string();
        Error::Serde(message)
    }
}

impl From<&str> for Error {
    fn from(value: &str) -> Self {
        Self::Generic(value.to_string())
    }
}

impl From<String> for Error {
    fn from(value: String) -> Self {
        Self::Generic(value)
    }
}

impl From<std::num::ParseIntError> for Error {
    fn from(error: std::num::ParseIntError) -> Self {
        Self::Generic(error.to_string())
    }
}

impl From<std::num::ParseFloatError> for Error {
    fn from(error: std::num::ParseFloatError) -> Self {
        Self::Generic(error.to_string())
    }
}

impl From<cosmwasm_std::StdError> for Error {
    fn from(error: cosmwasm_std::StdError) -> Self {
        Error::Secret(error.to_string())
    }
}

impl From<rsecret::Error> for Error {
    fn from(error: rsecret::Error) -> Self {
        Error::Secret(error.to_string())
    }
}

impl From<secretrs::ErrorReport> for Error {
    fn from(error: secretrs::ErrorReport) -> Self {
        Error::Secret(error.to_string())
    }
}

impl From<keplr::Error> for Error {
    fn from(error: keplr::Error) -> Self {
        Error::Keplr(error.to_string())
    }
}

impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Self {
        Error::Serde(error.to_string())
    }
}
