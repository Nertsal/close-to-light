mod config;
mod saved;
mod state;

pub use self::{config::*, saved::*, state::*};

use super::*;

type Name = Rc<str>;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GroupMeta {
    pub name: Name,
    /// Music info
    pub music: MusicMeta,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MusicMeta {
    /// Beats per minute.
    pub bpm: R32,
    pub author: Name,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LevelMeta {
    pub name: Name,
    pub author: Name,
}

impl MusicMeta {
    /// Returns the duration (in seconds) of a single beat.
    pub fn beat_time(&self) -> Time {
        r32(60.0) / self.bpm
    }
}
