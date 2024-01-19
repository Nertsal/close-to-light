use super::*;

pub async fn create(
    State(app): State<Arc<App>>,
    Json(player_name): Json<String>,
) -> Result<Json<ctl_core::Player>> {
    // Generate a random key
    let key = StringKey::generate(10).inner().to_owned();

    let id = sqlx::query("INSERT INTO players (key, name) VALUES (?, ?) RETURNING player_id")
        .bind(&key)
        .bind(&player_name)
        .try_map(|row: DBRow| row.try_get::<Uuid, _>("player_id"))
        .fetch_one(&app.database)
        .await?;

    Ok(Json(ctl_core::Player {
        id,
        key,
        name: player_name,
    }))
}
