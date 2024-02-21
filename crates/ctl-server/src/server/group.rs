use super::*;

pub fn route(router: Router) -> Router {
    router
        .route("/group/:group_id", get(group_get))
        .route("/group/create", post(group_create))
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
