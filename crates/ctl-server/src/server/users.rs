use super::*;

pub fn router() -> Router {
    Router::new().route("/user/me", get(user_me))
}

pub async fn user_me(session: AuthSession) -> Result<String> {
    let user = session.user.as_ref().ok_or(RequestError::Unathorized)?;
    Ok(user.username.clone())
}
