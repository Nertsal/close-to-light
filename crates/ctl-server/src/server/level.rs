use super::*;

use crate::database::types::LevelRow;

use ctl_core::{types::NewLevel, ScoreEntry, SubmitScore};

pub fn route(router: Router) -> Router {
    router
        .route("/levels", get(level_list))
        .route("/level/:level_id", get(level_get))
        .route(
            "/level/:level_id/scores",
            get(fetch_scores).post(submit_score),
        )
        .route("/level/:level_id/download", get(download))
        .route("/level/create", post(level_create))
}

// TODO: group list instead?
async fn level_list(State(app): State<Arc<App>>) -> Result<Json<Vec<LevelInfo>>> {
    let levels: Vec<LevelRow> = sqlx::query_as("SELECT * FROM levels")
        .fetch_all(&app.database)
        .await?;

    let authors: Vec<(Id, UserInfo)> = sqlx::query(
        "
    SELECT level_id, users.user_id, username
    FROM level_authors
    JOIN users ON level_authors.user_id = users.user_id
            ",
    )
    .try_map(|row: DBRow| {
        Ok((
            row.try_get("music_id")?,
            UserInfo {
                id: row.try_get("user_id")?,
                name: row.try_get("username")?,
            },
        ))
    })
    .fetch_all(&app.database)
    .await?;

    let mut result = Vec::with_capacity(levels.len());
    for level in levels {
        let authors = authors
            .iter()
            .filter(|(level_id, _)| *level_id == level.level_id)
            .map(|(_, user)| user.clone())
            .collect();

        result.push(LevelInfo {
            id: level.level_id,
            name: level.name,
            hash: level.hash,
            authors,
        });
    }

    Ok(Json(result))
}

async fn level_get(
    State(app): State<Arc<App>>,
    Path(level_id): Path<Id>,
) -> Result<Json<LevelInfo>> {
    let level: LevelRow = sqlx::query_as("SELECT * FROM levels WHERE level_id = ?")
        .bind(level_id)
        .fetch_one(&app.database)
        .await?;

    let authors: Vec<UserInfo> = sqlx::query(
        "
SELECT players.player_id, name
FROM level_authors
JOIN players ON level_authors.player_id = players.player_id
        ",
    )
    .try_map(|row: DBRow| {
        Ok(UserInfo {
            id: row.try_get("player_id")?,
            name: row.try_get("name")?,
        })
    })
    .fetch_all(&app.database)
    .await?;

    Ok(Json(LevelInfo {
        id: level_id,
        name: level.name,
        hash: level.hash,
        authors,
    }))
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
SELECT username, score, extra_info
FROM scores
JOIN users ON scores.user_id = users.user_id
WHERE level_id = ?
        ",
    )
    .bind(level_id)
    .try_map(|row: DBRow| {
        Ok(ScoreEntry {
            user: UserInfo {
                id: row.try_get("user_id")?,
                name: row.try_get("username")?,
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
    Json(score): Json<SubmitScore>,
) -> Result<()> {
    let user = check_user(&session).await?;

    // Check that the level exists
    let check = sqlx::query("SELECT null FROM levels WHERE level_id = ?")
        .bind(level_id)
        .fetch_optional(&app.database)
        .await?;
    if check.is_none() {
        return Err(RequestError::NoSuchLevel(level_id));
    }

    // Insert new score
    // TODO: Keep only highest score
    sqlx::query("INSERT INTO scores (level_id, user_id, score, extra_info) VALUES (?, ?, ?, ?)")
        .bind(level_id)
        .bind(user.user_id)
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
