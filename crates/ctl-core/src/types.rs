use geng::prelude::batbox::prelude::*;

pub type Id = u32;
pub type Time = R32;
pub type Coord = R32;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GroupInfo {
    pub id: Id,
    pub music: MusicInfo,
    pub levels: Vec<LevelInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MusicInfo {
    pub id: Id,
    pub name: String,
    pub authors: Vec<PlayerInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LevelInfo {
    pub id: Id,
    pub name: String,
    pub authors: Vec<PlayerInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PlayerInfo {
    pub id: Id,
    pub name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NewMusic {
    pub name: String,
    pub original: bool,
    pub bpm: f32,
}
