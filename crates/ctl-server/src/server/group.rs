use crate::database::types::LevelRow;

use super::*;

const GROUP_SIZE_LIMIT: usize = 5 * 1024 * 1024; // 5 MB

pub fn route(router: Router) -> Router {
    router
        .route("/groups", get(group_list))
        .route("/group/:group_id", get(group_get))
        .route("/group/:group_id/download", get(download))
        .route("/group/create", post(group_create))
}

// TODO: filter, sort, limit, pages
async fn group_list(State(app): State<Arc<App>>) -> Result<Json<Vec<GroupInfo>>> {
    let music = super::music::music_list(State(app.clone())).await?.0;

    #[derive(sqlx::FromRow)]
    struct LevelGroupRow {
        music_id: Id,
        // owner_id: Id,
        #[sqlx(flatten)]
        level: LevelRow,
    }

    let levels: Vec<LevelGroupRow> =
        sqlx::query_as("SELECT * FROM levels JOIN groups ON levels.group_id = groups.group_id")
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

    let mut groups = Vec::<GroupInfo>::new();
    for level_row in levels {
        let authors = authors
            .iter()
            .filter(|(level_id, _)| *level_id == level_row.level.level_id)
            .map(|(_, user)| user.clone())
            .collect();

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
                        .find(|music| music.id == level_row.music_id)
                        .cloned()
                        .unwrap_or_default(), // Default should never be reached TODO: warning or smth
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

    let level_rows: Vec<LevelRow> = sqlx::query_as("SELECT * FROM levels WHERE group_id = ?")
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
        levels,
        hash: group_row.hash,
    }))
}

// TODO update
async fn group_create(
    session: AuthSession,
    State(app): State<Arc<App>>,
    Query(music): Query<IdQuery>,
) -> Result<Json<Id>> {
    let user = check_user(&session).await?;

    music::music_exists(&app, music.id).await?;

    let group_id: Id =
        sqlx::query("INSERT INTO groups (music_id, owner_id) VALUES (?, ?) RETURNING group_id")
            .bind(music.id)
            .bind(user.user_id)
            .try_map(|row: DBRow| row.try_get("group_id"))
            .fetch_one(&app.database)
            .await?;

    Ok(Json(group_id))
}
// /// Create a new level or upload a new version of an existing one.
// async fn level_create(
//     session: AuthSession,
//     State(app): State<Arc<App>>,
//     Query(level): Query<NewLevel>,
//     body: Body,
// ) -> Result<Json<Id>> {
//     let user = check_user(&session).await?;

//     // Check that group exists
//     let group: Option<GroupRow> = sqlx::query_as("SELECT * FROM groups WHERE group_id = ?")
//         .bind(level.group)
//         .fetch_optional(&app.database)
//         .await?;
//     let Some(group) = group else {
//         return Err(RequestError::NoSuchGroup(level.group));
//     };

//     // Check if the player has rights to add levels to the group
//     if user.user_id != group.owner_id {
//         return Err(RequestError::Forbidden);
//     }

//     let data = axum::body::to_bytes(body, LEVEL_SIZE_LIMIT)
//         .await
//         .expect("not bytes idk");

//     // Calculate level hash
//     let hash = ctl_core::util::calculate_hash(&data);

//     // Check if such a level already exists
//     let conflict = sqlx::query("SELECT null FROM levels WHERE hash = ?")
//         .bind(&hash)
//         .fetch_optional(&app.database)
//         .await?;
//     if conflict.is_some() {
//         return Err(RequestError::LevelAlreadyExists);
//     }

//     // Validate level contents
//     let _parsed_level: Level =
//         bincode::deserialize(&data).map_err(|_| RequestError::InvalidLevel)?;
//     // TODO

//     let level_id = if let Some(level_id) = level.level_id {
//         let res = sqlx::query("UPDATE levels SET hash = ? WHERE level_id = ?")
//             .bind(&hash)
//             .bind(level_id)
//             .execute(&app.database)
//             .await?;
//         if res.rows_affected() == 0 {
//             return Err(RequestError::NoSuchLevel(level_id));
//         }

//         level_id
//     } else {
//         // Commit to database
//         let level_id: Id = sqlx::query(
//             "INSERT INTO levels (name, group_id, hash) VALUES (?, ?, ?) RETURNING level_id",
//         )
//         .bind(&level.name)
//         .bind(level.group)
//         .bind(&hash)
//         .try_map(|row: DBRow| row.try_get("level_id"))
//         .fetch_one(&app.database)
//         .await?;
//         debug!("New level committed to the database");

//         // Add user as author
//         sqlx::query("INSERT INTO level_authors (user_id, level_id) VALUES (?, ?)")
//             .bind(user.user_id)
//             .bind(level_id)
//             .execute(&app.database)
//             .await?;

//         level_id
//     };

//     // Check path
//     let dir_path = app.config.groups_path.join("levels");
//     std::fs::create_dir_all(&dir_path)?;
//     let path = dir_path.join(level_id.to_string());
//     debug!("Saving level file at {:?}", path);

//     // let Some(music_path) = path.to_str() else {
//     //     error!("Music path is not valid unicode");
//     //     return Err(RequestError::Internal);
//     // };

//     if level.level_id.is_none() && path.exists() {
//         error!("Duplicate level ID generated: {}", level_id);
//         return Err(RequestError::Internal);
//     }

//     // Write to file
//     std::fs::write(path, data)?;
//     debug!("Saved level file successfully");

//     Ok(Json(level_id))
// }

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
