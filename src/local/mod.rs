mod cache;
mod fs;

pub use self::cache::*;

use crate::prelude::*;

use std::path::{Path, PathBuf};

use anyhow::Result;

pub struct CachedMusic {
    pub meta: MusicInfo,
    pub music: Rc<geng::Sound>,
}

#[derive(Debug)]
pub struct CachedGroup {
    pub path: PathBuf,
    pub meta: GroupMeta,
    pub music: Option<Rc<CachedMusic>>,
    pub levels: Vec<Rc<CachedLevel>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GroupMeta {
    pub id: Id,
    pub music: Id,
}

#[derive(Debug, Clone)]
pub struct CachedLevel {
    /// Path to the folder containing the level data files.
    pub path: PathBuf,
    pub meta: LevelInfo, // TODO: maybe Rc to reduce String allocations
    pub data: Level,     // TODO: Rc
    /// Hash code of the level.
    pub hash: String,
}

impl Debug for CachedMusic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CachedMusic")
            .field("metal", &self.meta)
            .field("music", &"<data>")
            .finish()
    }
}
