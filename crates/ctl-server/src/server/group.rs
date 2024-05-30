use super::*;

use crate::database::types::LevelRow;

use axum::{body::Bytes, extract::DefaultBodyLimit};
use ctl_core::types::{GroupsQuery, LevelFull, LevelSet};

const GROUP_SIZE_LIMIT: usize = 1024 * 1024; // 1 MB
const GROUPS_PER_USER: usize = 5;
const GROUPS_PER_USER_PER_SONG: usize = 1;

pub fn route(router: Router) -> Router {
    router
        .route("/groups", get(group_list))
        .route("/group/:group_id", get(group_get))
        .route("/group/:group_id/download", get(download))
        .route("/group/create", post(group_create))
        .layer(DefaultBodyLimit::max(GROUP_SIZE_LIMIT))
}

// TODO: filter, sort, limit, pages
async fn group_list(
    State(app): State<Arc<App>>,
    Query(query): Query<GroupsQuery>,
) -> Result<Json<Vec<GroupInfo>>> {
    let music = super::music::music_list(State(app.clone())).await?.0;

    #[derive(sqlx::FromRow)]
    struct LevelGroupRow {
        #[sqlx(flatten)]
        group: GroupRow,
        #[sqlx(flatten)]
        level: LevelRow,
    }

    let query = if query.recommended {
        "SELECT * FROM levels JOIN (
            SELECT * FROM groups_recommended JOIN groups ON groups_recommended.group_id = groups.group_id
        ) AS groups ON levels.group_id = groups.group_id"
    } else {
        "SELECT * FROM levels JOIN groups ON levels.group_id = groups.group_id"
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

    let mut groups = Vec::<GroupInfo>::new();
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
            .position(|g| g.id == level_row.level.group_id)
            .unwrap_or_else(|| {
                groups.push(GroupInfo {
                    id: level_row.level.group_id,
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

async fn group_get(
    State(app): State<Arc<App>>,
    Path(group_id): Path<Id>,
) -> Result<Json<GroupInfo>> {
    let group_row: Option<GroupRow> = sqlx::query_as("SELECT * FROM groups WHERE group_id = ?")
        .bind(group_id)
        .fetch_optional(&app.database)
        .await?;
    let Some(group_row) = group_row else {
        return Err(RequestError::NoSuchGroup(group_id));
    };

    let music = music::music_get(State(app.clone()), Path(group_row.music_id))
        .await?
        .0;

    let level_rows: Vec<LevelRow> =
        sqlx::query_as("SELECT * FROM levels WHERE group_id = ? ORDER BY ord")
            .bind(group_id)
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

    Ok(Json(GroupInfo {
        id: group_id,
        music,
        owner,
        levels,
        hash: group_row.hash,
    }))
}

async fn group_create(
    session: AuthSession,
    State(app): State<Arc<App>>,
    data: Bytes,
) -> Result<Json<Id>> {
    let user = check_user(&session).await?;

    // NOTE: Not parsing into Rc, because we cant hold it across an await point
    // also we want to mutate it
    let parsed_group: LevelSet<LevelFull> =
        bincode::deserialize(&data).map_err(|_| RequestError::InvalidLevel)?;

    music::music_exists(&app, parsed_group.music).await?;

    let group_id = if parsed_group.id != 0 {
        let id = parsed_group.id;
        update_group(&app, user, parsed_group).await?;
        id
    } else {
        new_group(&app, user, parsed_group).await?
    };

    Ok(Json(group_id))
}

async fn update_group(app: &App, user: &User, mut parsed_group: LevelSet<LevelFull>) -> Result<()> {
    let group_id = parsed_group.id;
    let group: Option<GroupRow> = sqlx::query_as("SELECT * FROM groups WHERE group_id = ?")
        .bind(group_id)
        .fetch_optional(&app.database)
        .await?;
    let group = group.ok_or(RequestError::NoSuchGroup(group_id))?;

    // Check if the player has rights to change the group
    if user.user_id != group.owner_id {
        return Err(RequestError::Forbidden);
    }

    // Verify owner
    parsed_group.owner = UserInfo {
        id: user.user_id,
        name: user.username.clone().into(),
    };

    // Update levels
    // TODO: remove removed levels
    for (order, level) in parsed_group.levels.iter_mut().enumerate() {
        let order = order as i64;
        level.meta.hash = level.data.calculate_hash();
        if level.meta.id == 0 {
            // Create
            level.meta.id = sqlx::query(
                "INSERT INTO levels (hash, group_id, name, ord) VALUES (?, ?, ?, ?) RETURNING level_id",
            )
            .bind(&level.meta.hash)
            .bind(group_id)
            .bind(level.meta.name.as_ref())
            .bind(order)
            .try_map(|row: DBRow| row.try_get("level_id"))
            .fetch_one(&app.database)
            .await?;
        } else {
            // Update
            sqlx::query(
                "UPDATE levels SET hash = ?, name = ?, ord = ? WHERE level_id = ? AND group_id = ?",
            )
            .bind(&level.meta.hash)
            .bind(level.meta.name.as_ref())
            .bind(level.meta.id)
            .bind(group_id)
            .bind(order)
            .execute(&app.database)
            .await?;
        }
    }

    // Disallow further mutation to make sure the hash is valid
    let parsed_group = parsed_group;
    let hash = parsed_group.calculate_hash();

    // Update group
    sqlx::query("UPDATE groups SET hash = ? WHERE group_id = ?")
        .bind(&hash)
        .bind(group_id)
        .execute(&app.database)
        .await?;

    // Check path
    let dir_path = app.config.groups_path.join("levels");
    std::fs::create_dir_all(&dir_path)?;
    let path = dir_path.join(group_id.to_string());
    debug!("Saving group file at {:?}", path);

    if !path.exists() {
        error!(
            "Updating a group but it is not present in the file system: {}",
            group_id
        );
    }

    // Write to file
    let data = bincode::serialize(&parsed_group).map_err(|_| RequestError::Internal)?;
    std::fs::write(path, data)?;
    debug!("Saved group file successfully");

    Ok(())
}

async fn new_group(app: &App, user: &User, mut parsed_group: LevelSet<LevelFull>) -> Result<Id> {
    // Check if the user already has groups
    let user_groups: Vec<GroupRow> = sqlx::query_as("SELECT * FROM groups WHERE owner_id = ?")
        .bind(user.user_id)
        .fetch_all(&app.database)
        .await?;
    if user_groups.len() >= GROUPS_PER_USER {
        return Err(RequestError::TooManyGroups);
    }
    if user_groups
        .iter()
        .filter(|group| group.music_id == parsed_group.music)
        .count()
        >= GROUPS_PER_USER_PER_SONG
    {
        return Err(RequestError::TooManyGroupsForSong);
    }

    // Check if such a level already exists
    for level in &mut parsed_group.levels {
        level.meta.hash = level.data.calculate_hash();
        let conflict = sqlx::query("SELECT null FROM levels WHERE hash = ?")
            .bind(&level.meta.hash)
            .fetch_optional(&app.database)
            .await?;
        if conflict.is_some() {
            return Err(RequestError::LevelAlreadyExists);
        }
    }

    // Verify owner
    parsed_group.owner = UserInfo {
        id: user.user_id,
        name: user.username.clone().into(),
    };

    // Create group
    let group_id: Id = sqlx::query(
        "INSERT INTO groups (music_id, owner_id, hash) VALUES (?, ?, ?) RETURNING group_id",
    )
    .bind(parsed_group.music)
    .bind(user.user_id)
    .bind("")
    .try_map(|row: DBRow| row.try_get("group_id"))
    .fetch_one(&app.database)
    .await?;
    parsed_group.id = group_id;

    // Create levels
    for (order, level) in parsed_group.levels.iter_mut().enumerate() {
        let order = order as i64;

        // Check if such a level already exists
        let conflict = sqlx::query("SELECT null FROM levels WHERE hash = ?")
            .bind(&level.meta.hash)
            .fetch_optional(&app.database)
            .await?;
        if conflict.is_some() {
            return Err(RequestError::LevelAlreadyExists);
        }

        level.meta.id = sqlx::query(
            "INSERT INTO levels (hash, group_id, name, ord) VALUES (?, ?, ?, ?) RETURNING level_id",
        )
        .bind(&level.meta.hash)
        .bind(group_id)
        .bind(level.meta.name.as_ref())
        .bind(order)
        .try_map(|row: DBRow| row.try_get("level_id"))
        .fetch_one(&app.database)
        .await?;

        level.meta.authors = vec![UserInfo {
            id: user.user_id,
            name: user.username.clone().into(),
        }];
        sqlx::query("INSERT INTO level_authors (level_id, user_id) VALUES (?, ?)")
            .bind(level.meta.id)
            .bind(user.user_id)
            .execute(&app.database)
            .await?;
    }

    // Disallow further mutation to make sure the hash is valid
    let parsed_group = parsed_group;
    let hash = parsed_group.calculate_hash();

    // Update hash
    sqlx::query("UPDATE groups SET hash = ? WHERE group_id = ?")
        .bind(&hash)
        .bind(group_id)
        .execute(&app.database)
        .await?;

    // Check path
    let dir_path = app.config.groups_path.join("levels");
    std::fs::create_dir_all(&dir_path)?;
    let path = dir_path.join(group_id.to_string());
    debug!("Saving group file at {:?}", path);

    if path.exists() {
        error!("Duplicate group ID generated: {}", group_id);
        return Err(RequestError::Internal);
    }

    // Write to file
    let data = bincode::serialize(&parsed_group).map_err(|_| RequestError::Internal)?;
    std::fs::write(path, data)?;
    debug!("Saved group file successfully");

    Ok(group_id)
}

async fn download(
    State(app): State<Arc<App>>,
    Path(group_id): Path<Id>,
) -> Result<impl IntoResponse> {
    let level_row = sqlx::query("SELECT null FROM groups WHERE group_id = ?")
        .bind(group_id)
        .fetch_optional(&app.database)
        .await?;

    if level_row.is_none() {
        return Err(RequestError::NoSuchGroup(group_id));
    }

    let file_path = app
        .config
        .groups_path
        .join("levels")
        .join(group_id.to_string());
    send_file(file_path, content_level()).await
}
