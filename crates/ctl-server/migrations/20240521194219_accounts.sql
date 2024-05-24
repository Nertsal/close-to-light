CREATE TABLE user_accounts
(
    user_id INTEGER NOT NULL,
    discord BLOB,
    github BLOB,
    FOREIGN KEY(user_id) REFERENCES users(user_id)
);
