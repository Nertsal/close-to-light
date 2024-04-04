use crate::database::types::MusicRow;

use super::*;

use ctl_core::{
    prelude::r32,
    types::{ArtistInfo, MusicUpdate, NewMusic},
};

const MUSIC_SIZE_LIMIT: usize = 5 * 1024 * 1024; // 5 MB

pub fn route(router: Router) -> Router {
    router
        .route("/music", get(music_list))
        .route("/music/:music_id", get(music_get).patch(music_update))
        .route(
            "/music/:music_id/authors",
            post(add_author).delete(remove_author),
        )
        .route("/music/:music_id/download", get(download))
        .route("/music/create", post(music_create))
}

/// Check if music exists.
pub(super) async fn music_exists(app: &App, music_id: Id) -> Result<()> {
    let check = sqlx::query("SELECT null FROM musics WHERE music_id = ?")
        .bind(music_id)
        .fetch_optional(&app.database)
        .await?;
    if check.is_none() {
        return Err(RequestError::NoSuchMusic(music_id));
    }
    Ok(())
}

// TODO: filter, sort, limit, pages
pub(super) async fn music_list(State(app): State<Arc<App>>) -> Result<Json<Vec<MusicInfo>>> {
    let rows: Vec<MusicRow> = sqlx::query_as("SELECT * FROM musics WHERE public = 1")
        .fetch_all(&app.database)
        .await?;

    let authors: Vec<(Id, ArtistInfo)> = sqlx::query(
        "
SELECT music_id, artists.artist_id, name, user_id
FROM music_authors
JOIN artists ON music_authors.artist_id = artists.artist_id
        ",
    )
    .try_map(|row: DBRow| {
        Ok((
            row.try_get("music_id")?,
            ArtistInfo {
                id: row.try_get("artist_id")?,
                name: row.try_get("name")?,
                user: row.try_get("user_id")?,
            },
        ))
    })
    .fetch_all(&app.database)
    .await?;

    let music = rows
        .into_iter()
        .map(|music| {
            let authors = authors
                .iter()
                .filter(|(music_id, _)| *music_id == music.music_id)
                .map(|(_, info)| info.clone())
                .collect();
            MusicInfo {
                id: music.music_id,
                public: music.public,
                original: music.original,
                name: music.name,
                bpm: r32(music.bpm),
                authors,
            }
        })
        .collect();

    Ok(Json(music))
}

pub(super) async fn music_get(
    State(app): State<Arc<App>>,
    Path(music_id): Path<Id>,
) -> Result<Json<MusicInfo>> {
    let row: Option<MusicRow> = sqlx::query_as("SELECT * FROM musics WHERE music_id = ?")
        .bind(music_id)
        .fetch_optional(&app.database)
        .await?;
    let Some(music) = row else {
        return Err(RequestError::NoSuchMusic(music_id));
    };

    let authors: Vec<ArtistInfo> = sqlx::query(
        "
SELECT artists.artist_id, name, user_id
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
            user: row.try_get("user_id")?,
        })
    })
    .fetch_all(&app.database)
    .await?;

    let music = MusicInfo {
        id: music_id,
        public: music.public,
        original: music.original,
        name: music.name,
        bpm: r32(music.bpm),
        authors,
    };
    Ok(Json(music))
}

async fn music_create(
    session: AuthSession,
    State(app): State<Arc<App>>,
    Query(mut music): Query<NewMusic>,
    body: Body,
) -> Result<Json<Id>> {
    check_auth(&session, &app, AuthorityLevel::Admin).await?;

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
    debug!("New music committed to the database");

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

    Ok(Json(music_id))
}

async fn music_update(
    session: AuthSession,
    State(app): State<Arc<App>>,
    Path(music_id): Path<Id>,
    Json(update): Json<MusicUpdate>,
) -> Result<()> {
    check_auth(&session, &app, AuthorityLevel::Admin).await?;

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
    session: AuthSession,
    State(app): State<Arc<App>>,
    Path(music_id): Path<Id>,
    Query(artist): Query<IdQuery>,
) -> Result<()> {
    check_auth(&session, &app, AuthorityLevel::Admin).await?;

    let artist_id = artist.id;

    // Check that artist exists
    let check = sqlx::query("SELECT null FROM artists WHERE artist_id = ?")
        .bind(artist_id)
        .fetch_optional(&app.database)
        .await?;
    if check.is_none() {
        return Err(RequestError::NoSuchArtist(artist_id));
    }

    music_exists(&app, music_id).await?;

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
    session: AuthSession,
    State(app): State<Arc<App>>,
    Path(music_id): Path<Id>,
    Query(artist): Query<IdQuery>,
) -> Result<()> {
    check_auth(&session, &app, AuthorityLevel::Admin).await?;

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
    let id: Option<Id> = sqlx::query("SELECT music_id FROM musics WHERE music_id = ?")
        .bind(music_id)
        .try_map(|row: DBRow| row.try_get("music_id"))
        .fetch_optional(&app.database)
        .await?;

    let Some(music_id) = id else {
        return Err(RequestError::NoSuchMusic(music_id));
    };

    // Check path
    let dir_path = app.config.groups_path.join("music");
    std::fs::create_dir_all(&dir_path)?;
    let path = dir_path.join(music_id.to_string());

    send_file(path, content_mp3()).await
}
