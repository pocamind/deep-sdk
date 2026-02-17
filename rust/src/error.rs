
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DeepError {
    #[error("Parse error: {0}")]
    Req(String),
    #[error("Parse on line {line}: {message}")]
    Reqfile {
        line: usize,
        message: String
    },
    #[error("IO error: {0}")]
    IO(String),
    #[error("Serde error: {0}")]
    SerdeError(#[from] serde_json::Error),
    
    #[cfg(feature = "fetch")]
    #[error("Reqwest error: {0}")]
    ReqwestError(#[from] reqwest::Error),

    #[cfg(feature = "fetch")]
    #[error("Fetch data error: {0}")]
    FetchError(String),
}

pub type Result<T> = core::result::Result<T, DeepError>;

impl From<std::io::Error> for DeepError {
    fn from(value: std::io::Error) -> Self {
        Self::IO(value.to_string())
    }
}