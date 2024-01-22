use crate::{database::DatabasePool, prelude::*};

use tracing::level_filters::LevelFilter;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub fn setup() -> Result<()> {
    // Panic handler
    color_eyre::install()?;

    // Setup logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::builder()
                .with_default_directive(LevelFilter::DEBUG.into())
                .from_env_lossy(),
        )
        .with(
            tracing_subscriber::fmt::layer()
                .pretty()
                .with_writer(std::io::stdout),
        )
        .init();

    // Load .env
    dotenv::dotenv().ok(); // Error if file does not exist: ignore

    // SQL drivers
    sqlx::any::install_default_drivers();

    Ok(())
}

pub async fn connect_database(url: &str) -> Result<DatabasePool> {
    tracing::info!("Connecting to database {}", url);
    let pool = DatabasePool::connect(url).await?;

    crate::database::init_database(&pool)
        .await
        .context("when initializing the database")?;

    Ok(pool)
}
