mod cache;
pub mod fs;

pub use self::cache::*;

use crate::prelude::*;

use std::path::{Path, PathBuf};

use anyhow::Result;

pub struct LocalMusic {
    pub meta: MusicInfo,
    pub sound: Rc<geng::Sound>,
}

impl Debug for LocalMusic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LocalMusic")
            .field("meta", &self.meta)
            .field("sound", &"<bytes>")
            .finish()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupMeta {
    pub music: Option<MusicInfo>,
}

#[derive(Debug, Clone)]
pub struct LocalGroup {
    pub path: PathBuf,
    pub meta: GroupMeta,
    pub music: Option<Rc<LocalMusic>>,
    pub data: LevelSet,
}

#[derive(Debug, Clone)]
pub struct CachedGroup {
    pub local: LocalGroup,
    pub hash: String,
    /// The server version the group on the server, if uploaded.
    pub origin: Option<GroupInfo>,
    pub level_hashes: Vec<String>,
}

impl LocalMusic {
    pub fn new(meta: MusicInfo, mut sound: geng::Sound) -> Self {
        sound.looped = true;
        Self {
            meta,
            sound: Rc::new(sound),
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
