use super::*;

pub fn router() -> Router {
    Router::new().route("/auth/token", post(auth_token))
}

async fn auth_token(
    mut session: AuthSession,
    State(app): State<Arc<App>>,
    headers: HeaderMap,
) -> Result<Json<UserLogin>> {
    use base64::Engine;

    let Some(credentials) = headers.get("Authorization") else {
        return Err(RequestError::Unathorized);
    };

    let Some(credentials) = credentials.as_bytes().strip_prefix(b"Basic ") else {
        return Err(RequestError::InvalidCredentials);
    };

    let credentials = base64::prelude::BASE64_STANDARD
        .decode(credentials)
        .map_err(|_| RequestError::InvalidCredentials)?;
    let credentials =
        String::from_utf8(credentials).map_err(|_| RequestError::InvalidCredentials)?;

    let mut parts = credentials.split(':');
    let user_id = parts.next().ok_or(RequestError::InvalidCredentials)?;

    let user_id: Id = user_id
        .parse()
        .map_err(|_| RequestError::InvalidCredentials)?;

    let token = parts
        .next()
        .ok_or(RequestError::InvalidCredentials)?
        .to_owned();

    let user = login_via_token(&mut session, &app, user_id, token).await?;
    Ok(Json(user))
}

pub(super) async fn generate_login_token(app: &App, user_id: Id) -> Result<String> {
    let token = uuid::Uuid::new_v4().to_string();

    sqlx::query("INSERT INTO user_tokens (user_id, token) VALUES (?, ?)")
        .bind(user_id)
        .bind(&token)
        .execute(&app.database)
        .await?;

    Ok(token)
}

pub(super) async fn login_via_token(
    session: &mut AuthSession,
    app: &App,
    user_id: Id,
    token: String,
) -> Result<UserLogin> {
    let login = sqlx::query("SELECT null FROM user_tokens WHERE user_id = ? AND token = ?")
        .bind(user_id)
        .bind(&token)
        .fetch_optional(&app.database)
        .await?;
    if login.is_none() {
        return Err(RequestError::InvalidCredentials);
    }

    let user: User = sqlx::query_as("SELECT * FROM users WHERE user_id = ?")
        .bind(user_id)
        .fetch_one(&app.database)
        .await?;

    session
        .login(&user)
        .await
        .map_err(|_| RequestError::Internal)?;

    Ok(UserLogin {
        id: user.user_id,
        name: user.username.into(),
        token: token.into(),
    })
}
