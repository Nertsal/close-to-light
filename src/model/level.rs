mod config;
mod state;

pub use self::{config::*, state::*};

use super::*;

// TODO: move to assets?

type Name = Rc<str>;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GroupMeta {
    pub name: Name,
    /// Music ID.
    pub music: Id,
}

#[derive(geng::asset::Load, Debug, Clone, Serialize, Deserialize, PartialEq)]
#[load(serde = "toml")]
pub struct MusicMeta {
    pub id: Id,
    pub name: Name,
    pub author: Name,
    /// Beats per minute.
    pub bpm: R32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LevelMeta {
    /// 0 id for new levels not yet uploaded to the server.
    #[serde(default)]
    pub id: Id,
    pub name: Name,
    pub author: Name,
}

impl MusicMeta {
    /// Returns the duration (in seconds) of a single beat.
    pub fn beat_time(&self) -> Time {
        r32(60.0) / self.bpm
    }
}
