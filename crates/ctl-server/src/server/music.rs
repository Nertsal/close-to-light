use super::*;

use ctl_core::types::NewMusic;

const MUSIC_SIZE_LIMIT: usize = 5 * 1024 * 1024; // 5 MB

pub fn route(router: Router) -> Router {
    router
        .route(
            "/music/:music_id/authors",
            post(add_author).delete(remove_author),
        )
        .route("/music/:music_id/download", get(download))
        .route("/music/create", post(music_create))
}

async fn music_create(
    State(app): State<Arc<App>>,
    Query(mut music): Query<NewMusic>,
    api_key: ApiKey,
    body: Body,
) -> Result<Json<Id>> {
    let auth = get_auth(Some(api_key), &app.database).await?;
    check_auth(auth, AuthorityLevel::Admin)?;

    music.name = validate_name(music.name)?;

    // TODO: check that file is mp3 format
    // Download the file
    let data = axum::body::to_bytes(body, MUSIC_SIZE_LIMIT)
        .await
        .expect("not bytes idk");

    // Commit to database
    let music_id: Id = sqlx::query(
        "INSERT INTO musics (name, public, original, bpm) VALUES (?, ?, ?, ?) RETURNING music_id",
    )
    .bind(music.name)
    .bind(false)
    .bind(music.original)
    .bind(music.bpm)
    .try_map(|row: DBRow| row.try_get("music_id"))
    .fetch_one(&app.database)
    .await?;

    // Check path
    let dir_path = app.config.groups_path.join("music");
    std::fs::create_dir_all(&dir_path)?;
    let path = dir_path.join(music_id.to_string());
    debug!("Saving music file at {:?}", path);

    // let Some(music_path) = path.to_str() else {
    //     error!("Music path is not valid unicode");
    //     return Err(RequestError::Internal);
    // };

    if path.exists() {
        error!("Duplicate music ID generated: {}", music_id);
        return Err(RequestError::Internal);
    }

    // Write file to path
    std::fs::write(&path, data)?;
    debug!("Saved music file successfully");

    debug!("New music committed to the database");

    Ok(Json(music_id))
}

async fn add_author(
    State(app): State<Arc<App>>,
    Path(music_id): Path<Id>,
    Query(player): Query<PlayerIdQuery>,
    api_key: ApiKey,
) -> Result<()> {
    let auth = get_auth(Some(api_key), &app.database).await?;
    check_auth(auth, AuthorityLevel::Admin)?;

    let player_id = player.player_id;

    // Check that music exists
    let check = sqlx::query("SELECT null FROM musics WHERE music_id = ?")
        .bind(music_id)
        .fetch_optional(&app.database)
        .await?;
    if check.is_none() {
        return Err(RequestError::NoSuchMusic(music_id));
    }

    // Check that player is not already an author
    let check = sqlx::query("SELECT null FROM music_authors WHERE music_id = ? AND player_id = ?")
        .bind(music_id)
        .bind(player_id)
        .fetch_optional(&app.database)
        .await?;
    if check.is_some() {
        // Already in the database
        return Ok(());
    }

    // Add player as author
    sqlx::query("INSERT INTO music_authors (music_id, player_id) VALUES (?, ?)")
        .bind(music_id)
        .bind(player_id)
        .execute(&app.database)
        .await?;

    Ok(())
}

async fn remove_author(
    State(app): State<Arc<App>>,
    Path(music_id): Path<Id>,
    Query(player): Query<PlayerIdQuery>,
    api_key: ApiKey,
) -> Result<()> {
    let auth = get_auth(Some(api_key), &app.database).await?;
    check_auth(auth, AuthorityLevel::Admin)?;

    let player_id = player.player_id;

    sqlx::query("DELETE FROM music_authors WHERE music_id = ? AND player_id = ?")
        .bind(music_id)
        .bind(player_id)
        .execute(&app.database)
        .await?;

    Ok(())
}

async fn download(
    State(app): State<Arc<App>>,
    Path(music_id): Path<Id>,
) -> Result<impl IntoResponse> {
    let music_row = sqlx::query("SELECT file_path FROM musics WHERE music_id = ?")
        .bind(music_id)
        .fetch_optional(&app.database)
        .await?;

    let Some(row) = music_row else {
        return Err(RequestError::NoSuchMusic(music_id));
    };

    let file_path: String = row.try_get("file_path")?;
    let file_path = PathBuf::from(file_path);

    send_file(file_path, content_mp3()).await
}
