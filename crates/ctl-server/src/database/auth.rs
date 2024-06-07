use self::error::RequestError;

use super::{types::DatabasePool, *};

use axum_login::{AuthUser, AuthnBackend, UserId};
use ctl_core::{auth::Credentials, types::UserInfo};
use sqlx::FromRow;

#[derive(Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub user_id: Id,
    pub username: String,
    /// Password hash.
    password: Option<String>,
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

impl From<User> for UserInfo {
    fn from(val: User) -> Self {
        Self {
            id: val.user_id,
            name: val.username.into(),
        }
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
        self.password.as_ref().map_or(&[], |pass| pass.as_bytes())
    }
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
    type Error = RequestError;

    async fn authenticate(
        &self,
        creds: Self::Credentials,
    ) -> Result<Option<Self::User>, Self::Error> {
        let login = sqlx::query("SELECT null FROM user_tokens WHERE user_id = ? AND token = ?")
            .bind(creds.user_id)
            .bind(&creds.token)
            .fetch_optional(&self.db)
            .await?;
        if login.is_none() {
            return Err(RequestError::InvalidCredentials);
        }

        let user: User = sqlx::query_as("SELECT * FROM users WHERE user_id = ?")
            .bind(creds.user_id)
            .fetch_one(&self.db)
            .await?;

        Ok(Some(user))
    }

    async fn get_user(&self, user_id: &UserId<Self>) -> Result<Option<Self::User>, Self::Error> {
        let user = sqlx::query_as("SELECT * FROM users WHERE user_id = ?")
            .bind(user_id)
            .fetch_optional(&self.db)
            .await?;
        Ok(user)
    }
}
