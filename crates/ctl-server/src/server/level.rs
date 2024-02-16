use super::*;

use ctl_core::{types::NewLevel, ScoreEntry};

pub fn route(router: Router) -> Router {
    router
        // .route("/level/:level_id", patch(level_update))
        .route(
            "/level/:level_id/scores",
            get(fetch_scores).post(submit_score),
        )
        .route("/level/:level_id/download", get(download))
        .route("/level/create", post(level_create))
}

async fn level_create(
    session: AuthSession,
    State(app): State<Arc<App>>,
    Json(level): Json<NewLevel>,
) -> Result<Json<Id>> {
    check_auth(&session, &app, AuthorityLevel::User).await?;

    // Check that group exists and the player has rights to add levels to it
    let group = sqlx::query("SELECT null FROM groups WHERE group_id = ?")
        .bind(level.group)
        .fetch_optional(&app.database)
        .await?;
    if group.is_none() {
        return Err(RequestError::NoSuchGroup(level.group));
    }

    let level_id = todo!();

    Ok(Json(level_id))
}

async fn fetch_scores(
    State(app): State<Arc<App>>,
    Path(level_id): Path<Id>,
) -> Result<Json<Vec<ScoreEntry>>> {
    // Check that the level exists
    let check = sqlx::query("SELECT null FROM levels WHERE level_id = ?")
        .bind(level_id)
        .fetch_optional(&app.database)
        .await?;
    if check.is_none() {
        return Err(RequestError::NoSuchLevel(level_id));
    }

    // Fetch scores
    let scores = sqlx::query(
        "
SELECT name, score, extra_info
FROM scores
JOIN players ON scores.player_id = players.player_id
WHERE level_id = ?
        ",
    )
    .bind(level_id)
    .try_map(|row: DBRow| {
        Ok(ScoreEntry {
            player: PlayerInfo {
                id: row.try_get("player_id")?,
                name: row.try_get("name")?,
            },
            score: row.try_get("score")?,
            extra_info: row.try_get("extra_info")?,
        })
    })
    .fetch_all(&app.database)
    .await?;

    Ok(Json(scores))
}

async fn submit_score(
    session: AuthSession,
    State(app): State<Arc<App>>,
    Path(level_id): Path<Id>,
    player_key: PlayerKey,
    Json(score): Json<ScoreEntry>,
) -> Result<()> {
    check_auth(&session, &app, AuthorityLevel::User).await?;

    // Check that the level exists
    let check = sqlx::query("SELECT null FROM levels WHERE level_id = ?")
        .bind(level_id)
        .fetch_optional(&app.database)
        .await?;
    if check.is_none() {
        return Err(RequestError::NoSuchLevel(level_id));
    }

    // Authorize player
    let (real_key, player_name): (String, String) =
        sqlx::query("SELECT key, name FROM players WHERE player_id = ?")
            .bind(score.player.id)
            .try_map(|row: DBRow| Ok((row.try_get("key")?, row.try_get("name")?)))
            .fetch_one(&app.database)
            .await?;

    if real_key != player_key.0 {
        // Incorrect key
        return Err(RequestError::InvalidPlayer);
    }

    if player_name != score.player.name {
        // Name changed
        sqlx::query("UPDATE players SET name = ? WHERE player_id = ?")
            .bind(&score.player.name)
            .bind(score.player.id)
            .execute(&app.database)
            .await?;
    }

    // Insert new score
    // TODO: Keep only highest score
    sqlx::query("INSERT INTO scores (level_id, player_id, score, extra_info) VALUES (?, ?, ?, ?)")
        .bind(level_id)
        .bind(score.player.id)
        .bind(score.score)
        .bind(&score.extra_info)
        .execute(&app.database)
        .await?;

    Ok(())
}

async fn download(
    State(app): State<Arc<App>>,
    Path(level_id): Path<Id>,
) -> Result<impl IntoResponse> {
    let level_row = sqlx::query("SELECT file_path FROM levels WHERE level_id = ?")
        .bind(level_id)
        .fetch_optional(&app.database)
        .await?;

    let Some(row) = level_row else {
        return Err(RequestError::NoSuchLevel(level_id));
    };

    let file_path: String = row.try_get("file_path")?;
    let file_path = PathBuf::from(file_path);

    send_file(file_path, content_level()).await
}
