use super::*;

/// Queries information about the board by name and returns its id
/// together with the authority level of the provided api key.
async fn check_board(
    Path(board_name): Path<String>,
    State(database): State<Arc<DatabasePool>>,
    api_key: Option<ApiKey>,
) -> Result<(Id, AuthorityLevel)> {
    let board_row = sqlx::query(
        "SELECT board_id, read_key, submit_key, admin_key FROM boards WHERE board_name = ?",
    )
    .bind(&board_name)
    .fetch_optional(&*database)
    .await?;

    let Some(row) = board_row else {
        return Err(RequestError::NoSuchBoard(board_name.clone()));
    };

    let board_id: i32 = row.try_get("board_id")?;
    let keys = BoardKeys {
        read: StringKey::new(row.try_get::<String, _>("read_key")?),
        submit: StringKey::new(row.try_get::<String, _>("submit_key")?),
        admin: StringKey::new(row.try_get::<String, _>("admin_key")?),
    };
    let authority = api_key.map_or(AuthorityLevel::Unauthorized, |key| {
        keys.check_authority(&key.0)
    });
    Ok((board_id, authority))
}

fn check_auth(auth: AuthorityLevel, required: AuthorityLevel) -> Result<()> {
    if let AuthorityLevel::Unauthorized = auth {
        Err(RequestError::Unathorized)
    } else if auth < required {
        Err(RequestError::Forbidden)
    } else {
        Ok(())
    }
}

fn validate_board_name(name: String) -> Result<String> {
    let name = name.trim().to_owned();
    if name.is_empty() {
        return Err(RequestError::InvalidBoardName(name));
    }
    Ok(name)
}

pub async fn create_board(
    State(database): State<Arc<DatabasePool>>,
    Json(board_name): Json<String>,
) -> Result<Json<BoardKeys>> {
    // Validate the name
    let board_name = validate_board_name(board_name)?;

    // Check if a board with this name already exists
    let check = check_board(Path(board_name.clone()), State(database.clone()), None).await;
    if check.is_ok() {
        return Err(RequestError::BoardAlreadyExists(board_name));
    }

    // Generate keys
    let keys = BoardKeys::generate();

    // Create an entry
    sqlx::query(
        "
INSERT INTO boards (board_name, read_key, submit_key, admin_key)
VALUES (?, ?, ?, ?)
        ",
    )
    .bind(board_name)
    .bind(keys.read.inner())
    .bind(keys.submit.inner())
    .bind(keys.admin.inner())
    .execute(&*database)
    .await?;

    Ok(Json(keys))
}

pub async fn delete_board(
    Path(board_name): Path<String>,
    State(database): State<Arc<DatabasePool>>,
    api_key: Option<ApiKey>,
) -> Result<()> {
    let (board_id, auth) = check_board(Path(board_name), State(database.clone()), api_key).await?;
    check_auth(auth, AuthorityLevel::Admin)?;

    // Delete scores
    sqlx::query("DELETE FROM scores WHERE board_id = ?")
        .bind(board_id)
        .execute(&*database)
        .await?;

    // Delete entry
    sqlx::query("DELETE FROM boards WHERE board_id = ?")
        .bind(board_id)
        .execute(&*database)
        .await?;

    Ok(())
}

pub async fn submit_score(
    Path(board_name): Path<String>,
    Query(PlayerIdQuery { player_id }): Query<PlayerIdQuery>,
    State(database): State<Arc<DatabasePool>>,
    api_key: Option<ApiKey>,
    player_key: PlayerKey,
    Json(score): Json<ctl_core::ScoreEntry>,
) -> Result<()> {
    // Authorize player
    let (real_key, name) = sqlx::query("SELECT key, name FROM players WHERE player_id = ?")
        .bind(player_id)
        .try_map(|row: AnyRow| {
            Ok((
                row.try_get::<String, _>("key")?,
                row.try_get::<String, _>("name")?,
            ))
        })
        .fetch_one(&*database)
        .await?;

    if real_key != player_key.0 {
        // Invalid key
        return Err(RequestError::InvalidPlayer);
    }

    if name != score.player {
        // Name changed
        sqlx::query("UPDATE players SET name = ? WHERE player_id = ?")
            .bind(&score.player)
            .bind(player_id)
            .execute(&*database)
            .await?;
    }

    // Access the board
    let (board_id, auth) = check_board(Path(board_name), State(database.clone()), api_key).await?;
    check_auth(auth, AuthorityLevel::Submit)?;

    // Insert a new score
    sqlx::query("INSERT INTO scores (board_id, player_id, score, extra_info) VALUES (?, ?, ?, ?)")
        .bind(board_id)
        .bind(player_id)
        .bind(score.score)
        .bind(&score.extra_info)
        .execute(&*database)
        .await?;

    Ok(())
}

pub async fn get_scores(
    Path(board_name): Path<String>,
    State(database): State<Arc<DatabasePool>>,
    api_key: Option<ApiKey>,
) -> Result<Json<Vec<ctl_core::ScoreEntry>>> {
    let (board_id, auth) = check_board(Path(board_name), State(database.clone()), api_key).await?;
    check_auth(auth, AuthorityLevel::Read)?;

    // Fetch scores
    let scores = sqlx::query(
        "
SELECT players.name AS player_name, score, extra_info
FROM scores
JOIN players ON scores.player_id = players.player_id
WHERE board_id = ?
        ",
    )
    .bind(board_id)
    .try_map(|row: AnyRow| {
        Ok(ctl_core::ScoreEntry {
            player: row.try_get("player_name")?,
            score: row.try_get("score")?,
            extra_info: row.try_get("extra_info").ok(),
        })
    })
    .fetch_all(&*database)
    .await?;

    Ok(Json(scores))
}
