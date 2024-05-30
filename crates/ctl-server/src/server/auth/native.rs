use super::*;

// use axum::http::StatusCode;
// use ctl_core::auth::Credentials;

pub fn router() -> Router {
    Router::new()
        // .route("/register", post(register))
        // .route("/login", post(login))
        .route("/logout", get(logout))
}

// async fn register(
//     State(app): State<Arc<App>>,
//     Form(mut creds): Form<Credentials>,
// ) -> Result<(), RegisterError> {
//     use ctl_core::auth::{PASSWORD_MIN_LEN, USERNAME_MIN_LEN};

//     TODO
//     super::register_user()

//     Ok(())
// }

// async fn login(mut session: AuthSession, Form(creds): Form<Credentials>) -> Result<Json<UserInfo>> {
//     let user = match session.authenticate(creds.clone()).await {
//         Ok(Some(user)) => user,
//         Ok(None) => {
//             return Err(RequestError::InvalidCredentials);
//         }
//         Err(err) => {
//             error!("Authentication failed: {:?}", err);
//             return Err(RequestError::Internal);
//         }
//     };

//     if let Err(err) = session.login(&user).await {
//         error!("Login failed: {:?}", err);
//         return Err(RequestError::Internal);
//     }

//     Ok(Json(UserInfo {
//         id: user.user_id,
//         name: user.username.into(),
//     }))
// }

#[derive(Deserialize)]
struct TokenQuery {
    token: Option<String>,
}

async fn logout(
    mut session: AuthSession,
    State(app): State<Arc<App>>,
    Query(query): Query<TokenQuery>,
) -> Result<()> {
    let user = session.logout().await.map_err(|err| {
        error!("Logout failed: {:?}", err);
        RequestError::Internal
    })?;

    if let Some(user) = user {
        if let Some(token) = query.token {
            // Remove the login token from the database
            sqlx::query("DELETE FROM user_tokens WHERE user_id = ? AND token = ?")
                .bind(user.user_id)
                .bind(&token)
                .execute(&app.database)
                .await?;
        }
    }

    Ok(())
}
