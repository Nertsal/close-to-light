-- Music authors
CREATE TABLE musicians
(
    musician_id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    romanized_name TEXT NOT NULL,
    user_id INTEGER,
    created_at DATE NOT NULL
);

-- Individual music files.
-- Raw mp3 data is stored separately.
CREATE TABLE musics
(
    music_id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    uploaded_by_user INTEGER NOT NULL,
    name TEXT NOT NULL,
    romanized_name TEXT NOT NULL,
    original BIT NOT NULL, -- Whether the music was created specifically for Close to Light
    featured BIT NOT NULL, -- Whether the music was agreed with the creator to include in the game
    hash BLOB NOT NULL,
    FOREIGN KEY(uploaded_by_user) REFERENCES users(user_id)
);

-- (Many-many) Relationship between `musics` and `musicians`.
CREATE TABLE music_authors
(
    musician_id INTEGER,
    music_id INTEGER NOT NULL,
    name TEXT NOT NULL,
    romanized_name TEXT NOT NULL,
    FOREIGN KEY(musician_id) REFERENCES musicians(musician_id),
    FOREIGN KEY(music_id) REFERENCES musics(music_id)
);

-- Level sets for a specific music, containing multiple levels.
-- The data of the level-set itself is stored separately.
CREATE TABLE level_sets
(
    level_set_id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    music_id INTEGER NOT NULL,
    owner_id INTEGER NOT NULL,
    featured BIT NOT NULL,
    hash BLOB NOT NULL,
    created_at DATE NOT NULL,
    FOREIGN KEY(music_id) REFERENCES musics(music_id),
    FOREIGN KEY(owner_id) REFERENCES users(user_id)
);

-- Individual playable levels in a level-set.
-- The data of the level itself is stored separately.
CREATE TABLE levels
(
    level_id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    level_set_id INTEGER NOT NULL,
    enabled BIT NOT NULL,
    name TEXT NOT NULL,
    ord INTEGER NOT NULL,
    hash BLOB NOT NULL,
    created_at DATE NOT NULL,
    FOREIGN KEY(level_set_id) REFERENCES level_sets(level_set_id)
);

-- Relationship between `levels` and `users`.
CREATE TABLE level_authors
(
    user_id INTEGER,
    level_id INTEGER NOT NULL,
    name TEXT NOT NULL,
    romanized_name TEXT NOT NULL,
    FOREIGN KEY(user_id) REFERENCES users(user_id),
    FOREIGN KEY(level_id) REFERENCES levels(level_id)
);
