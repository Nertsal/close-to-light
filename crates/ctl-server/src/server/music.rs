use crate::database::types::MusicRow;

use super::*;

use ctl_core::types::{MusicUpdate, MusicianInfo, NewMusic};
use sqlx::FromRow;

const MUSIC_SIZE_LIMIT: usize = 10 * 1024 * 1024; // 10 MB
const MAX_MUSIC_UPLOADS_PER_USER: usize = 5;

pub fn route(router: Router) -> Router {
    router
        .route("/music", get(music_list))
        .route("/music/:music_id", get(music_get).patch(music_update))
        .route(
            "/music/:music_id/authors",
            post(add_author).delete(remove_author),
        )
        .route("/music/:music_id/download", get(download_by_music_id))
        .route("/music/download", get(download_by_query))
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

#[derive(Deserialize)]
pub(super) struct GetMusicQuery {
    pub level_set_id: Option<Id>,
}

// TODO: filter, sort, limit, pages
pub(super) async fn music_list(
    State(app): State<Arc<App>>,
    Query(query): Query<GetMusicQuery>,
) -> Result<Json<Vec<MusicInfo>>> {
    if let Some(level_set_id) = query.level_set_id {
        // Get music info for a specific group
        let music_id = get_music_id_for_level(&app, level_set_id).await?;
        let music_info = music_get(State(app), Path(music_id)).await?.0;
        return Ok(Json(vec![music_info]));
    }

    let rows: Vec<MusicRow> = sqlx::query_as("SELECT * FROM musics")
        .fetch_all(&app.database)
        .await?;

    let authors: Vec<(Id, MusicianRow)> = sqlx::query(
        "
SELECT *
FROM music_authors
JOIN musicians ON music_authors.musician_id = musicians.musician_id
        ",
    )
    .try_map(|row: DBRow| Ok((row.try_get("music_id")?, MusicianRow::from_row(&row)?)))
    .fetch_all(&app.database)
    .await?;

    let music = rows
        .into_iter()
        .map(|music| {
            let authors = authors
                .iter()
                .filter(|(music_id, _)| *music_id == music.music_id)
                .map(|(_, info)| info.clone().into())
                .collect();
            MusicInfo {
                id: music.music_id,
                original: music.original,
                name: music.name.into(),
                romanized: music.romanized_name.into(),
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

    let authors: Vec<MusicianRow> = sqlx::query_as(
        "
SELECT *
FROM music_authors
JOIN musicians ON music_authors.musician_id = musicians.musician_id
WHERE music_id = ?
        ",
    )
    .bind(music_id)
    .fetch_all(&app.database)
    .await?;
    let authors: Vec<MusicianInfo> = authors.into_iter().map(Into::into).collect();

    let music = MusicInfo {
        id: music_id,
        original: music.original,
        name: music.name.into(),
        romanized: music.romanized_name.into(),
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
    let user = check_user(&session).await?;

    music.name = validate_name(&music.name)?;
    music.romanized_name = validate_romanized_name(&music.romanized_name)?;

    // TODO: check that file is mp3 format
    // Download the file
    let data = axum::body::to_bytes(body, MUSIC_SIZE_LIMIT)
        .await
        .expect("not bytes idk");

    // Check user's uploaded music count
    let music_counts: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM musics WHERE uploaded_by_user = ?")
            .bind(user.user_id)
            .fetch_one(&app.database)
            .await?;
    if music_counts >= MAX_MUSIC_UPLOADS_PER_USER as i64 {
        return Err(RequestError::TooManyMusic);
    }

    // Commit to database
    let music_id: Id = sqlx::query_scalar(
        "INSERT INTO musics (name, romanized_name, original, featured, uploaded_by_user)
         VALUES (?, ?, ?, ?, ?) RETURNING music_id",
    )
    .bind(music.name)
    .bind(music.romanized_name)
    .bind(false)
    .bind(false)
    .bind(user.user_id)
    .fetch_one(&app.database)
    .await?;
    debug!("New music committed to the database");

    // Check path
    let dir_path = app.config.level_sets_path.join("music");
    std::fs::create_dir_all(&dir_path)?;
    let path = dir_path.join(format!("{}.mp3", music_id));
    debug!("Saving music file at {:?}", path);

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
    original = COALESCE(?, original),
    featured = COALESCE(?, featured),
WHERE music_id = ?",
    )
    .bind(&update.name)
    .bind(update.original)
    .bind(update.featured)
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
    Query(musician): Query<IdQuery>,
) -> Result<()> {
    check_auth(&session, &app, AuthorityLevel::Admin).await?;

    let musician_id = musician.id;

    // Check that artist exists
    let check = sqlx::query("SELECT null FROM musicians WHERE musician_id = ?")
        .bind(musician_id)
        .fetch_optional(&app.database)
        .await?;
    if check.is_none() {
        return Err(RequestError::NoSuchMusician(musician_id));
    }

    music_exists(&app, music_id).await?;

    // Check that musician is not already an author
    let check =
        sqlx::query("SELECT null FROM music_authors WHERE music_id = ? AND musician_id = ?")
            .bind(music_id)
            .bind(musician_id)
            .fetch_optional(&app.database)
            .await?;
    if check.is_some() {
        // Already in the database
        return Ok(());
    }

    // Add musician as author
    sqlx::query("INSERT INTO music_authors (music_id, musician_id) VALUES (?, ?)")
        .bind(music_id)
        .bind(musician_id)
        .execute(&app.database)
        .await?;

    Ok(())
}

async fn remove_author(
    session: AuthSession,
    State(app): State<Arc<App>>,
    Path(music_id): Path<Id>,
    Query(musician): Query<IdQuery>,
) -> Result<()> {
    check_auth(&session, &app, AuthorityLevel::Admin).await?;

    let musician_id = musician.id;

    sqlx::query("DELETE FROM music_authors WHERE music_id = ? AND musician_id = ?")
        .bind(music_id)
        .bind(musician_id)
        .execute(&app.database)
        .await?;

    Ok(())
}

async fn download_by_music_id(
    State(app): State<Arc<App>>,
    Path(music_id): Path<Id>,
) -> Result<impl IntoResponse> {
    let id: Option<Id> = sqlx::query_scalar("SELECT music_id FROM musics WHERE music_id = ?")
        .bind(music_id)
        .fetch_optional(&app.database)
        .await?;

    let Some(music_id) = id else {
        return Err(RequestError::NoSuchMusic(music_id));
    };

    // Check path
    let dir_path = app.config.level_sets_path.join("music");
    let path = dir_path.join(music_id.to_string());

    send_file(path, content_mp3()).await
}

#[derive(Deserialize)]
struct DownloadQuery {
    music_id: Option<Id>,
    level_set_id: Option<Id>,
}

async fn download_by_query(
    State(app): State<Arc<App>>,
    Query(query): Query<DownloadQuery>,
) -> Result<impl IntoResponse> {
    let music_id = if let Some(level_set_id) = query.level_set_id {
        // Get the music for the level_set
        let music_id = get_music_id_for_level(&app, level_set_id).await?;

        if let Some(query_music) = query.music_id {
            if music_id != query_music {
                return Err(RequestError::InvalidLevel); // TODO: better error
            }
        }

        music_id
    } else if let Some(music_id) = query.music_id {
        music_id
    } else {
        return Err(RequestError::InvalidRequest);
    };

    download_by_music_id(State(app), Path(music_id)).await
}

async fn get_music_id_for_level(app: &App, level_set_id: Id) -> Result<Id> {
    let music_id: Option<Id> =
        sqlx::query_scalar("SELECT music_id FROM level_sets WHERE level_set_id = ?")
            .bind(level_set_id)
            .fetch_optional(&app.database)
            .await?;
    let Some(music_id) = music_id else {
        return Err(RequestError::NoSuchLevelSet(level_set_id));
    };
    Ok(music_id)
}
