use super::*;

use crate::database::auth::Credentials;

pub fn router() -> Router {
    Router::new()
        .route("/login", post(login))
        .route("/logout", get(logout))
}

async fn login(mut session: AuthSession, Form(creds): Form<Credentials>) -> Result<()> {
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

    Ok(())
}

async fn logout(mut session: AuthSession) -> Result<()> {
    session.logout().await.map_err(|err| {
        error!("Logout failed: {:?}", err);
        RequestError::Internal
    })?;
    Ok(())
}
