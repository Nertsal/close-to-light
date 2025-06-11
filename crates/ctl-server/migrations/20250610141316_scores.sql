CREATE TABLE scores
(
    level_set_id INTEGER NOT NULL,
    level_id INTEGER NOT NULL,
    level_hash BLOB NOT NULL,
    user_id INTEGER NOT NULL,
    score INTEGER NOT NULL,
    extra_info TEXT,
    submitted_at DATE NOT NULL,
    FOREIGN KEY(level_set_id) REFERENCES level_sets(level_set_id),
    FOREIGN KEY(level_id) REFERENCES levels(level_id),
    FOREIGN KEY(user_id) REFERENCES users(user_id)
);
