use ctl_core::prelude::serde_json;
use reqwest::StatusCode;

#[derive(thiserror::Error, Debug)]
pub enum ClientError {
    #[error("Connection failed")]
    Connection,
    #[error("Unexpected error occurred")]
    UnexpectedFormat(String),
    #[error("Unexpected error occurred")]
    Reqwest(reqwest::Error),
    #[error("Unexpected error occurred")]
    Bincode(#[from] bincode::Error),
    #[error("Unexpected error occurred")]
    Json(#[from] serde_json::Error),
    #[error("Unexpected error occurred")]
    Url(#[from] url::ParseError),
    #[error("Unexpected error occurred")]
    Io(#[from] std::io::Error),
    #[error("Server error occurred")]
    Server(String),
    #[error("{0}")]
    Client(String),
    #[error("Not found")]
    NotFound,
    #[cfg(feature = "steam")]
    #[error("Could not connect to Steam")]
    Steam,
}

impl From<reqwest::Error> for ClientError {
    fn from(value: reqwest::Error) -> Self {
        #[cfg(not(target_arch = "wasm32"))] // TODO: figure out what's up
        if value.is_connect() {
            return Self::Connection;
        }
        if let Some(StatusCode::NOT_FOUND) = value.status() {
            Self::NotFound
        } else {
            Self::Reqwest(value)
        }
    }
}
