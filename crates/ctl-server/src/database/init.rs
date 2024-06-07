use super::types::DatabasePool;

pub async fn init_database(database: &DatabasePool) -> color_eyre::Result<()> {
    sqlx::migrate!().run(database).await?;
    Ok(())
}
