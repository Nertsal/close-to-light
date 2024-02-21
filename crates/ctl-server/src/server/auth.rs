use axum::http::StatusCode;

use super::*;

use ctl_core::auth::Credentials;

pub fn router() -> Router {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/logout", get(logout))
}

#[derive(thiserror::Error, Debug)]
pub enum RegisterError {
    #[error("Password has to be at least {minimum} characters")]
    PasswordTooShort { minimum: usize },
    #[error("Username has to be at least {minimum} characters")]
    UsernameTooShort { minimum: usize },
    #[error("User with that name already exist")]
    UsernameTaken,
    #[error("Database error")]
    Sql(#[from] sqlx::Error),
}

impl RegisterError {
    fn status(&self) -> StatusCode {
        match self {
            RegisterError::PasswordTooShort { .. } => StatusCode::BAD_REQUEST,
            RegisterError::UsernameTooShort { .. } => StatusCode::BAD_REQUEST,
            RegisterError::UsernameTaken => StatusCode::BAD_REQUEST,
            RegisterError::Sql(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl axum::response::IntoResponse for RegisterError {
    fn into_response(self) -> axum::response::Response {
        tracing::debug!("Responding with an error: {:?}", self);
        let body = format!("{}", self);
        (self.status(), body).into_response()
    }
}

async fn register(
    State(app): State<Arc<App>>,
    Form(mut creds): Form<Credentials>,
) -> Result<(), RegisterError> {
    use ctl_core::auth::{PASSWORD_MIN_LEN, USERNAME_MIN_LEN};

    // Validate password
    if creds.password.len() < PASSWORD_MIN_LEN {
        return Err(RegisterError::PasswordTooShort {
            minimum: PASSWORD_MIN_LEN,
        });
    }
    // Hash password
    creds.password = password_auth::generate_hash(&creds.password);

    // Validate username
    creds.username = creds.username.trim().to_string();
    if creds.username.len() < USERNAME_MIN_LEN {
        return Err(RegisterError::UsernameTooShort {
            minimum: USERNAME_MIN_LEN,
        });
    }

    // Check if username is taken
    let check = sqlx::query("SELECT null FROM users WHERE username = ?")
        .bind(&creds.username)
        .fetch_optional(&app.database)
        .await?;
    if check.is_some() {
        return Err(RegisterError::UsernameTaken);
    }

    // Create new user
    sqlx::query("INSERT INTO users (username, password) VALUES (?, ?)")
        .bind(&creds.username)
        .bind(&creds.password)
        .execute(&app.database)
        .await?;

    Ok(())
}

async fn login(mut session: AuthSession, Form(creds): Form<Credentials>) -> Result<Json<UserInfo>> {
    let user = match session.authenticate(creds.clone()).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            return Err(RequestError::InvalidCredentials);
        }
        Err(err) => {
            error!("Authentication failed: {:?}", err);
            return Err(RequestError::Internal);
        }
    };

    if let Err(err) = session.login(&user).await {
        error!("Login failed: {:?}", err);
        return Err(RequestError::Internal);
    }

    Ok(Json(UserInfo {
        id: user.user_id,
        name: user.username,
    }))
}

async fn logout(mut session: AuthSession) -> Result<()> {
    session.logout().await.map_err(|err| {
        error!("Logout failed: {:?}", err);
        RequestError::Internal
    })?;
    Ok(())
}
