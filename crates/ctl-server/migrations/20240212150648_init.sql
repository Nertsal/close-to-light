CREATE TABLE admins
(
    user_id INTEGER NOT NULL
);

CREATE TABLE users
(
    user_id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    username TEXT NOT NULL,
    password TEXT NOT NULL
);

CREATE TABLE artists
(
    artist_id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    user_id INTEGER
);

CREATE TABLE scores
(
    level_id INTEGER NOT NULL,
    user_id INTEGER NOT NULL,
    score INTEGER NOT NULL,
    extra_info TEXT,
    FOREIGN KEY(level_id) REFERENCES levels(level_id),
    FOREIGN KEY(user_id) REFERENCES users(user_id)
);

CREATE TABLE musics
(
    music_id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    name TEXT,
    public BIT,
    original BIT,
    bpm REAL
);

CREATE TABLE music_authors
(
    artist_id INTEGER NOT NULL,
    music_id INTEGER NOT NULL,
    FOREIGN KEY(artist_id) REFERENCES artists(artist_id),
    FOREIGN KEY(music_id) REFERENCES musics(music_id)
);

CREATE TABLE groups
(
    group_id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    music_id INTEGER NOT NULL,
    owner_id INTEGER NOT NULL,
    FOREIGN KEY(music_id) REFERENCES musics(music_id),
    FOREIGN KEY(owner_id) REFERENCES users(user_id)
);

CREATE TABLE levels
(
    level_id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    hash BLOB NOT NULL,
    group_id INTEGER NOT NULL,
    name TEXT
);

CREATE TABLE level_authors
(
    user_id INTEGER NOT NULL,
    level_id INTEGER NOT NULL,
    FOREIGN KEY(user_id) REFERENCES users(user_id),
    FOREIGN KEY(level_id) REFERENCES levels(level_id)
);
