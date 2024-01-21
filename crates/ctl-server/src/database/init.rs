use super::*;

use color_eyre::eyre::Context;

pub async fn init_database(database: &DatabasePool) -> color_eyre::Result<()> {
    sqlx::query(
        "
    CREATE TABLE IF NOT EXISTS keys
    (
        key TEXT NOT NULL,
        submit BIT,
        admin BIT
    )
            ",
    )
    .execute(database)
    .await
    .context("when creating table `keys`")?;

    sqlx::query(
        "
    CREATE TABLE IF NOT EXISTS players
    (
        player_id BLOB NOT NULL PRIMARY KEY,
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
    level_id BLOB NOT NULL,
    player_id BLOB NOT NULL,
    score INTEGER NOT NULL,
    extra_info TEXT,
    FOREIGN KEY(level_id) REFERENCES levels(level_id),
    FOREIGN KEY(player_id) REFERENCES players(player_id)
)
        ",
    )
    .execute(database)
    .await
    .context("when creating table `scores`")?;

    sqlx::query(
        "
CREATE TABLE IF NOT EXISTS musics
(
    music_id BLOB NOT NULL PRIMARY KEY,
    name TEXT,
    file_path TEXT
)
        ",
    )
    .execute(database)
    .await
    .context("when creating table `musics`")?;

    sqlx::query(
        "
CREATE TABLE IF NOT EXISTS music_authors
(
    player_id BLOB NOT NULL,
    music_id BLOB NOT NULL,
    FOREIGN KEY(player_id) REFERENCES players(player_id),
    FOREIGN KEY(music_id) REFERENCES musics(music_id)
)
        ",
    )
    .execute(database)
    .await
    .context("when creating table `music_authors`")?;

    sqlx::query(
        "
CREATE TABLE IF NOT EXISTS groups
(
    group_id BLOB NOT NULL PRIMARY KEY,
    music_id BLOB NOT NULL,
    FOREIGN KEY(music_id) REFERENCES musics(music_id)
)
        ",
    )
    .execute(database)
    .await
    .context("when creating table `groups`")?;

    sqlx::query(
        "
CREATE TABLE IF NOT EXISTS levels
(
    level_id BLOB NOT NULL PRIMARY KEY,
    group_id BLOB NOT NULL,
    name TEXT,
    file_path TEXT
)
        ",
    )
    .execute(database)
    .await
    .context("when creating table `levels`")?;

    sqlx::query(
        "
CREATE TABLE IF NOT EXISTS level_authors
(
    player_id BLOB NOT NULL,
    level_id BLOB NOT NULL,
    FOREIGN KEY(player_id) REFERENCES players(player_id),
    FOREIGN KEY(level_id) REFERENCES levels(level_id)
)
        ",
    )
    .execute(database)
    .await
    .context("when creating table `level_authors`")?;

    Ok(())
}
