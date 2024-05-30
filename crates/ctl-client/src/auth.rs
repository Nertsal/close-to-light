use super::*;

use ctl_core::types::UserLogin;

impl Nertboard {
    /// Waits for the authentication from the external service and logs in after.
    pub async fn login_external(&self, state: String) -> Result<Result<UserLogin, String>> {
        let url = self.url.join("auth/wait").unwrap();
        let req = self.client.get(url).query(&[("state", state)]);
        let response = req.send().await?;
        get_json_or(response).await
    }

    pub async fn login_token(&self, user_id: Id, token: &str) -> Result<Result<UserLogin, String>> {
        let url = self.url.join("auth/token")?;
        let req = self.client.post(url).basic_auth(user_id, Some(token));
        let response = req.send().await?;
        get_json_or(response).await
    }

    // pub async fn register(&self, creds: &Credentials) -> Result<Result<(), String>> {
    //     let url = self.url.join("register")?;
    //     let req = self.client.post(url).form(creds);
    //     let response = req.send().await?;
    //     let status = response.status();
    //     let response = get_body(response).await?;
    //     if status.is_server_error() || status.is_client_error() {
    //         Ok(Err(response))
    //     } else {
    //         Ok(Ok(()))
    //     }
    // }

    // pub async fn login(&self, creds: &Credentials) -> Result<Result<UserInfo, String>> {
    //     let url = self.url.join("login")?;
    //     let req = self.client.post(url).form(creds);
    //     let response = req.send().await?;
    //     let status = response.status();
    //     if status.is_server_error() || status.is_client_error() {
    //         Ok(Err(get_body(response).await?))
    //     } else {
    //         Ok(Ok(read_json(response).await?))
    //     }
    // }

    pub async fn logout(&self, token: Option<&str>) -> Result<()> {
        let url = self.url.join("logout")?;
        let mut req = self.client.get(url);
        if let Some(token) = token {
            req = req.query(&[("token", token)]);
        }
        let response = req.send().await?;
        get_body(response).await?;
        Ok(())
    }
}

async fn get_json_or<T: DeserializeOwned>(response: Response) -> Result<Result<T, String>> {
    let status = response.status();
    if status.is_server_error() || status.is_client_error() {
        Ok(Err(get_body(response).await?))
    } else {
        Ok(Ok(read_json(response).await?))
    }
}
