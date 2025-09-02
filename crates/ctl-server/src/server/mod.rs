mod auth;
mod level;
mod level_set;
mod music;
mod musicians;
mod users;

#[cfg(test)]
mod tests;

use crate::{
    database::{
        auth::{AuthSession, User},
        error::{RequestError, RequestResult as Result},
        types::*,
    },
    prelude::*,
    AppConfig, AppSecrets,
};

use std::collections::BTreeMap;

use ctl_core::prelude::{Id, LevelInfo, LevelSetInfo, MusicInfo, UserInfo};

use axum::{
    body::Body,
    extract::{Path, Query, State},
    http::header,
    response::IntoResponse,
    routing::{get, post},
    Extension, Form, Json,
};
use axum_login::{
    tower_sessions::{ExpiredDeletion, Expiry, SessionManagerLayer},
    AuthManagerLayerBuilder,
};
use reqwest::Client;
use serde::Deserialize;
use sqlx::Row;
use time::{Duration, OffsetDateTime};
use tokio::sync::RwLock;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tower_sessions::cookie::Key;
use tower_sessions_sqlx_store::SqliteStore;

type Router = axum::Router<Arc<App>>;

#[derive(Deserialize)]
struct IdQuery {
    id: Id,
}

struct App {
    database: DatabasePool,
    config: AppConfig,
    secrets: AppSecrets,

    account_links: RwLock<BTreeMap<String, Id>>,
}

pub async fn run(
    port: u16,
    database: DatabasePool,
    config: AppConfig,
    secrets: AppSecrets,
) -> color_eyre::Result<()> {
    let addr = format!("0.0.0.0:{port}");
    info!("Starting the server on {}", addr);

    let app = Arc::new(App {
        database,
        config,
        secrets,

        account_links: RwLock::new(BTreeMap::new()),
    });

    // Session layer
    let session_store = SqliteStore::new(app.database.clone());
    session_store.migrate().await?;

    let deletion_task = tokio::task::spawn(
        session_store
            .clone()
            .continuously_delete_expired(tokio::time::Duration::from_secs(60)),
    );

    let key = Key::generate();
    let session_layer = SessionManagerLayer::new(session_store)
        .with_expiry(Expiry::OnInactivity(Duration::days(1)))
        .with_signed(key);

    // Auth service
    let backend = crate::database::auth::Backend::new(app.database.clone());
    let auth_layer = AuthManagerLayerBuilder::new(backend, session_layer).build();

    let router = Router::new()
        .route("/", get(get_root))
        .merge(auth::router())
        .merge(users::router())
        .merge(musicians::router());

    let router = music::route(router);
    let router = level_set::route(router);
    let router = level::route(router);

    let mut client = Client::builder();
    if let Some(proxy) = &app.config.proxy {
        info!("Configuring proxy server: {proxy}");
        client = client.proxy(reqwest::Proxy::all(proxy).wrap_err("when configuring proxy")?);
    }
    let client = client.build().wrap_err("failed to build the http client")?;

    let router = router
        .layer(axum::middleware::from_fn(
            auth::token::auth_header_required_middleware,
        ))
        .layer(auth_layer)
        .layer(TraceLayer::new_for_http())
        .layer(
            CorsLayer::new()
                .allow_origin(tower_http::cors::Any)
                .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE]),
        )
        .layer(Extension(client))
        .with_state(app);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .context("when binding a tcp listener")?;
    axum::serve(listener, router).await?;

    deletion_task.await??;

    Ok(())
}

async fn get_root() -> &'static str {
    "Hello, world"
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum AuthorityLevel {
    Unauthorized,
    User,
    Admin,
}

async fn get_user_auth(user: &User, trans: &mut Transaction) -> Result<AuthorityLevel> {
    let auth = sqlx::query("SELECT null FROM admins WHERE user_id = ?")
        .bind(user.user_id)
        .fetch_optional(&mut **trans)
        .await?;
    match auth {
        None => Ok(AuthorityLevel::User),
        Some(_) => Ok(AuthorityLevel::Admin),
    }
}

async fn get_auth(session: &AuthSession, app: &App) -> Result<AuthorityLevel> {
    let Some(user) = &session.user else {
        return Ok(AuthorityLevel::Unauthorized);
    };
    let mut trans = app.database.begin().await?;
    let auth = get_user_auth(user, &mut trans).await?;
    trans.commit().await?;
    Ok(auth)
}

fn cmp_auth(auth: AuthorityLevel, required: AuthorityLevel) -> Result<()> {
    if let AuthorityLevel::Unauthorized = auth {
        Err(RequestError::Unathorized)
    } else if auth < required {
        Err(RequestError::Forbidden)
    } else {
        Ok(())
    }
}

async fn check_user(session: &AuthSession) -> Result<&User> {
    session.user.as_ref().ok_or(RequestError::Unathorized)
}

async fn check_auth(session: &AuthSession, app: &App, required: AuthorityLevel) -> Result<()> {
    let auth = get_auth(session, app).await?;
    cmp_auth(auth, required)
}

fn validate_name(name: &str) -> Result<String> {
    let name = name.trim().to_owned();
    if name.is_empty() {
        return Err(RequestError::InvalidName(name));
    }
    Ok(name)
}

fn validate_romanized_name(name: &str) -> Result<String> {
    let name = validate_name(name)?;
    if !name.is_ascii() {
        return Err(RequestError::NonAscii);
    }
    Ok(name)
}

// /// Load the file as bytes from a multipart message.
// async fn receive_file(mut multipart: Multipart) -> std::io::Result<Vec<u8>> {
//     use std::io::Write;

//     debug!("Receiving a file...");
//     let mut file = Vec::new();
//     while let Some(field) = multipart.next_field().await.unwrap() {
//         // let name = field.name().unwrap().to_string();
//         let data = field.bytes().await.unwrap();
//         file.write_all(&data)?;
//     }
//     debug!("File downloaded successfully");

//     Ok(file)
// }

async fn send_file(
    path: impl AsRef<std::path::Path>,
    content_type: String,
) -> Result<impl IntoResponse> {
    let path = path.as_ref();
    let filename = path.file_name().expect("not a file"); // TODO: not crash

    // `File` implements `AsyncRead`
    let file = match tokio::fs::File::open(path).await {
        Ok(file) => file,
        Err(err) => return Err(RequestError::FileNotFound(format!("{err}"))),
    };
    // convert the `AsyncRead` into a `Stream`
    let stream = tokio_util::io::ReaderStream::new(file);
    // convert the `Stream` into an `Body`
    let body = Body::from_stream(stream);

    let headers = [
        (header::CONTENT_TYPE, content_type),
        (
            header::CONTENT_DISPOSITION,
            format!("attachment; filename={filename:?}"),
        ),
    ];

    Ok((headers, body))
}

fn content_mp3() -> String {
    "audio/mpeg".to_owned()
}

// fn content_wav() -> String {
//     "audio/wav".to_owned()
// }

fn content_level() -> String {
    "application/octet-stream".to_owned()
}
