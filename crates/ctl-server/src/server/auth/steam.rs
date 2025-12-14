use super::*;

pub fn router() -> Router {
    Router::new().route("/auth/steam", post(auth_steam))
}

#[derive(Deserialize)]
struct CodeQuery {
    ticket: String,
    username: String,
}

#[derive(Deserialize)]
struct SteamUser {
    id: String,
    username: String,
}

#[derive(Deserialize)]
struct AuthResponse {
    response: AuthResponseR,
}
#[derive(Deserialize)]
struct AuthResponseR {
    params: AuthResponseP,
}
#[derive(Deserialize)]
struct AuthResponseP {
    // result: String,
    steamid: String,
    // ownersteamid: String,
    // vacbanned: bool,
    // publisherbanned: bool,
}

async fn auth_steam(
    session: AuthSession,
    State(app): State<Arc<App>>,
    Query(query): Query<CodeQuery>,
    Extension(client): Extension<Client>,
) -> Result<Json<UserLogin>> {
    let user = steam_auth(&app, &client, query.ticket, query.username).await?;
    let user_id = steam_login(&app, user).await?;
    let expiration_date = Some(time::OffsetDateTime::now_utc() + time::Duration::days(1));
    login_user(session, &app, user_id, expiration_date).await
}

async fn steam_auth(
    app: &App,
    client: &Client,
    ticket: String,
    username: String,
) -> Result<SteamUser> {
    let response: color_eyre::Result<AuthResponse> = async {
        #[derive(serde::Serialize)]
        struct Query<'a> {
            key: &'a str,
            appid: u32,
            ticket: &'a str,
            identity: &'a str,
        }
        let query = Query {
            key: &app.secrets.steam.web_api_key,
            appid: ctl_constants::STEAM_APP_ID,
            ticket: &ticket,
            identity: ctl_constants::STEAM_SERVER_IDENTITY,
        };

        let response = client
            .get("https://partner.steam-api.com/ISteamUserAuth/AuthenticateUserTicket/v1/")
            .query(&query)
            .send()
            .await?;
        let content = response.text().await?;
        let steam_id = serde_json::from_str(&content).map_err(|err| {
            color_eyre::eyre::eyre!("received message: {content:?}, error: {err:?}")
        })?;
        Ok(steam_id)
    }
    .await;
    let response = match response {
        Ok(response) => response.response.params,
        Err(err) => {
            tracing::error!("failed to authenticate steam user: {:?}", err);
            return Err(RequestError::Internal); // TODO: better error
        }
    };
    let steam_id = response.steamid;

    Ok(SteamUser {
        id: steam_id,
        username,
    })
}

async fn steam_login(app: &App, user: SteamUser) -> Result<Id> {
    // Check for a user with that steam account linked
    let user_id: Option<Id> =
        sqlx::query_scalar("SELECT user_id FROM user_linked_accounts WHERE steam = ?")
            .bind(&user.id)
            .fetch_optional(&app.database)
            .await?;

    if let Some(user_id) = user_id {
        // Log in as the user
        // TODO: potentially rename user
        return Ok(user_id);
    }

    // Register a new user
    let username = user.username;
    let user_id = super::register_user(app, username.clone(), None, true)
        .await
        .map_err(|_| RequestError::InvalidCredentials)?; // TODO: better error
    link_steam(app, user_id, user.id).await?;

    Ok(user_id)
}

async fn link_steam(app: &App, user_id: Id, steam_id: String) -> Result<()> {
    sqlx::query("INSERT INTO user_linked_accounts (user_id, steam) VALUES (?, ?)")
        .bind(user_id)
        .bind(&steam_id)
        .execute(&app.database)
        .await?;
    Ok(())
}
