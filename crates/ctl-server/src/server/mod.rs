mod auth;
mod group;
mod level;
mod music;
mod player;

#[cfg(test)]
mod tests;

use crate::{
    api_key::*,
    database::{
        auth::AuthSession,
        error::{RequestError, RequestResult as Result},
        types::{DBRow, DatabasePool},
    },
    prelude::*,
    AppConfig,
};

use std::path::PathBuf;

use axum_login::AuthManagerLayerBuilder;
use ctl_core::prelude::{GroupInfo, Id, LevelInfo, MusicInfo, PlayerInfo};

use axum::{
    body::Body,
    extract::{Path, Query, State},
    http::header,
    response::IntoResponse,
    routing::{get, post},
    Form, Json,
};
use serde::Deserialize;
use sqlx::Row;
use time::Duration;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tower_sessions::{ExpiredDeletion, Expiry, SessionManagerLayer, SqliteStore};

type Router = axum::Router<Arc<App>>;

#[derive(Deserialize)]
struct IdQuery {
    id: Id,
}

struct App {
    database: DatabasePool,
    config: AppConfig,
}

pub async fn run(port: u16, database: DatabasePool, config: AppConfig) -> color_eyre::Result<()> {
    let addr = format!("0.0.0.0:{}", port);
    info!("Starting the server on {}", addr);

    let app = Arc::new(App { database, config });

    // Session layer
    let session_store = SqliteStore::new(app.database.clone());
    session_store.migrate().await?;

    let deletion_task = tokio::task::spawn(
        session_store
            .clone()
            .continuously_delete_expired(tokio::time::Duration::from_secs(60)),
    );

    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false)
        .with_expiry(Expiry::OnInactivity(Duration::days(1)));

    // Auth service
    let backend = crate::database::auth::Backend::new(app.database.clone());
    let auth_layer = AuthManagerLayerBuilder::new(backend, session_layer).build();

    let router = Router::new()
        .route("/", get(get_root))
        .merge(auth::router());

    let router = player::route(router);
    let router = music::route(router);
    let router = group::route(router);
    let router = level::route(router);

    let router = router
        .layer(TraceLayer::new_for_http())
        .layer(
            CorsLayer::new()
                .allow_origin(tower_http::cors::Any)
                .allow_headers(tower_http::cors::Any),
        )
        .layer(auth_layer)
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

async fn get_auth(key: Option<ApiKey>, database: &DatabasePool) -> Result<AuthorityLevel> {
    let Some(key) = key else {
        return Ok(AuthorityLevel::Unauthorized);
    };

    let row = sqlx::query("SELECT submit, admin FROM keys WHERE key = ?")
        .bind(key.0)
        .fetch_optional(database)
        .await?;
    let Some(row) = row else {
        return Ok(AuthorityLevel::Unauthorized);
    };

    let submit: bool = row.try_get("submit")?;
    let admin: bool = row.try_get("admin")?;

    let auth = if admin {
        AuthorityLevel::Admin
    } else if submit {
        AuthorityLevel::Submit
    } else {
        AuthorityLevel::Read
    };
    Ok(auth)
}

fn check_auth(auth: AuthorityLevel, required: AuthorityLevel) -> Result<()> {
    if let AuthorityLevel::Unauthorized = auth {
        Err(RequestError::Unathorized)
    } else if auth < required {
        Err(RequestError::Forbidden)
    } else {
        Ok(())
    }
}

fn validate_name(name: String) -> Result<String> {
    let name = name.trim().to_owned();
    if name.is_empty() {
        return Err(RequestError::InvalidName(name));
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
        Err(err) => return Err(RequestError::FileNotFound(format!("{}", err))),
    };
    // convert the `AsyncRead` into a `Stream`
    let stream = tokio_util::io::ReaderStream::new(file);
    // convert the `Stream` into an `Body`
    let body = Body::from_stream(stream);

    let headers = [
        (header::CONTENT_TYPE, content_type),
        (
            header::CONTENT_DISPOSITION,
            format!("attachment; filename={:?}", filename),
        ),
    ];

    Ok((headers, body))
}

fn content_mp3() -> String {
    "audio/mpeg".to_owned()
}

fn content_wav() -> String {
    "audio/wav".to_owned()
}

fn content_level() -> String {
    "application/octet-stream".to_owned()
}
