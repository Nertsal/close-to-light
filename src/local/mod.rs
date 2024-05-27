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

#[derive(Debug, Clone)]
pub struct CachedGroup {
    pub path: PathBuf,
    pub music: Option<Rc<CachedMusic>>,
    pub data: LevelSet,
    pub hash: String,
    /// The hash of the group on the server, if uploaded.
    pub origin_hash: Option<String>,
}

impl Debug for CachedMusic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CachedMusic")
            .field("metal", &self.meta)
            .field("music", &"<data>")
            .finish()
    }
}

impl CachedMusic {
    pub fn new(meta: MusicInfo, mut music: geng::Sound) -> Self {
        music.looped = true;
        Self {
            meta,
            music: Rc::new(music),
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
