use super::*;

use sqlx::FromRow;

pub type DatabasePool = sqlx::SqlitePool; // TODO: behind a trait?
pub type DBRow = sqlx::sqlite::SqliteRow;

pub type Score = i32;

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
pub struct LevelRow {
    pub level_id: Id,
    pub hash: String,
    pub group_id: Id,
    pub name: String,
}
