mod achievements;
mod cache;
pub mod fs;
mod leaderboard;

pub use self::{achievements::*, cache::*, leaderboard::*};

use std::path::{Path, PathBuf};

use anyhow::Result;
use ctl_core::prelude::*;
use generational_arena::Arena;

pub const PLAYER_LOGIN_STORAGE: &str = "user";

#[derive(Clone)]
pub struct LocalMusic {
    pub meta: MusicInfo,
    pub sound: Rc<geng::Sound>,
    /// Raw bytes of the music file, used when saving.
    pub bytes: Rc<[u8]>,
}

impl Debug for LocalMusic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LocalMusic")
            .field("meta", &self.meta)
            .field("sound", &"<bytes>")
            .field("data", &"<bytes>")
            .finish()
    }
}

#[derive(Debug, Clone)]
pub struct LocalGroup {
    /// Path to the directory containing data files.
    pub path: PathBuf,
    /// Whether the group was loaded from the assets folder, as opposed to the custom levels folder.
    pub loaded_from_assets: bool,
    pub meta: LevelSetInfo,
    pub music: Option<Rc<LocalMusic>>,
    pub data: LevelSet,
}

impl LocalGroup {
    pub fn update_hash(&mut self) {
        self.meta.hash = self.data.calculate_hash();
        for (level, meta) in self.data.levels.iter().zip(&mut self.meta.levels) {
            meta.hash = level.calculate_hash();
        }
    }
}

#[derive(Debug, Clone)]
pub struct CachedGroup {
    pub local: LocalGroup,
    /// The server version the group on the server, if uploaded.
    pub origin: Option<LevelSetInfo>,
}

impl CachedGroup {
    pub fn update_hashes(&mut self) {
        self.local.meta.hash = self.local.data.calculate_hash();
        for (meta, level) in self
            .local
            .meta
            .levels
            .iter_mut()
            .zip(self.local.data.levels.iter())
        {
            meta.hash = level.calculate_hash();
        }
    }
}

impl LocalMusic {
    pub fn new(meta: MusicInfo, sound: geng::Sound, bytes: Rc<[u8]>) -> Self {
        Self {
            meta,
            sound: Rc::new(sound),
            bytes,
        }
    }
}

// impl CachedGroup {
//     /// Return the list of map authors in a readable string format.
//     pub fn mappers(&self) -> String {
//         let mut authors: Vec<&str> = self
//             .data
//             .levels
//             .iter()
//             .flat_map(|level| level.meta.authors.iter().map(|user| user.name.as_ref()))
//             .collect();
//         authors.sort();
//         authors.dedup();

//         itertools::Itertools::intersperse(authors.into_iter(), ",").collect::<String>()
//     }
// }
