use ctl_core::types::NewArtist;

use super::*;

pub fn router() -> Router {
    Router::new().route("/artists", post(artist_create))
}

pub async fn artist_create(
    session: AuthSession,
    State(app): State<Arc<App>>,
    Form(artist): Form<NewArtist>,
) -> Result<Json<Id>> {
    check_auth(&session, &app, AuthorityLevel::Admin).await?;

    let artist_id: Id = sqlx::query(
        "INSERT INTO artists (name, romanized_name, user_id) VALUES (?, ?, ?) RETURNING artist_id",
    )
    .bind(&artist.name)
    .bind(&artist.romanized_name)
    .bind(artist.user)
    .try_map(|row: DBRow| row.try_get("artist_id"))
    .fetch_one(&app.database)
    .await?;

    Ok(Json(artist_id))
}
