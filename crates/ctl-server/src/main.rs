mod api_key;
mod database;
mod prelude;
mod server;
mod setup;

use self::prelude::*;

use std::path::PathBuf;

const DEFAULT_DATABASE: &str = "sqlite://database.db";
const DEFAULT_GROUPS: &str = "groups";

#[derive(clap::Parser)]
struct Opts {
    port: u16,
}

struct AppConfig {
    groups_path: PathBuf,
}

#[tokio::main]
async fn main() -> Result<()> {
    let opts: Opts = clap::Parser::parse();

    setup::setup()?;

    let database_url: String = dotenv::var("DATABASE_URL").unwrap_or_else(|_| {
        warn!("DATABASE_URL environment variable is not set, using default");
        DEFAULT_DATABASE.to_owned()
    });

    let groups_path: String = dotenv::var("GROUPS_PATH").unwrap_or_else(|_| {
        warn!("GROUPS_PATH environment variable is not set, using default");
        DEFAULT_GROUPS.to_owned()
    });
    let groups_path: PathBuf = PathBuf::from(groups_path);

    info!("Database: {}", database_url);
    info!("Groups: {:?}", groups_path);

    let config = AppConfig { groups_path };

    let database_pool = setup::connect_database(&database_url)
        .await
        .context(format!("when connecting to the database: {}", database_url))?;

    server::run(opts.port, database_pool, config)
        .await
        .context("server error")
}
