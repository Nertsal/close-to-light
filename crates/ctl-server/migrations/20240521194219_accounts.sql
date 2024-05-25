CREATE TABLE user_accounts
(
    user_id INTEGER NOT NULL,
    discord BLOB,
    github BLOB,
    FOREIGN KEY(user_id) REFERENCES users(user_id)
);

CREATE TABLE user_tokens
(
    user_id INTEGER NOT NULL,
    token BLOB,
    FOREIGN KEY(user_id) REFERENCES users(user_id)
);
