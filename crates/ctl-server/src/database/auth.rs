use super::{types::DatabasePool, *};

use axum_login::{AuthUser, AuthnBackend, UserId};
use sqlx::FromRow;

#[derive(Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub user_id: Id,
    pub username: String,
    /// Password hash.
    password: String,
}

impl std::fmt::Debug for User {
    fn fmt(&self, f: &mut ctl_core::prelude::fmt::Formatter<'_>) -> ctl_core::prelude::fmt::Result {
        f.debug_struct("Player")
            .field("id", &self.user_id)
            .field("name", &self.username)
            .field("password", &"[redacted]")
            .finish()
    }
}

impl AuthUser for User {
    type Id = Id;

    fn id(&self) -> Self::Id {
        self.user_id
    }

    fn session_auth_hash(&self) -> &[u8] {
        // Password hash is session auth hash
        // so changing password invalidates session
        self.password.as_bytes()
    }
}

/// Authentications credentials.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credentials {
    pub username: String,
    pub password: String,
}

pub type AuthSession = axum_login::AuthSession<Backend>;

#[derive(Debug, Clone)]
pub struct Backend {
    db: DatabasePool,
}

impl Backend {
    pub fn new(db: DatabasePool) -> Self {
        Self { db }
    }
}

#[axum::async_trait]
impl AuthnBackend for Backend {
    type User = User;
    type Credentials = Credentials;
    type Error = sqlx::Error;

    async fn authenticate(
        &self,
        creds: Self::Credentials,
    ) -> Result<Option<Self::User>, Self::Error> {
        let user: Option<Self::User> = sqlx::query_as("SELECT * FROM users WHERE username = ?")
            .bind(&creds.username)
            .fetch_optional(&self.db)
            .await?;

        Ok(user.filter(|user| {
            password_auth::verify_password(creds.password, &user.password)
                .ok()
                .is_some()
        }))
    }

    async fn get_user(&self, user_id: &UserId<Self>) -> Result<Option<Self::User>, Self::Error> {
        let user = sqlx::query_as("SELECT * FROM users WHERE user_id = ?")
            .bind(user_id)
            .fetch_optional(&self.db)
            .await?;
        Ok(user)
    }
}
