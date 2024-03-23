use crate::database::types::LevelRow;

use super::*;

pub fn route(router: Router) -> Router {
    router
        .route("/groups", get(group_list))
        .route("/group/:group_id", get(group_get))
        .route("/group/create", post(group_create))
}

// TODO: filter, sort, limit, pages
async fn group_list(State(app): State<Arc<App>>) -> Result<Json<Vec<GroupInfo>>> {
    let music = super::music::music_list(State(app.clone())).await?.0;

    #[derive(sqlx::FromRow)]
    struct LevelGroupRow {
        music_id: Id,
        owner_id: Id,
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
            row.try_get("music_id")?,
            UserInfo {
                id: row.try_get("user_id")?,
                name: row.try_get("username")?,
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
            name: level_row.level.name,
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
    let music_id: Option<Id> = sqlx::query("SELECT music_id WHERE group_id = ?")
        .bind(group_id)
        .try_map(|row: DBRow| row.try_get("music_id"))
        .fetch_optional(&app.database)
        .await?;
    let Some(music_id) = music_id else {
        return Err(RequestError::NoSuchGroup(group_id));
    };

    let music = music::music_get(State(app.clone()), Path(music_id))
        .await?
        .0;

    let level_rows: Vec<(Id, String)> =
        sqlx::query("SELECT level_id, name FROM levels WHERE group_id = ?")
            .bind(group_id)
            .try_map(|row: DBRow| Ok((row.try_get("level_id")?, row.try_get("name")?)))
            .fetch_all(&app.database)
            .await?;

    let authors: Vec<(Id, UserInfo)> = sqlx::query(
        "
SELECT level_id, players.player_id, name
FROM level_authors
JOIN players ON level_authors.player_id = players.player_id
        ",
    )
    .try_map(|row: DBRow| {
        Ok((
            row.try_get("level_id")?,
            UserInfo {
                id: row.try_get("player_id")?,
                name: row.try_get("name")?,
            },
        ))
    })
    .fetch_all(&app.database)
    .await?;

    let mut levels = Vec::new();
    for (level_id, level_name) in level_rows {
        let authors = authors
            .iter()
            .filter(|(id, _)| *id == level_id)
            .map(|(_, player)| player.clone())
            .collect();
        levels.push(LevelInfo {
            id: level_id,
            name: level_name,
            authors,
        });
    }

    Ok(Json(GroupInfo {
        id: group_id,
        music,
        levels,
    }))
}

async fn group_create(
    session: AuthSession,
    State(app): State<Arc<App>>,
    Query(music): Query<IdQuery>,
) -> Result<Json<Id>> {
    check_auth(&session, &app, AuthorityLevel::User).await?;

    music::music_exists(&app, music.id).await?;

    let group_id: Id = todo!();
    // sqlx::query("INSERT INTO groups (music_id, owner_id) VALUES (?, ?) RETURNING group_id")
    //     .bind(music.id)
    //     .bind(player_id)
    //     .try_map(|row: DBRow| row.try_get("group_id"))
    //     .fetch_one(&app.database)
    //     .await?;

    Ok(Json(group_id))
}
