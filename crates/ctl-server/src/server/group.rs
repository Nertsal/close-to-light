use super::*;

pub fn route(router: Router) -> Router {
    router
        .route("/group/:group_id", get(group_get).delete(group_delete))
        .route("/group/create", post(group_create))
}

async fn group_get(
    State(app): State<Arc<App>>,
    Path(group_id): Path<Uuid>,
) -> Result<Json<GroupInfo>> {
    let music_id: Option<Uuid> = sqlx::query("SELECT music_id WHERE group_id = ?")
        .bind(group_id)
        .try_map(|row: DBRow| row.try_get("music_id"))
        .fetch_optional(&app.database)
        .await?;
    let Some(music_id) = music_id else {
        return Err(RequestError::NoSuchGroup(group_id));
    };

    let music_name: Option<String> = sqlx::query("SELECT name FROM musics WHERE music_id = ?")
        .bind(music_id)
        .try_map(|row: DBRow| row.try_get("name"))
        .fetch_optional(&app.database)
        .await?;
    let Some(music_name) = music_name else {
        return Err(RequestError::NoSuchMusic(music_id));
    };

    let music_authors: Vec<PlayerInfo> = sqlx::query(
        "
SELECT players.player_id, name
FROM music_authors
WHERE music_id = ?
JOIN players ON music_authors.player_id = players.player_id
        ",
    )
    .bind(music_id)
    .try_map(|row: DBRow| {
        Ok(PlayerInfo {
            id: row.try_get("player_id")?,
            name: row.try_get("name")?,
        })
    })
    .fetch_all(&app.database)
    .await?;

    let music = MusicInfo {
        id: music_id,
        name: music_name,
        authors: music_authors,
    };

    let level_rows: Vec<(Uuid, String)> =
        sqlx::query("SELECT level_id, name FROM levels WHERE group_id = ?")
            .bind(group_id)
            .try_map(|row: DBRow| Ok((row.try_get("level_id")?, row.try_get("name")?)))
            .fetch_all(&app.database)
            .await?;

    let mut levels = Vec::new();
    for (level_id, level_name) in level_rows {
        let authors: Vec<PlayerInfo> = sqlx::query(
            "
SELECT players.player_id, name
FROM level_authors
WHERE level_id = ?
JOIN players ON level_authors.player_id = players.player_id
        ",
        )
        .bind(level_id)
        .try_map(|row: DBRow| {
            Ok(PlayerInfo {
                id: row.try_get("player_id")?,
                name: row.try_get("name")?,
            })
        })
        .fetch_all(&app.database)
        .await?;

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

async fn group_delete() {}

async fn group_create() {}
