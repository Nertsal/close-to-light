mod discord;
mod native;
pub mod token;

use super::*;

use axum::http::StatusCode;
use ctl_core::{
    auth::{PASSWORD_MIN_LEN, USERNAME_MIN_LEN},
    types::UserLogin,
};

pub fn router() -> Router {
    native::router()
        .merge(discord::router())
        .merge(token::router())
        .route("/auth/wait", get(auth_wait))
}

#[derive(Deserialize)]
struct StateQuery {
    state: String,
}

/// Waits until the client with the given `state` authenticates.
async fn auth_wait(
    mut session: AuthSession,
    State(app): State<Arc<App>>,
    Query(query): Query<StateQuery>,
) -> Result<Json<UserLogin>> {
    let user_id = wait_login_state(&app, &query.state).await?;
    let user: User = sqlx::query_as("SELECT * FROM users WHERE user_id = ?")
        .bind(user_id)
        .fetch_one(&app.database)
        .await?;

    let token = token::generate_login_token(&app, user_id).await?;

    session
        .login(&user)
        .await
        .map_err(|_| RequestError::Internal)?;

    Ok(Json(UserLogin {
        id: user.user_id,
        name: user.username.into(),
        token: token.into(),
    }))
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

async fn register_user(
    app: &App,
    mut username: String,
    mut password: Option<String>,
    bypass_name_validation: bool,
) -> Result<Id, RegisterError> {
    if let Some(pass) = &password {
        // Validate password
        if pass.len() < PASSWORD_MIN_LEN {
            return Err(RegisterError::PasswordTooShort {
                minimum: PASSWORD_MIN_LEN,
            });
        }
        // Hash password
        password = Some(password_auth::generate_hash(pass));
    }

    if !bypass_name_validation {
        // Validate username
        username = username.trim().to_string();
        if username.len() < USERNAME_MIN_LEN {
            return Err(RegisterError::UsernameTooShort {
                minimum: USERNAME_MIN_LEN,
            });
        }

        // Check if username is taken
        let check = sqlx::query("SELECT null FROM users WHERE username = ?")
            .bind(&username)
            .fetch_optional(&app.database)
            .await?;
        if check.is_some() {
            return Err(RegisterError::UsernameTaken);
        }
    }

    // Create new user
    let user_id =
        sqlx::query("INSERT INTO users (username, password) VALUES (?, ?) RETURNING user_id")
            .bind(&username)
            .bind(&password)
            .try_map(|row: DBRow| row.try_get("user_id"))
            .fetch_one(&app.database)
            .await?;

    Ok(user_id)
}

async fn register_login_state(app: &App, user_id: Id, state: String) -> Result<()> {
    let mut states = app.account_links.write().await;
    match states.entry(state) {
        std::collections::btree_map::Entry::Vacant(entry) => {
            entry.insert(user_id);
            Ok(())
        }
        std::collections::btree_map::Entry::Occupied(_) => Err(RequestError::InvalidCredentials), // TODO: better error
    }
}

/// Wait until the login state is registered, or until timeout.
async fn wait_login_state(app: &App, state: &String) -> Result<Id> {
    const TIMEOUT: u64 = 300; // seconds until timeout

    let wait_state = async {
        loop {
            let mut states = app.account_links.write().await;
            if let Some(user) = states.remove(state) {
                return user;
            }
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        }
    };
    let timeout = tokio::time::sleep(tokio::time::Duration::from_secs(TIMEOUT));

    tokio::select! {
        _ = timeout => {
            Err(RequestError::Timeout)
        }
        user = wait_state => {
            Ok(user)
        }
    }
}
