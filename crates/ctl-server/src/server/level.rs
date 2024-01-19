use super::*;

pub fn route(router: Router) -> Router {
    router
        .route("/level/:level_id", delete(level_delete))
        .route("/level/:level_id/download", get(download))
        .route("/level/create", post(level_create))
}

async fn level_delete() {
    // TODO
}

async fn level_create() {
    // TODO
}

async fn download(
    State(database): State<Arc<DatabasePool>>,
    Path(level_id): Path<Uuid>,
) -> Result<impl IntoResponse> {
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
