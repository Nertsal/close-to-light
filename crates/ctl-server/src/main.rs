mod api_key;
mod database;
mod prelude;
mod server;
mod setup;

use self::prelude::*;

#[derive(clap::Parser)]
struct Opts {
    port: u16,
}

#[tokio::main]
async fn main() -> Result<()> {
    let opts: Opts = clap::Parser::parse();

    setup::setup()?;

    let database_url =
        dotenv::var("DATABASE_URL").expect("DATABASE_URL environment variable is not set");

    let database_pool = setup::connect_database(&database_url)
        .await
        .context(format!("when connecting to the database: {}", database_url))?;

    server::run(opts.port, database_pool)
        .await
        .context("server error")
}
