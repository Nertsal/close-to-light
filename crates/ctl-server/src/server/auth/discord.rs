use super::*;

// const REDIRECT_URI: &str = "https://nertboard.kuviman.com/auth/discord";
const REDIRECT_URI: &str = "http://localhost:3000/auth/discord";

pub fn router() -> Router {
    Router::new().route("/auth/discord", get(auth_discord))
}

#[derive(Deserialize)]
struct CodeQuery {
    code: String,
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
) -> Result<String> {
    let code = query.code;

    let client = reqwest::Client::builder()
        .build()
        .expect("failed to build the http client");

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

    let username = user.display_name.unwrap_or(user.username);

    Ok(format!(
        "Logged in as {}, you can close this page and go back to the game",
        username
    ))
}
