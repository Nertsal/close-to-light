CREATE TABLE groups_recommended
(
    group_id INTEGER NOT NULL,
    FOREIGN KEY(group_id) REFERENCES groups(group_id)
);
