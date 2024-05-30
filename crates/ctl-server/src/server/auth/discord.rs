use super::*;

const REDIRECT_URI: &str = "https://nertboard.kuviman.com/auth/discord";
// const REDIRECT_URI: &str = "http://localhost:3000/auth/discord";

pub fn router() -> Router {
    Router::new().route("/auth/discord", get(auth_discord))
}

#[derive(Deserialize)]
struct CodeQuery {
    code: String,
    state: String,
}

#[derive(Deserialize)]
struct AccessTokenResponse {
    access_token: String,
    token_type: String,
    // expires_in: u64,
    // refresh_token: String,
    // scope: String,
}

#[derive(Deserialize)]
struct User {
    id: String,
    username: String,
    display_name: Option<String>,
}

async fn auth_discord(
    State(app): State<Arc<App>>,
    Query(query): Query<CodeQuery>,
    Extension(client): Extension<Client>,
) -> Result<String> {
    let user = discord_oauth(&app, &client, query.code).await?;
    let user_id = discord_login(&app, user).await?;

    register_login_state(&app, user_id, query.state).await?;

    let user: UserRow = sqlx::query_as("SELECT * FROM users WHERE user_id = ?")
        .bind(user_id)
        .fetch_one(&app.database)
        .await?;
    Ok(format!(
        "Logged in as {}, you can close this page and go back to the game",
        user.username
    ))
}

async fn discord_oauth(app: &App, client: &Client, code: String) -> Result<User> {
    let token: color_eyre::Result<AccessTokenResponse> = async {
        let body = format!(
            "grant_type=authorization_code&code={}&redirect_uri={}",
            code, REDIRECT_URI
        );
        let response = client
            .post("https://discord.com/api/oauth2/token")
            .basic_auth(
                &app.secrets.discord.client_id,
                Some(&app.secrets.discord.client_secret),
            )
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await?;
        let token = response.json().await?;
        Ok(token)
    }
    .await;
    let token = match token {
        Ok(token) => token,
        Err(err) => {
            tracing::error!("failed to retrieve user token: {:?}", err);
            return Err(RequestError::Internal); // TODO: better error
        }
    };

    let user: color_eyre::Result<User> = async {
        let response = client
            .get("https://discord.com/api/users/@me")
            .header(
                "Authorization",
                format!("{} {}", token.token_type, token.access_token),
            )
            .send()
            .await?;
        let user = response.json().await?;
        Ok(user)
    }
    .await;
    let user = match user {
        Ok(user) => user,
        Err(err) => {
            tracing::error!("failed to retrive user information: {:?}", err);
            return Err(RequestError::Internal); // TODO: Better error
        }
    };

    Ok(user)
}

async fn discord_login(app: &App, user: User) -> Result<Id> {
    // Check for a user with that discord account linked
    let user_id: Option<Id> = sqlx::query("SELECT user_id FROM user_accounts WHERE discord = ?")
        .bind(&user.id)
        .try_map(|row: DBRow| row.try_get("user_id"))
        .fetch_optional(&app.database)
        .await?;

    if let Some(user_id) = user_id {
        // Log in as the user
        return Ok(user_id);
    }

    // Register a new user
    let username = user.display_name.unwrap_or(user.username);
    let user_id = super::register_user(app, username.clone(), None, true)
        .await
        .map_err(|_| RequestError::InvalidCredentials)?; // TODO: better error
    link_discord(app, user_id, user.id).await?;

    Ok(user_id)
}

async fn link_discord(app: &App, user_id: Id, discord_id: String) -> Result<()> {
    sqlx::query("INSERT INTO user_accounts (user_id, discord) VALUES (?, ?)")
        .bind(user_id)
        .bind(&discord_id)
        .execute(&app.database)
        .await?;
    Ok(())
}
