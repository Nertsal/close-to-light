-- Users/players of the game.
CREATE TABLE users
(
    user_id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    username TEXT NOT NULL,
    password TEXT,
    created_at DATE NOT NULL
);

-- Users marked as admins are given privileged server access.
CREATE TABLE admins
(
    user_id INTEGER NOT NULL
);
