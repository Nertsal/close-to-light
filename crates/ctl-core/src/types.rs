use geng::prelude::batbox::prelude::*;
use uuid::Uuid;

pub type Time = R32;
pub type Coord = R32;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GroupInfo {
    pub id: Uuid,
    pub music: MusicInfo,
    pub levels: Vec<LevelInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MusicInfo {
    pub id: Uuid,
    pub name: String,
    pub authors: Vec<PlayerInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LevelInfo {
    pub id: Uuid,
    pub name: String,
    pub authors: Vec<PlayerInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PlayerInfo {
    pub id: Uuid,
    pub name: String,
}
