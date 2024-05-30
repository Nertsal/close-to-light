use super::*;

use crate::database::types::LevelRow;

use ctl_core::{ScoreEntry, SubmitScore};

pub fn route(router: Router) -> Router {
    router
        .route("/levels", get(level_list))
        .route("/level/:level_id", get(level_get))
        .route(
            "/level/:level_id/scores",
            get(fetch_scores).post(submit_score),
        )
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
            row.try_get("level_id")?,
            UserInfo {
                id: row.try_get("user_id")?,
                name: row.try_get::<String, _>("username")?.into(),
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
            name: level.name.into(),
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
    let level: Option<LevelRow> = sqlx::query_as("SELECT * FROM levels WHERE level_id = ?")
        .bind(level_id)
        .fetch_optional(&app.database)
        .await?;

    let Some(level) = level else {
        return Err(RequestError::NoSuchLevel(level_id));
    };

    let authors: Vec<UserInfo> = sqlx::query(
        "
SELECT users.user_id, username
FROM level_authors
JOIN users ON level_authors.user_id = users.user_id
WHERE level_id = ?
        ",
    )
    .bind(level_id)
    .try_map(|row: DBRow| {
        Ok(UserInfo {
            id: row.try_get("user_id")?,
            name: row.try_get::<String, _>("username")?.into(),
        })
    })
    .fetch_all(&app.database)
    .await?;

    Ok(Json(LevelInfo {
        id: level_id,
        name: level.name.into(),
        hash: level.hash,
        authors,
    }))
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

    #[derive(sqlx::FromRow)]
    struct ScoreRow {
        #[sqlx(flatten)]
        user: UserRow,
        score: Score,
        extra_info: Option<String>,
    }

    // Fetch scores
    let scores: Vec<ScoreRow> = sqlx::query_as(
        "
SELECT users.user_id, username, score, extra_info
FROM scores
JOIN users ON scores.user_id = users.user_id
WHERE level_id = ?
        ",
    )
    .bind(level_id)
    .fetch_all(&app.database)
    .await?;

    let scores = scores
        .into_iter()
        .map(|score| ScoreEntry {
            user: UserInfo {
                id: score.user.user_id,
                name: score.user.username.into(),
            },
            score: score.score,
            extra_info: score.extra_info,
        })
        .collect();

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
