mod group;
mod level;
mod player;

#[cfg(test)]
mod tests;

use crate::{
    api_key::*,
    database::{DBRow, DatabasePool, Id, RequestError, RequestResult as Result},
    prelude::*,
};

use std::path::PathBuf;

use ctl_core::prelude::{GroupInfo, LevelInfo, MusicInfo, PlayerInfo};

use axum::{
    body::Body,
    extract::{Path, Query, State},
    http::{header, StatusCode},
    response::IntoResponse,
    routing::{delete, get, post},
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::{types::Uuid, Row};
use tower_http::{cors::CorsLayer, trace::TraceLayer};

type Router = axum::Router<Arc<DatabasePool>>;

#[derive(Deserialize)]
struct PlayerIdQuery {
    player_id: Id,
}

pub async fn run(
    port: u16,
    database_pool: DatabasePool,
    levels_path: PathBuf,
) -> color_eyre::Result<()> {
    let addr = format!("0.0.0.0:{}", port);
    info!("Starting the server on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .context("when binding a tcp listener")?;

    axum::serve(listener, app(Arc::new(database_pool))).await?;
    Ok(())
}

fn app(database_pool: Arc<DatabasePool>) -> axum::Router {
    let router = Router::new()
        .route("/", get(get_root))
        .route("/player/create", post(player::create));

    let router = level::route(router);
    let router = group::route(router);

    router
        .layer(TraceLayer::new_for_http())
        .layer(
            CorsLayer::new()
                .allow_origin(tower_http::cors::Any)
                .allow_headers(tower_http::cors::Any),
        )
        .with_state(database_pool)
}

async fn get_root() -> &'static str {
    "Hello, world"
}

async fn upload_file(
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

fn content_level() -> String {
    "application/octet-stream".to_owned()
}
