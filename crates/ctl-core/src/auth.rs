use super::*;

pub const PASSWORD_MIN_LEN: usize = 6;
pub const USERNAME_MIN_LEN: usize = 3;

/// Authentications credentials.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credentials {
    pub username: String,
    pub password: String,
}
