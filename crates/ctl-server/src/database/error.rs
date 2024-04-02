use super::*;

pub type RequestResult<T, E = RequestError> = std::result::Result<T, E>;

#[derive(thiserror::Error, Debug)]
pub enum RequestError {
    #[error("Server error")]
    Internal,
    #[error("Unathorized request")]
    Unathorized,
    #[error("Forbidden")]
    Forbidden,
    #[error("Invalid credentials")]
    InvalidCredentials,
    #[error("Player key is invalid")]
    InvalidPlayer,
    #[error("Invalid name {0}")]
    InvalidName(String),
    #[error("Player {0} not found")]
    NoSuchPlayer(Id),
    #[error("Artist {0} not found")]
    NoSuchArtist(Id),
    #[error("Group {0} not found")]
    NoSuchGroup(Id),
    #[error("Music {0} not found")]
    NoSuchMusic(Id),
    #[error("Level {0} not found")]
    NoSuchLevel(Id),
    #[error("Such a level already exists")]
    LevelAlreadyExists,
    #[error("Server error")]
    FileNotFound(String),
    #[error("Database error")]
    Sql(#[from] sqlx::Error),
    #[error("Server error")]
    Io(#[from] std::io::Error),
}

impl RequestError {
    fn status(&self) -> StatusCode {
        match self {
            RequestError::Internal => StatusCode::INTERNAL_SERVER_ERROR,
            RequestError::Unathorized => StatusCode::UNAUTHORIZED,
            RequestError::Forbidden => StatusCode::FORBIDDEN,
            RequestError::InvalidCredentials => StatusCode::UNAUTHORIZED,
            RequestError::InvalidPlayer => StatusCode::FORBIDDEN,
            RequestError::InvalidName(_) => StatusCode::BAD_REQUEST,
            RequestError::FileNotFound(_) => StatusCode::NOT_FOUND,
            RequestError::NoSuchPlayer(_) => StatusCode::NOT_FOUND,
            RequestError::NoSuchArtist(_) => StatusCode::NOT_FOUND,
            RequestError::NoSuchMusic(_) => StatusCode::NOT_FOUND,
            RequestError::NoSuchGroup(_) => StatusCode::NOT_FOUND,
            RequestError::NoSuchLevel(_) => StatusCode::NOT_FOUND,
            RequestError::LevelAlreadyExists => StatusCode::CONFLICT,
            RequestError::Sql(_) => StatusCode::INTERNAL_SERVER_ERROR,
            RequestError::Io(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl axum::response::IntoResponse for RequestError {
    fn into_response(self) -> axum::response::Response {
        tracing::debug!("Responding with an error: {:?}", self);
        let body = format!("{}", self);
        (self.status(), body).into_response()
    }
}
