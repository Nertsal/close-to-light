mod init;

pub use self::init::init_database;

use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use sqlx::types::Uuid;

pub type DatabasePool = sqlx::SqlitePool; // TODO: behind a trait?
pub type DBRow = sqlx::sqlite::SqliteRow;

pub type RequestResult<T, E = RequestError> = std::result::Result<T, E>;

pub type Score = i32;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreRecord {
    pub player_id: Uuid,
    pub score: Score,
    pub extra_info: Option<String>,
}

// TODO: fix so that the message appears in logs, not on the client
#[derive(thiserror::Error, Debug)]
pub enum RequestError {
    #[error("internal error")]
    Internal,
    #[error("unathorized request")]
    Unathorized,
    #[error("unathorized request, not enough rights")]
    Forbidden,
    #[error("player key is invalid")]
    InvalidPlayer,
    #[error("invalid name {0}")]
    InvalidName(String),
    #[error("group {0} not found")]
    NoSuchGroup(Uuid),
    #[error("music {0} not found")]
    NoSuchMusic(Uuid),
    #[error("level {0} not found")]
    NoSuchLevel(Uuid),
    #[error("file not found: {0}")]
    FileNotFound(String),
    #[error("database error: {0}")]
    Sql(#[from] sqlx::Error),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

impl RequestError {
    fn status(&self) -> StatusCode {
        match self {
            RequestError::Internal => StatusCode::INTERNAL_SERVER_ERROR,
            RequestError::Unathorized => StatusCode::UNAUTHORIZED,
            RequestError::Forbidden => StatusCode::FORBIDDEN,
            RequestError::InvalidPlayer => StatusCode::FORBIDDEN,
            RequestError::InvalidName(_) => StatusCode::BAD_REQUEST,
            RequestError::FileNotFound(_) => StatusCode::NOT_FOUND,
            RequestError::NoSuchMusic(_) => StatusCode::NOT_FOUND,
            RequestError::NoSuchGroup(_) => StatusCode::NOT_FOUND,
            RequestError::NoSuchLevel(_) => StatusCode::NOT_FOUND,
            RequestError::Sql(_) => StatusCode::INTERNAL_SERVER_ERROR,
            RequestError::Io(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl axum::response::IntoResponse for RequestError {
    fn into_response(self) -> axum::response::Response {
        let body = format!("{}", self);
        (self.status(), body).into_response()
    }
}
