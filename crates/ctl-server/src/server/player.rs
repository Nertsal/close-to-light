use super::*;

pub fn route(router: Router) -> Router {
    router.route("/player/create", post(create))
}

pub async fn create(
    State(app): State<Arc<App>>,
    Json(player_name): Json<String>,
) -> Result<Json<ctl_core::Player>> {
    // Generate a random key and uuid
    let key = StringKey::generate(10).inner().to_owned();
    let id = Uuid::new_v4();

    sqlx::query("INSERT INTO players (player_id, key, name) VALUES (?, ?, ?)")
        .bind(id)
        .bind(&key)
        .bind(&player_name)
        .execute(&app.database)
        .await?;

    Ok(Json(ctl_core::Player {
        id,
        key,
        name: player_name,
    }))
}
