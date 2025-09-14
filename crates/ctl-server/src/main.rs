mod database;
mod prelude;
mod server;
mod setup;

use self::prelude::*;

use std::path::PathBuf;

use ctl_core::prelude::toml;
use serde::Deserialize;

const DEFAULT_DATABASE: &str = "sqlite://server-data/database.db";
const DEFAULT_LEVEL_SETS: &str = "server-data/level_sets";
const DEFAULT_SECRETS: &str = "secrets/secrets.toml";

#[derive(clap::Parser)]
struct Opts {
    port: u16,
}

struct AppConfig {
    level_sets_path: PathBuf,
    proxy: Option<String>,
}

#[derive(Deserialize)]
struct AppSecrets {
    server_addr: String,
    discord: DiscordSecrets,
}

#[derive(Deserialize)]
struct DiscordSecrets {
    client_id: Box<str>,
    client_secret: Box<str>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let opts: Opts = clap::Parser::parse();

    setup::setup()?;

    let database_url: String = dotenv::var("DATABASE_URL").unwrap_or_else(|_| {
        warn!("DATABASE_URL environment variable is not set, using default");
        DEFAULT_DATABASE.to_owned()
    });

    let level_sets_path: String = dotenv::var("LEVEL_SETS_PATH").unwrap_or_else(|_| {
        warn!("LEVEL_SETS_PATH environment variable is not set, using default");
        DEFAULT_LEVEL_SETS.to_owned()
    });
    let level_sets_path: PathBuf = PathBuf::from(level_sets_path);

    let secrets_path: String =
        dotenv::var("SECRETS_PATH").unwrap_or_else(|_| DEFAULT_SECRETS.to_owned());
    let secrets_path: PathBuf = PathBuf::from(secrets_path);

    let proxy = dotenv::var("PROXY").ok();

    info!("Database: {}", database_url);
    info!("Level sets: {:?}", level_sets_path);

    ensure_exists(&database_url, true)?;
    ensure_exists(&level_sets_path, false)?;

    let config = AppConfig {
        level_sets_path,
        proxy,
    };

    let secrets: AppSecrets = toml::from_str(&std::fs::read_to_string(&secrets_path)?)?;

    let database_pool = setup::connect_database(&database_url)
        .await
        .context(format!("when connecting to the database: {database_url}"))?;

    server::run(opts.port, database_pool, config, secrets)
        .await
        .context("server error")
}

/// Ensures that the path to the file or directory exists.
/// If `create_file` is true and the path is a file, also creates an empty file.
fn ensure_exists(path: impl AsRef<std::path::Path>, create_file: bool) -> Result<()> {
    let mut path = path.as_ref();

    if let Ok(tail) = path.strip_prefix("sqlite:") {
        path = tail;
    };

    let dir_path = if !path.is_dir() {
        match path.parent() {
            Some(parent) => parent,
            None => return Ok(()),
        }
    } else {
        path
    };

    std::fs::create_dir_all(dir_path)?;
    if create_file && !path.exists() && !path.is_dir() {
        std::fs::File::create(path)?;
    }

    Ok(())
}
