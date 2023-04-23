use thiserror::Error;

pub type WeLoveResult<T> = Result<T, WeLoveError>;

#[derive(Error, Debug)]
pub enum WeLoveError {
    #[error("reqwest_error: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("serde_error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("none_error: {0}")]
    None(&'static str),
    #[error("other_error: {0}")]
    Other(String),
}
