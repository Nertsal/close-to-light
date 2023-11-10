mod saved;
mod state;

pub use self::{saved::*, state::*};

use super::*;

type Name = Rc<str>;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GroupMeta {
    pub name: Name,
    /// Music info
    pub music: MusicMeta,
    /// Normal level info.
    pub normal: LevelMeta,
    /// Hard level info.
    pub hard: LevelMeta,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MusicMeta {
    pub author: Name,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LevelMeta {
    pub author: Name,
}
