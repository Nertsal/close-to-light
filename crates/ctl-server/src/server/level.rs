use super::*;

use crate::database::types::LevelRow;

use ctl_core::{
    score::{ServerScore, SubmitScore},
    types::MapperInfo,
};

pub fn route(router: Router) -> Router {
    router.route("/level/:level_id", get(level_get)).route(
        "/level/:level_id/scores",
        get(fetch_scores).post(submit_score),
    )
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

    let authors: Vec<LevelAuthorRow> = sqlx::query_as(
        "
SELECT users.user_id, username
FROM level_authors
WHERE level_id = ?
        ",
    )
    .bind(level_id)
    .fetch_all(&app.database)
    .await?;
    let authors = authors.into_iter().map(MapperInfo::from).collect();

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
) -> Result<Json<Vec<ServerScore>>> {
    // Check that the level exists
    let level: Option<LevelRow> = sqlx::query_as("SELECT * FROM levels WHERE level_id = ?")
        .bind(level_id)
        .fetch_optional(&app.database)
        .await?;
    let Some(_level) = level else {
        return Err(RequestError::NoSuchLevel(level_id));
    };

    #[derive(sqlx::FromRow)]
    struct Row {
        #[sqlx(flatten)]
        user: UserRow,
        #[sqlx(flatten)]
        score: ScoreRow,
    }

    // Fetch scores
    let scores: Vec<Row> = sqlx::query_as(
        "
SELECT *
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
        .map(|score| ServerScore {
            user: UserInfo {
                id: score.user.user_id,
                name: score.user.username.into(),
            },
            score: score.score.score,
            submitted_at: score.score.submitted_at,
            meta: score.score.extra_info,
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
    let mut trans = app.database.begin().await?;

    // Check that the level exists
    let level: Option<LevelRow> = sqlx::query_as("SELECT * FROM levels WHERE level_id = ?")
        .bind(level_id)
        .fetch_optional(&mut *trans)
        .await?;
    let Some(level) = level else {
        return Err(RequestError::NoSuchLevel(level_id));
    };

    if score.level_hash != level.hash {
        return Err(RequestError::LevelHashMismatch);
    }

    // Insert new score
    let current: Option<ScoreRow> =
        sqlx::query_as("SELECT * FROM scores WHERE level_id = ? AND user_id = ?")
            .bind(level_id)
            .bind(user.user_id)
            .fetch_optional(&mut *trans)
            .await?;

    if let Some(current) = current {
        if score.score > current.score {
            sqlx::query(
                "UPDATE scores SET score = ?, extra_info = ? WHERE level_id = ? AND user_id = ?",
            )
            .bind(score.score)
            .bind(&score.meta)
            .bind(level_id)
            .bind(user.user_id)
            .execute(&mut *trans)
            .await?;
        }
    } else {
        sqlx::query(
            "INSERT INTO scores (level_id, level_hash, user_id, score, extra_info, submitted_at) VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(level_id)
        .bind(&level.hash)
        .bind(user.user_id)
        .bind(score.score)
        .bind(&score.meta)
        .bind(OffsetDateTime::now_utc())
        .execute(&mut *trans)
        .await?;
    }

    trans.commit().await?;
    Ok(())
}
