use super::*;

use ctl_core::types::{MusicianInfo, UserInfo};
use sqlx::FromRow;

pub type DatabasePool = sqlx::SqlitePool; // TODO: behind a trait?
pub type DBRow = sqlx::sqlite::SqliteRow;

pub type Score = i32;

#[derive(Debug, Clone, FromRow)]
pub struct UserRow {
    pub user_id: Id,
    pub username: String,
}

impl From<UserRow> for UserInfo {
    fn from(val: UserRow) -> Self {
        Self {
            id: val.user_id,
            name: val.username.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ScoreRow {
    pub level_set_id: Id,
    pub level_id: Id,
    pub level_hash: String,
    pub user_id: Id,
    pub score: Score,
    pub extra_info: Option<String>,
    pub submitted_at: OffsetDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct MusicianRow {
    pub musician_id: Id,
    pub name: String,
    pub romanized_name: String,
    pub user_id: Option<Id>,
    pub created_at: OffsetDateTime,
}

impl From<MusicianRow> for MusicianInfo {
    fn from(value: MusicianRow) -> Self {
        Self {
            id: value.musician_id,
            name: value.name.into(),
            romanized: value.romanized_name.into(),
            user: value.user_id,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct MusicRow {
    pub music_id: Id,
    pub name: String,
    pub romanized_name: String,
    pub original: bool,
    pub featured: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct LevelSetRow {
    pub level_set_id: Id,
    pub music_id: Id,
    pub owner_id: Id,
    pub hash: String,
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct LevelRow {
    pub level_id: Id,
    pub level_set_id: Id,
    pub name: String,
    pub order: i32,
    pub hash: String,
    pub created_at: OffsetDateTime,
}
