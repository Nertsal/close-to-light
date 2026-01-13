use super::*;

pub const PASSWORD_MIN_LEN: usize = 6;
pub const USERNAME_MIN_LEN: usize = 3;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UserLogin {
    pub id: Id,
    pub name: Name,
    /// The token that can be used to login later.
    pub token: Name,
}

/// Authentications credentials.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credentials {
    pub user_id: Id,
    pub token: String,
}

/// Credentials used to authenticate via Steam.
#[derive(Serialize, Deserialize)]
pub struct LoginSteam {
    pub demo: bool,
    pub ticket: String,
    pub username: String,
}
