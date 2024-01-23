CREATE TABLE IF NOT EXISTS keys
(
    key TEXT NOT NULL,
    submit BIT,
    admin BIT
);

CREATE TABLE IF NOT EXISTS players
(
    player_id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    key TEXT NOT NULL,
    email TEXT,
    name TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS artists
(
    artist_id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    player_id INTEGER
);

CREATE TABLE IF NOT EXISTS scores
(
    level_id INTEGER NOT NULL,
    player_id INTEGER NOT NULL,
    score INTEGER NOT NULL,
    extra_info TEXT,
    FOREIGN KEY(level_id) REFERENCES levels(level_id),
    FOREIGN KEY(player_id) REFERENCES players(player_id)
);

CREATE TABLE IF NOT EXISTS musics
(
    music_id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    name TEXT,
    public BIT,
    original BIT,
    bpm REAL
);

CREATE TABLE IF NOT EXISTS music_authors
(
    artist_id INTEGER NOT NULL,
    music_id INTEGER NOT NULL,
    FOREIGN KEY(artist_id) REFERENCES artists(artist_id),
    FOREIGN KEY(music_id) REFERENCES musics(music_id)
);

CREATE TABLE IF NOT EXISTS groups
(
    group_id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    music_id INTEGER NOT NULL,
    owner_id INTEGER NOT NULL,
    FOREIGN KEY(music_id) REFERENCES musics(music_id),
    FOREIGN KEY(owner_id) REFERENCES players(player_id)
);

CREATE TABLE IF NOT EXISTS levels
(
    level_id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    hash BLOB NOT NULL,
    group_id INTEGER NOT NULL,
    name TEXT
);

CREATE TABLE IF NOT EXISTS level_authors
(
    player_id INTEGER NOT NULL,
    level_id INTEGER NOT NULL,
    FOREIGN KEY(player_id) REFERENCES players(player_id),
    FOREIGN KEY(level_id) REFERENCES levels(level_id)
);
