use super::*;

use ctl_core::types::UserInfo;
use sqlx::FromRow;

pub type DatabasePool = sqlx::SqlitePool; // TODO: behind a trait?
pub type DBRow = sqlx::sqlite::SqliteRow;

pub type Score = i32;

#[derive(FromRow)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreRecord {
    pub player_id: Id,
    pub score: Score,
    pub extra_info: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct MusicRow {
    pub music_id: Id,
    pub name: String,
    pub original: bool,
    pub bpm: f32,
    pub public: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct GroupRow {
    pub group_id: Id,
    pub music_id: Id,
    pub owner_id: Id,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct LevelRow {
    pub level_id: Id,
    pub hash: String,
    pub group_id: Id,
    pub name: String,
}
