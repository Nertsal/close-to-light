use axum::http::{request::Parts, StatusCode};
use rand::Rng;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct StringKey(Box<str>);

#[derive(Serialize, Deserialize)]
pub struct BoardKeys {
    pub read: StringKey,
    pub submit: StringKey,
    pub admin: StringKey,
}

impl StringKey {
    pub fn new(key: impl Into<Box<str>>) -> Self {
        Self(key.into())
    }

    pub fn inner(&self) -> &str {
        &self.0
    }

    pub fn generate(length: usize) -> Self {
        let rng = rand::thread_rng();
        let key: String = rng
            .sample_iter(rand::distributions::Alphanumeric)
            .take(length)
            .map(char::from)
            .collect();
        Self(key.into())
    }
}

pub struct PlayerKey(pub String);

#[axum::async_trait]
impl<S> axum::extract::FromRequestParts<S> for PlayerKey {
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        match parts.headers.get("player-key") {
            None => Err((StatusCode::BAD_REQUEST, "player key missing")),
            Some(key) => match key.to_str() {
                Ok(key) => Ok(Self(key.to_string())),
                Err(_) => Err((StatusCode::BAD_REQUEST, "player key is invalid")),
            },
        }
    }
}
