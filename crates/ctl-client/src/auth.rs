use super::*;

use ctl_core::{auth::Credentials, types::UserInfo};

impl Nertboard {
    pub async fn register(&self, creds: &Credentials) -> Result<Result<(), String>> {
        let url = self.url.join("register")?;
        let req = self.client.post(url).form(creds);
        let response = req.send().await.context("when sending request")?;
        let status = response.status();
        let response = get_body(response).await?;
        if status.is_server_error() || status.is_client_error() {
            Ok(Err(response))
        } else {
            Ok(Ok(()))
        }
    }

    pub async fn login(&self, creds: &Credentials) -> Result<Result<UserInfo, String>> {
        let url = self.url.join("login")?;
        let req = self.client.post(url).form(creds);
        let response = req.send().await.context("when sending request")?;
        let status = response.status();
        if status.is_server_error() || status.is_client_error() {
            Ok(Err(get_body(response).await?))
        } else {
            Ok(Ok(read_json(response).await?))
        }
    }

    pub async fn logout(&self) -> Result<()> {
        let url = self.url.join("logout")?;
        let req = self.client.get(url);
        let response = req.send().await.context("when sending request")?;
        get_body(response).await?;
        Ok(())
    }
}
