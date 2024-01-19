use super::*;

pub fn route(router: Router) -> Router {
    router
        .route(
            "/level/:level_id",
            get(level_get).post(level_post).delete(level_delete),
        )
        .route("/level/:level_id/download", get(download))
        .route("/level/create", post(level_create))
}

async fn level_get() {}

async fn level_post() {}

async fn level_delete() {}

async fn level_create() {}

async fn download(
    State(database): State<Arc<DatabasePool>>,
    level_id: String,
) -> Result<impl IntoResponse> {
    let level_id =
        Uuid::try_parse(&level_id).map_err(|_| RequestError::InvalidLevelId(level_id))?;

    let level_row = sqlx::query("SELECT file_path FROM levels WHERE level_id = ?")
        .bind(level_id)
        .fetch_optional(&*database)
        .await?;

    let Some(row) = level_row else {
        return Err(RequestError::NoSuchLevel(level_id));
    };

    let file_path: String = row.try_get("file_path")?;
    let file_path = PathBuf::from(file_path);

    upload_file(file_path, content_level()).await
}
