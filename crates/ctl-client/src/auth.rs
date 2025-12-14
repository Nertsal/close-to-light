use super::*;

use ctl_core::types::UserLogin;

impl Nertboard {
    async fn login(&self, response: Response) -> Result<Result<UserLogin, String>> {
        match get_json_or::<UserLogin>(response).await? {
            Err(err) => Ok(Err(err)),
            Ok(user) => {
                log::debug!("Logged in as {} ({})", user.name, user.id);
                *self.auth.write().await = Some((user.id.to_string(), user.token.to_string()));
                Ok(Ok(user))
            }
        }
    }

    #[cfg(feature = "steam")]
    pub async fn login_steam(&self) -> Result<Result<UserLogin, String>> {
        let Some(steam) = &self.steam else {
            return Err(ClientError::Steam);
        };

        let ticket = get_steam_ticket(steam).await?;
        let username = steam.friends().name();

        let url = self.url.join("auth/steam").unwrap();
        let req = self
            .client
            .post(url)
            .query(&[("ticket", ticket), ("username", username)]);
        let response = self.send(req).await?;
        self.login(response).await
    }

    /// Waits for the authentication from the external service and logs in after.
    pub async fn login_external(&self, state: String) -> Result<Result<UserLogin, String>> {
        let url = self.url.join("auth/wait").unwrap();
        let req = self.client.get(url).query(&[("state", state)]);
        let response = self.send(req).await?;
        self.login(response).await
    }

    pub async fn login_token(&self, user_id: Id, token: &str) -> Result<Result<UserLogin, String>> {
        let url = self.url.join("auth/token")?;
        let req = self.client.post(url).basic_auth(user_id, Some(token));
        let response = self.send(req).await?;
        self.login(response).await
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
        let response = self.send(req).await?;
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

#[cfg(feature = "steam")]
async fn get_steam_ticket(steam: &steamworks::Client) -> Result<String> {
    log::debug!("Retrieving Steam session ticket...");

    // Ask Steam for session ticket
    let (sender, receiver) = std::sync::mpsc::channel();
    let _ticket_callback =
        steam.register_callback(move |response: steamworks::TicketForWebApiResponse| {
            log::debug!("Retrieved Steam ticket: {:?}", response.result);
            if let Err(err) = sender.send(response.ticket) {
                log::error!("failed to send steam ticket over mpsc: {:?}", err);
            }
        });
    let _ticket = steam
        .user()
        .authentication_session_ticket_for_webapi(ctl_constants::STEAM_SERVER_IDENTITY);

    // Wait for Steam response
    for _ in 0..20 {
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        if let Ok(ticket) = receiver.try_recv() {
            return Ok(hex::encode(&ticket));
        }
    }

    // Timeout
    log::error!("Waiting for Steam session ticket timed out");
    Err(ClientError::Steam)
}
