use super::*;

use ctl_core::{
    prelude::r32,
    types::{ArtistInfo, MusicUpdate, NewMusic},
};

const MUSIC_SIZE_LIMIT: usize = 5 * 1024 * 1024; // 5 MB

pub fn route(router: Router) -> Router {
    router
        .route("/music/:music_id", get(music_get).patch(music_update))
        .route(
            "/music/:music_id/authors",
            post(add_author).delete(remove_author),
        )
        .route("/music/:music_id/download", get(download))
        .route("/music/create", post(music_create))
}

pub(super) async fn music_get(
    State(app): State<Arc<App>>,
    Path(music_id): Path<Id>,
) -> Result<Json<MusicInfo>> {
    let row: Option<(String, bool, bool, f32)> =
        sqlx::query("SELECT name, public, original, bpm FROM musics WHERE music_id = ?")
            .bind(music_id)
            .try_map(|row: DBRow| {
                Ok((
                    row.try_get("name")?,
                    row.try_get("public")?,
                    row.try_get("original")?,
                    row.try_get("bpm")?,
                ))
            })
            .fetch_optional(&app.database)
            .await?;
    let Some((music_name, public, original, bpm)) = row else {
        return Err(RequestError::NoSuchMusic(music_id));
    };

    let authors: Vec<ArtistInfo> = sqlx::query(
        "
SELECT artists.artist_id, name
FROM music_authors
JOIN artists ON music_authors.artist_id = artists.artist_id
WHERE music_id = ?
        ",
    )
    .bind(music_id)
    .try_map(|row: DBRow| {
        Ok(ArtistInfo {
            id: row.try_get("artist_id")?,
            name: row.try_get("name")?,
        })
    })
    .fetch_all(&app.database)
    .await?;

    let music = MusicInfo {
        id: music_id,
        public,
        original,
        name: music_name,
        bpm: r32(bpm),
        authors,
    };
    Ok(Json(music))
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

async fn music_update(
    State(app): State<Arc<App>>,
    Path(music_id): Path<Id>,
    api_key: ApiKey,
    Json(update): Json<MusicUpdate>,
) -> Result<()> {
    let auth = get_auth(Some(api_key), &app.database).await?;
    check_auth(auth, AuthorityLevel::Admin)?;

    let result = sqlx::query(
        "
UPDATE musics
SET name = COALESCE(?, name),
    public = COALESCE(?, public),
    original = COALESCE(?, original),
    bpm = COALESCE(?, bpm)
WHERE music_id = ?",
    )
    .bind(&update.name)
    .bind(update.public)
    .bind(update.original)
    .bind(update.bpm)
    .bind(music_id)
    .execute(&app.database)
    .await?;

    if result.rows_affected() == 0 {
        return Err(RequestError::NoSuchMusic(music_id));
    }

    Ok(())
}

async fn add_author(
    State(app): State<Arc<App>>,
    Path(music_id): Path<Id>,
    Query(artist): Query<IdQuery>,
    api_key: ApiKey,
) -> Result<()> {
    let auth = get_auth(Some(api_key), &app.database).await?;
    check_auth(auth, AuthorityLevel::Admin)?;

    let artist_id = artist.id;

    // Check that artist exists
    let check = sqlx::query("SELECT null FROM artists WHERE artist_id = ?")
        .bind(artist_id)
        .fetch_optional(&app.database)
        .await?;
    if check.is_none() {
        return Err(RequestError::NoSuchArtist(artist_id));
    }

    // Check that music exists
    let check = sqlx::query("SELECT null FROM musics WHERE music_id = ?")
        .bind(music_id)
        .fetch_optional(&app.database)
        .await?;
    if check.is_none() {
        return Err(RequestError::NoSuchMusic(music_id));
    }

    // Check that artist is not already an author
    let check = sqlx::query("SELECT null FROM music_authors WHERE music_id = ? AND artist_id = ?")
        .bind(music_id)
        .bind(artist_id)
        .fetch_optional(&app.database)
        .await?;
    if check.is_some() {
        // Already in the database
        return Ok(());
    }

    // Add artist as author
    sqlx::query("INSERT INTO music_authors (music_id, artist_id) VALUES (?, ?)")
        .bind(music_id)
        .bind(artist_id)
        .execute(&app.database)
        .await?;

    Ok(())
}

async fn remove_author(
    State(app): State<Arc<App>>,
    Path(music_id): Path<Id>,
    Query(artist): Query<IdQuery>,
    api_key: ApiKey,
) -> Result<()> {
    let auth = get_auth(Some(api_key), &app.database).await?;
    check_auth(auth, AuthorityLevel::Admin)?;

    let artist_id = artist.id;

    sqlx::query("DELETE FROM music_authors WHERE music_id = ? AND artist_id = ?")
        .bind(music_id)
        .bind(artist_id)
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
