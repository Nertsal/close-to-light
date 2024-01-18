use super::*;

use color_eyre::eyre::Context;

pub async fn init_database(database: &DatabasePool) -> color_eyre::Result<()> {
    sqlx::query(
        "
CREATE TABLE IF NOT EXISTS boards
(
    board_id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    board_name TEXT NOT NULL,
    read_key TEXT,
    submit_key TEXT,
    admin_key TEXT NOT NULL
)
        ",
    )
    .execute(database)
    .await
    .context("when creating table `boards`")?;

    sqlx::query(
        "
    CREATE TABLE IF NOT EXISTS players
    (
        player_id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
        key TEXT NOT NULL,
        name TEXT NOT NULL
    )
            ",
    )
    .execute(database)
    .await
    .context("when creating table `players`")?;

    sqlx::query(
        "
CREATE TABLE IF NOT EXISTS scores
(
    board_id INTEGER NOT NULL,
    player_id INTEGER NOT NULL,
    score INTEGER NOT NULL,
    extra_info TEXT,
    FOREIGN KEY(board_id) REFERENCES boards(board_id),
    FOREIGN KEY(player_id) REFERENCES players(player_id)
)
        ",
    )
    .execute(database)
    .await
    .context("when creating table `scores`")?;

    Ok(())
}
