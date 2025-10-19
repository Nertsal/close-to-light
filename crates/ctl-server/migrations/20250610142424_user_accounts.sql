-- User's linked accounts from other platforms.
CREATE TABLE user_linked_accounts
(
    user_id INTEGER NOT NULL,
    discord BLOB,
    github BLOB,
    FOREIGN KEY(user_id) REFERENCES users(user_id)
);

-- Authentication tokens.
CREATE TABLE user_auth_tokens
(
    user_id INTEGER NOT NULL,
    token BLOB,
    expiration_date DATE,
    FOREIGN KEY(user_id) REFERENCES users(user_id)
);
