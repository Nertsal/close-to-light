use super::*;

use crate::database::types::LevelRow;

use axum::{body::Bytes, extract::DefaultBodyLimit};
use ctl_core::types::{LevelSetFull, LevelSetsQuery};

const LEVEL_SET_SIZE_LIMIT: usize = 1024 * 1024; // 1 MB
const LEVEL_SETS_PER_USER: usize = 5;
const LEVEL_SETS_PER_USER_PER_SONG: usize = 1;
/// seconds
const LEVEL_MIN_DURATION: f32 = 30.0;

pub fn route(router: Router) -> Router {
    router
        .route("/level_sets", get(level_set_list))
        .route("/level_set/:level_set_id", get(level_set_get))
        .route("/level_set/:level_set_id/download", get(download))
        .route("/level_set/create", post(level_set_create))
        .layer(DefaultBodyLimit::max(LEVEL_SET_SIZE_LIMIT))
}

// TODO: filter, sort, limit, pages
async fn level_set_list(
    State(app): State<Arc<App>>,
    Query(query): Query<LevelSetsQuery>,
) -> Result<Json<Vec<LevelSetInfo>>> {
    // TODO: lazy?
    let music = super::music::music_list(
        State(app.clone()),
        Query(super::music::GetMusicQuery { level_set_id: None }),
    )
    .await?
    .0;

    #[derive(sqlx::FromRow)]
    struct LevelGroupRow {
        #[sqlx(flatten)]
        group: LevelSetRow,
        #[sqlx(flatten)]
        level: LevelRow,
    }

    let query = if query.recommended {
        // TODO
        // "SELECT * FROM levels JOIN (
        //     SELECT * FROM groups_recommended JOIN groups ON groups_recommended.group_id = groups.group_id
        // ) AS groups ON levels.group_id = groups.group_id"
        return Err(RequestError::Internal);
    } else {
        "SELECT * FROM levels JOIN level_sets ON levels.level_set_id = level_sets.level_set_id"
    };

    let levels: Vec<LevelGroupRow> = sqlx::query_as(query).fetch_all(&app.database).await?;

    #[derive(sqlx::FromRow)]
    struct AuthorRow {
        level_id: Id,
        #[sqlx(flatten)]
        user: UserRow,
    }

    let authors: Vec<AuthorRow> = sqlx::query_as(
        "
    SELECT level_id, users.user_id, username
    FROM level_authors
    JOIN users ON level_authors.user_id = users.user_id
        ",
    )
    .fetch_all(&app.database)
    .await?;

    let mut groups = Vec::<LevelSetInfo>::new();
    for level_row in levels {
        let authors: Vec<UserInfo> = authors
            .iter()
            .filter(|author| author.level_id == level_row.level.level_id)
            .map(|author| author.user.clone().into())
            .collect();

        let owner = match authors
            .iter()
            .find(|user| user.id == level_row.group.owner_id)
        {
            Some(user) => user.clone(),
            None => {
                sqlx::query("SELECT user_id, username FROM users WHERE user_id = ?")
                    .bind(level_row.group.owner_id)
                    .try_map(|row: DBRow| {
                        Ok(UserInfo {
                            id: row.try_get("user_id")?,
                            name: row.try_get::<String, _>("username")?.into(),
                        })
                    })
                    .fetch_one(&app.database)
                    .await?
            }
        };

        let level_info = LevelInfo {
            id: level_row.level.level_id,
            name: level_row.level.name.into(),
            hash: level_row.level.hash,
            authors,
        };

        let group_i = groups
            .iter()
            .position(|g| g.id == level_row.level.level_set_id)
            .unwrap_or_else(|| {
                groups.push(LevelSetInfo {
                    id: level_row.level.level_set_id,
                    music: music
                        .iter()
                        .find(|music| music.id == level_row.group.music_id)
                        .cloned()
                        .unwrap_or_default(), // Default should never be reached TODO: warning or smth
                    owner,
                    levels: Vec::new(),
                    hash: String::new(), // TODO
                });
                groups.len() - 1
            });
        groups[group_i].levels.push(level_info);
    }

    Ok(Json(groups))
}

async fn level_set_get(
    State(app): State<Arc<App>>,
    Path(level_set_id): Path<Id>,
) -> Result<Json<LevelSetInfo>> {
    let group_row: Option<LevelSetRow> =
        sqlx::query_as("SELECT * FROM level_sets WHERE level_set_id = ?")
            .bind(level_set_id)
            .fetch_optional(&app.database)
            .await?;
    let Some(group_row) = group_row else {
        return Err(RequestError::NoSuchLevelSet(level_set_id));
    };

    let music = music::music_get(State(app.clone()), Path(group_row.music_id))
        .await?
        .0;

    let level_rows: Vec<LevelRow> = sqlx::query_as(
        "SELECT * FROM levels WHERE level_set_id = ? AND enabled = TRUE ORDER BY ord",
    )
    .bind(level_set_id)
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

    let owner = match authors
        .iter()
        .find(|(_, user)| user.id == group_row.owner_id)
    {
        Some((_, user)) => user.clone(),
        None => {
            sqlx::query("SELECT user_id, username FROM users WHERE user_id = ?")
                .bind(group_row.owner_id)
                .try_map(|row: DBRow| {
                    Ok(UserInfo {
                        id: row.try_get("user_id")?,
                        name: row.try_get::<String, _>("username")?.into(),
                    })
                })
                .fetch_one(&app.database)
                .await?
        }
    };

    let mut levels = Vec::new();
    for level in level_rows {
        let authors = authors
            .iter()
            .filter(|(id, _)| *id == level.level_id)
            .map(|(_, player)| player.clone())
            .collect();
        levels.push(LevelInfo {
            id: level.level_id,
            name: level.name.into(),
            hash: level.hash,
            authors,
        });
    }

    Ok(Json(LevelSetInfo {
        id: level_set_id,
        music,
        owner,
        levels,
        hash: group_row.hash,
    }))
}

#[derive(Deserialize)]
struct LevelSetCreateQuery {
    music_id: Id,
}

async fn level_set_create(
    session: AuthSession,
    State(app): State<Arc<App>>,
    Query(query): Query<LevelSetCreateQuery>,
    data: Bytes,
) -> Result<Json<Id>> {
    let user = check_user(&session).await?;

    // NOTE: Not parsing into Rc, because we cant hold it across an await point
    // also we want to mutate it
    let parsed_level_set: LevelSetFull =
        bincode::deserialize(&data).map_err(|_| RequestError::InvalidLevel)?;
    validate_level_set(&parsed_level_set)?;

    music::music_exists(&app, query.music_id).await?;

    let level_set_id = if parsed_level_set.meta.id != 0 {
        let id = parsed_level_set.meta.id;
        update_level_set(&app, user, parsed_level_set).await?;
        id
    } else {
        new_level_set(&app, user, query.music_id, parsed_level_set).await?
    };

    Ok(Json(level_set_id))
}

async fn update_level_set(
    app: &App,
    user: &User,
    mut parsed_level_set: LevelSetFull,
) -> Result<()> {
    let level_set_id = parsed_level_set.meta.id;
    let level_set: Option<LevelSetRow> =
        sqlx::query_as("SELECT * FROM level_sets WHERE level_set_id = ?")
            .bind(level_set_id)
            .fetch_optional(&app.database)
            .await?;
    let group = level_set.ok_or(RequestError::NoSuchLevelSet(level_set_id))?;

    // Check if the player has rights to change the group
    if user.user_id != group.owner_id {
        return Err(RequestError::Forbidden);
    }

    // Verify owner
    parsed_level_set.meta.owner = UserInfo {
        id: user.user_id,
        name: user.username.clone().into(),
    };

    // Update levels
    let old_levels: Vec<LevelRow> = sqlx::query_as("SELECT * FROM levels WHERE level_set_id = ?")
        .bind(level_set_id)
        .fetch_all(&app.database)
        .await?;

    // Disable removed levels
    for old_level in &old_levels {
        if !parsed_level_set
            .meta
            .levels
            .iter()
            .any(|level| level.id == old_level.level_id)
        {
            sqlx::query("UPDATE levels SET enabled = 0 WHERE level_id = ?")
                .bind(old_level.level_id)
                .execute(&app.database)
                .await?;
        }
    }

    // Set new levels
    for ((order, level), level_meta) in parsed_level_set
        .data
        .levels
        .iter_mut()
        .enumerate()
        .zip(&mut parsed_level_set.meta.levels)
    {
        let order = order as i64;
        level_meta.hash = level.calculate_hash(); // Make sure the hash is valid
        if level_meta.id == 0 {
            // Create a new level
            level_meta.id = sqlx::query_scalar(
                "INSERT INTO levels (hash, level_set_id, enabled, name, ord, created_at)
                 VALUES (?, ?, ?, ?, ?, ?) RETURNING level_id",
            )
            .bind(&level_meta.hash)
            .bind(level_set_id)
            .bind(true)
            .bind(level_meta.name.as_ref())
            .bind(order)
            .bind(OffsetDateTime::now_utc())
            .fetch_one(&app.database)
            .await?;
        } else {
            let old_level: Option<&LevelRow> = old_levels
                .iter()
                .find(|old_level| old_level.level_id == level_meta.id);
            if let Some(old_level) = old_level {
                // Update an existing level
                if old_level.hash != level_meta.hash {
                    // Reset the leaderboard
                    // TODO: make a new endpoint to delete old scores maybe?
                    sqlx::query("DELETE FROM scores WHERE level_id = ?")
                        .bind(old_level.level_id)
                        .execute(&app.database)
                        .await?;
                }

                // Update
                sqlx::query(
                "UPDATE levels SET hash = ?, name = ?, ord = ? WHERE level_id = ? AND level_set_id = ?",
                )
                .bind(&level_meta.hash)
                .bind(level_meta.name.as_ref())
                .bind(order)
                .bind(level_meta.id)
                .bind(level_set_id)
                .execute(&app.database)
                .await?;
            } else {
                // TODO: maybe invalidate request
            }
        }
    }

    // Disallow further mutation to make sure the hash is valid
    let parsed_level_set = parsed_level_set;
    let hash = parsed_level_set.data.calculate_hash();

    // Update level_set
    sqlx::query("UPDATE level_sets SET hash = ? WHERE level_set_id = ?")
        .bind(&hash)
        .bind(level_set_id)
        .execute(&app.database)
        .await?;

    // Check path
    let dir_path = app.config.level_sets_path.join("levels");
    std::fs::create_dir_all(&dir_path)?;
    let path = dir_path.join(level_set_id.to_string());
    debug!("Saving level_set file at {:?}", path);

    if !path.exists() {
        error!(
            "Updating a level_set but it is not present in the file system: {}",
            level_set_id
        );
    }

    // Write to file
    let data = bincode::serialize(&parsed_level_set).map_err(|_| RequestError::Internal)?;
    std::fs::write(path, data)?;
    debug!("Saved level_set file successfully");

    Ok(())
}

async fn new_level_set(
    app: &App,
    user: &User,
    music_id: Id,
    mut parsed_level_set: LevelSetFull,
) -> Result<Id> {
    // Check if the user already has level_sets
    let user_groups: Vec<LevelSetRow> =
        sqlx::query_as("SELECT * FROM level_sets WHERE owner_id = ?")
            .bind(user.user_id)
            .fetch_all(&app.database)
            .await?;
    if user_groups.len() >= LEVEL_SETS_PER_USER {
        return Err(RequestError::TooManyGroups);
    }
    if user_groups
        .iter()
        .filter(|group| group.music_id == music_id)
        .count()
        >= LEVEL_SETS_PER_USER_PER_SONG
    {
        return Err(RequestError::TooManyGroupsForSong);
    }

    // Check if such a level already exists
    for (level, level_meta) in parsed_level_set
        .data
        .levels
        .iter_mut()
        .zip(&mut parsed_level_set.meta.levels)
    {
        level_meta.hash = level.calculate_hash();
        let conflict = sqlx::query("SELECT null FROM levels WHERE hash = ?")
            .bind(&level_meta.hash)
            .fetch_optional(&app.database)
            .await?;
        if conflict.is_some() {
            return Err(RequestError::LevelAlreadyExists);
        }
    }

    // Verify owner
    parsed_level_set.meta.owner = UserInfo {
        id: user.user_id,
        name: user.username.clone().into(),
    };

    let current_time = OffsetDateTime::now_utc();

    // Create group
    let hash = parsed_level_set.data.calculate_hash();
    let level_set_id: Id = sqlx::query_scalar(
        "INSERT INTO level_sets (music_id, owner_id, hash, created_at)
         VALUES (?, ?, ?, ?) RETURNING level_set_id",
    )
    .bind(music_id)
    .bind(user.user_id)
    .bind(&hash)
    .bind(current_time)
    .fetch_one(&app.database)
    .await?;
    parsed_level_set.meta.id = level_set_id;

    // Create levels
    for (order, level_meta) in parsed_level_set.meta.levels.iter_mut().enumerate() {
        let order = order as i64;

        // Check if such a level already exists
        let conflict = sqlx::query("SELECT null FROM levels WHERE hash = ?")
            .bind(&level_meta.hash)
            .fetch_optional(&app.database)
            .await?;
        if conflict.is_some() {
            return Err(RequestError::LevelAlreadyExists);
        }

        level_meta.id = sqlx::query_scalar(
            "INSERT INTO levels (hash, level_set_id, enabled, name, ord, created_at)
             VALUES (?, ?, ?, ?, ?, ?) RETURNING level_id",
        )
        .bind(&level_meta.hash)
        .bind(level_set_id)
        .bind(true)
        .bind(level_meta.name.as_ref())
        .bind(order)
        .bind(current_time)
        .fetch_one(&app.database)
        .await?;

        level_meta.authors = vec![UserInfo {
            id: user.user_id,
            name: user.username.clone().into(),
        }];
        sqlx::query("INSERT INTO level_authors (level_set_id, level_id, user_id) VALUES (?, ?, ?)")
            .bind(level_set_id)
            .bind(level_meta.id)
            .bind(user.user_id)
            .execute(&app.database)
            .await?;
    }

    // Disallow further mutation to make sure the hash is valid
    let parsed_group = parsed_level_set;
    let hash = parsed_group.data.calculate_hash();

    // Update hash
    sqlx::query("UPDATE level_sets SET hash = ? WHERE level_set_id = ?")
        .bind(&hash)
        .bind(level_set_id)
        .execute(&app.database)
        .await?;

    // Check path
    let dir_path = app.config.level_sets_path.join("levels");
    std::fs::create_dir_all(&dir_path)?;
    let path = dir_path.join(level_set_id.to_string());
    debug!("Saving level_set file at {:?}", path);

    if path.exists() {
        error!("Duplicate level_set ID generated: {}", level_set_id);
        return Err(RequestError::Internal);
    }

    // Write to file
    let data = bincode::serialize(&parsed_group).map_err(|_| RequestError::Internal)?;
    std::fs::write(path, data)?;
    debug!("Saved level_set file successfully");

    Ok(level_set_id)
}

async fn download(
    State(app): State<Arc<App>>,
    Path(level_set_id): Path<Id>,
) -> Result<impl IntoResponse> {
    let level_row = sqlx::query("SELECT null FROM level_sets WHERE level_set_id = ?")
        .bind(level_set_id)
        .fetch_optional(&app.database)
        .await?;

    if level_row.is_none() {
        return Err(RequestError::NoSuchLevelSet(level_set_id));
    }

    let file_path = app
        .config
        .level_sets_path
        .join("levels")
        .join(level_set_id.to_string());
    send_file(file_path, content_level()).await
}

// TODO: move to core, so the client can reuse it
fn validate_level_set(level_set: &LevelSetFull) -> Result<()> {
    if level_set.data.levels.is_empty() {
        return Err(RequestError::NoLevels);
    }
    if level_set.meta.levels.len() != level_set.data.levels.len() {
        return Err(RequestError::InvalidLevel);
    }

    // TODO: check empty space

    for level in &level_set.data.levels {
        let duration = level.last_time();
        if ctl_core::types::time_to_seconds(duration).as_f32() < LEVEL_MIN_DURATION {
            return Err(RequestError::LevelTooShort);
        }
    }
    Ok(())
}
