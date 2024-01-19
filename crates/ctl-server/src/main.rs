mod api_key;
mod database;
mod prelude;
mod server;
mod setup;

use self::prelude::*;

use std::path::PathBuf;

const DEFAULT_DATABASE: &str = "sqlite://database.db";
const DEFAULT_LEVELS: &str = "levels";

#[derive(clap::Parser)]
struct Opts {
    port: u16,
}

#[tokio::main]
async fn main() -> Result<()> {
    let opts: Opts = clap::Parser::parse();

    setup::setup()?;

    let database_url: String = dotenv::var("DATABASE_URL").unwrap_or_else(|_| {
        warn!("DATABASE_URL environment variable is not set, using default");
        DEFAULT_DATABASE.to_owned()
    });

    let levels_path: String = dotenv::var("LEVELS_PATH").unwrap_or_else(|_| {
        warn!("LEVELS_PATH environment variable is not set, using default");
        DEFAULT_LEVELS.to_owned()
    });
    let levels_path: PathBuf = PathBuf::from(levels_path);

    let database_pool = setup::connect_database(&database_url)
        .await
        .context(format!("when connecting to the database: {}", database_url))?;

    server::run(opts.port, database_pool, levels_path)
        .await
        .context("server error")
}
