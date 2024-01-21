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
    CREATE TABLE IF NOT EXISTS artists
    (
        artist_id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
        name TEXT NOT NULL,
        player_id INTEGER
    )
            ",
    )
    .execute(database)
    .await
    .context("when creating table `artists`")?;

    sqlx::query(
        "
CREATE TABLE IF NOT EXISTS scores
(
    level_id INTEGER NOT NULL,
    player_id INTEGER NOT NULL,
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
    music_id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    name TEXT,
    public BIT,
    original BIT,
    bpm REAL
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
    artist_id INTEGER NOT NULL,
    music_id INTEGER NOT NULL,
    FOREIGN KEY(artist_id) REFERENCES artists(artist_id),
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
    group_id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    music_id INTEGER NOT NULL,
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
    level_id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    group_id INTEGER NOT NULL,
    name TEXT
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
    player_id INTEGER NOT NULL,
    level_id INTEGER NOT NULL,
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
