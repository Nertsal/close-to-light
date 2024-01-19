use super::*;

pub fn route(router: Router) -> Router {
    router
        // .route(
        //     "/music/:music_id/authors",
        //     post(add_author).delete(remove_author),
        // )
        .route("/music/:music_id/download", get(download))
        .route("/music/create", post(music_create))
}

#[derive(Deserialize)]
struct NewMusic {
    music_name: String,
}

async fn music_create(
    State(database): State<Arc<DatabasePool>>,
    Query(music): Query<NewMusic>,
    api_key: ApiKey,
    multipart: Multipart,
) -> Result<Json<Uuid>> {
    let auth = get_auth(Some(api_key), &database).await?;
    check_auth(auth, AuthorityLevel::Admin)?;

    let music_name = validate_name(music.music_name)?;

    // TODO: check that file is mp3 format
    // Download the file
    let data = receive_file(multipart).await?;
    let uuid = Uuid::new_v4();

    // Check path
    let base_path = PathBuf::from(crate::DEFAULT_GROUPS);
    let dir_path = base_path.join("music");
    std::fs::create_dir_all(&dir_path)?;
    let path = dir_path.join(uuid.hyphenated().to_string());
    debug!("Saving music file at {:?}", path);

    let Some(music_path) = path.to_str() else {
        error!("Music path is not valid unicode");
        return Err(RequestError::Internal);
    };

    if path.exists() {
        error!("Duplicate music UUID generated: {}", uuid);
        return Err(RequestError::Internal);
    }

    // Write file to path
    std::fs::write(&path, data)?;
    debug!("Saved music file successfully");

    // Commit to database
    sqlx::query("INSERT INTO musics (music_id, name, file_path) VALUES (?, ?, ?)")
        .bind(uuid)
        .bind(music_name)
        .bind(music_path)
        .execute(&*database)
        .await?;

    debug!("New music committed to the database");

    Ok(Json(uuid))
}

async fn download(
    State(database): State<Arc<DatabasePool>>,
    Path(music_id): Path<Uuid>,
) -> Result<impl IntoResponse> {
    let music_row = sqlx::query("SELECT file_path FROM musics WHERE music_id = ?")
        .bind(music_id)
        .fetch_optional(&*database)
        .await?;

    let Some(row) = music_row else {
        return Err(RequestError::NoSuchMusic(music_id));
    };

    let file_path: String = row.try_get("file_path")?;
    let file_path = PathBuf::from(file_path);

    send_file(file_path, content_mp3()).await
}
