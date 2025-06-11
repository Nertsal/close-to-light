use ctl_core::types::NewMusician;

use super::*;

pub fn router() -> Router {
    Router::new().route("/musicians", post(musician_create))
}

pub async fn musician_create(
    session: AuthSession,
    State(app): State<Arc<App>>,
    Form(musician): Form<NewMusician>,
) -> Result<Json<Id>> {
    check_auth(&session, &app, AuthorityLevel::Admin).await?;

    let artist_id: Id = sqlx::query_scalar(
        "INSERT INTO musicians (name, romanized_name, user_id, created_at) VALUES (?, ?, ?, ?) RETURNING artist_id",
    )
    .bind(&musician.name)
    .bind(&musician.romanized_name)
    .bind(musician.user)
    .bind(OffsetDateTime::now_utc())
    .fetch_one(&app.database)
    .await?;

    Ok(Json(artist_id))
}
