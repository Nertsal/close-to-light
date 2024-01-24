use super::*;

use ctl_core::auth::Credentials;

impl Nertboard {
    pub async fn register(&self, creds: Credentials) -> Result<()> {
        let url = self.url.join("register")?;
        let req = self.client.post(url).form(&creds);
        let response = req.send().await.context("when sending request")?;
        get_body(response).await?;
        Ok(())
    }

    pub async fn login(&self, creds: Credentials) -> Result<()> {
        let url = self.url.join("login")?;
        let req = self.client.post(url).form(&creds);
        let response = req.send().await.context("when sending request")?;
        get_body(response).await?;
        Ok(())
    }

    pub async fn logout(&self) -> Result<()> {
        let url = self.url.join("logout")?;
        let req = self.client.get(url);
        let response = req.send().await.context("when sending request")?;
        get_body(response).await?;
        Ok(())
    }
}
