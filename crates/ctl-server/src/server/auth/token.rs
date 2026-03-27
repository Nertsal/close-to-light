use super::*;

use axum_extra::TypedHeader;
use ctl_core::auth::Credentials;
use headers::{Authorization, authorization::Basic};

pub fn router() -> Router {
    Router::new().route("/auth/token", post(auth_token_route))
}

pub async fn auth_header_required_middleware(
    mut session: AuthSession,
    auth_header: Option<TypedHeader<Authorization<Basic>>>,
    mut request: axum::extract::Request,
    next: axum::middleware::Next,
) -> impl IntoResponse {
    if session.user.is_none()
        && let Some(auth_header) = auth_header
    {
        // Attempt extracting token from header
        if auth_token(&mut session, auth_header.0).await.is_ok() {
            request.extensions_mut().insert(session);
        }
    }
    next.run(request).await
}

async fn auth_token_route(
    mut session: AuthSession,
    TypedHeader(auth_header): TypedHeader<Authorization<Basic>>,
) -> Result<Json<UserLogin>> {
    auth_token(&mut session, auth_header).await
}

async fn auth_token(
    session: &mut AuthSession,
    auth_header: Authorization<Basic>,
) -> Result<Json<UserLogin>> {
    let user_id = auth_header
        .username()
        .parse()
        .map_err(|_| RequestError::InvalidCredentials)?;
    let token = auth_header.password().to_owned();

    let back_err = |err| match err {
        axum_login::Error::Session(_) => RequestError::InvalidCredentials,
        axum_login::Error::Backend(err) => err,
    };

    let user = session
        .authenticate(Credentials {
            user_id,
            token: token.clone(),
        })
        .await
        .map_err(back_err)?
        .ok_or(RequestError::InvalidCredentials)?;
    session.login(&user).await.map_err(back_err)?;

    let user = UserLogin {
        id: user.user_id,
        name: user.username.into(),
        token: token.into(),
    };

    Ok(Json(user))
}

pub(super) async fn generate_login_token(
    app: &App,
    user_id: Id,
    expiration_date: Option<time::OffsetDateTime>,
) -> Result<String> {
    let token = uuid::Uuid::new_v4().to_string();

    sqlx::query("INSERT INTO user_auth_tokens (user_id, token, expiration_date) VALUES (?, ?, ?)")
        .bind(user_id)
        .bind(&token)
        .bind(expiration_date)
        .execute(&app.database)
        .await?;

    Ok(token)
}

pub async fn deletion_task(app: Arc<App>) {
    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(60));
    loop {
        interval.tick().await;
        if let Err(err) = delete_expired_tokens(&app).await {
            tracing::error!("{:?}", err);
        }
    }
}

async fn delete_expired_tokens(app: &App) -> Result<()> {
    let now = time::OffsetDateTime::now_utc();
    sqlx::query("DELETE FROM user_auth_tokens WHERE expiration_date < ?")
        .bind(now)
        .execute(&app.database)
        .await?;

    Ok(())
}
