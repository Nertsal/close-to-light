use serde::{Deserialize, Serialize};

pub type Score = i32;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScoreEntry {
    pub player: String,
    pub score: Score,
    pub extra_info: Option<String>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Player {
    pub id: i32,
    /// Secret key used to authenticate.
    pub key: String,
    pub name: String,
}
