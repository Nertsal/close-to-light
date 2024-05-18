use super::*;

use crate::database::types::{GroupRow, LevelRow};

use ctl_core::{model::Level, types::NewLevel, ScoreEntry, SubmitScore};

const LEVEL_SIZE_LIMIT: usize = 1024 * 1024; // 1 MB

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

/// Create a new level or upload a new version of an existing one.
async fn level_create(
    session: AuthSession,
    State(app): State<Arc<App>>,
    Query(level): Query<NewLevel>,
    body: Body,
) -> Result<Json<Id>> {
    let user = check_user(&session).await?;

    // Check that group exists
    let group: Option<GroupRow> = sqlx::query_as("SELECT * FROM groups WHERE group_id = ?")
        .bind(level.group)
        .fetch_optional(&app.database)
        .await?;
    let Some(group) = group else {
        return Err(RequestError::NoSuchGroup(level.group));
    };

    // Check if the player has rights to add levels to the group
    if user.user_id != group.owner_id {
        return Err(RequestError::Forbidden);
    }

    let data = axum::body::to_bytes(body, LEVEL_SIZE_LIMIT)
        .await
        .expect("not bytes idk");

    // Calculate level hash
    let hash = ctl_core::util::calculate_hash(&data);

    // Check if such a level already exists
    let conflict = sqlx::query("SELECT null FROM levels WHERE hash = ?")
        .bind(&hash)
        .fetch_optional(&app.database)
        .await?;
    if conflict.is_some() {
        return Err(RequestError::LevelAlreadyExists);
    }

    // Validate level contents
    let _parsed_level: Level =
        bincode::deserialize(&data).map_err(|_| RequestError::InvalidLevel)?;
    // TODO

    let level_id = if let Some(level_id) = level.level_id {
        let res = sqlx::query("UPDATE levels SET hash = ? WHERE level_id = ?")
            .bind(&hash)
            .bind(level_id)
            .execute(&app.database)
            .await?;
        if res.rows_affected() == 0 {
            return Err(RequestError::NoSuchLevel(level_id));
        }

        level_id
    } else {
        // Commit to database
        let level_id: Id = sqlx::query(
            "INSERT INTO levels (name, group_id, hash) VALUES (?, ?, ?) RETURNING level_id",
        )
        .bind(&level.name)
        .bind(level.group)
        .bind(&hash)
        .try_map(|row: DBRow| row.try_get("level_id"))
        .fetch_one(&app.database)
        .await?;
        debug!("New level committed to the database");

        // Add user as author
        sqlx::query("INSERT INTO level_authors (user_id, level_id) VALUES (?, ?)")
            .bind(user.user_id)
            .bind(level_id)
            .execute(&app.database)
            .await?;

        level_id
    };

    // Check path
    let dir_path = app.config.groups_path.join("levels");
    std::fs::create_dir_all(&dir_path)?;
    let path = dir_path.join(level_id.to_string());
    debug!("Saving level file at {:?}", path);

    // let Some(music_path) = path.to_str() else {
    //     error!("Music path is not valid unicode");
    //     return Err(RequestError::Internal);
    // };

    if level.level_id.is_none() && path.exists() {
        error!("Duplicate level ID generated: {}", level_id);
        return Err(RequestError::Internal);
    }

    // Write to file
    std::fs::write(path, data)?;
    debug!("Saved level file successfully");

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
                name: row.try_get::<String, _>("username")?.into(),
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
    let level_row = sqlx::query("SELECT null FROM levels WHERE level_id = ?")
        .bind(level_id)
        .fetch_optional(&app.database)
        .await?;

    if level_row.is_none() {
        return Err(RequestError::NoSuchLevel(level_id));
    }

    let file_path = app
        .config
        .groups_path
        .join("levels")
        .join(level_id.to_string());
    send_file(file_path, content_level()).await
}
