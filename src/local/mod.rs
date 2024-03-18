use crate::prelude::*;

use std::path::{Path, PathBuf};

use anyhow::Result;

pub struct LevelCache {
    manager: geng::asset::Manager,
    pub music: HashMap<Id, Rc<CachedMusic>>,
    pub groups: Vec<CachedGroup>,
}

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

#[derive(Debug)]
pub struct CachedLevel {
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

impl LevelCache {
    pub fn new(manager: &geng::asset::Manager) -> Self {
        Self {
            manager: manager.clone(),
            music: HashMap::new(),
            groups: Vec::new(),
        }
    }

    /// Load from the local storage.
    pub async fn load(manager: &geng::asset::Manager) -> Result<Self> {
        // TODO: report failures but continue working

        #[cfg(target_arch = "wasm32")]
        {
            return Ok(Self::new(manager));
        }

        log::info!("Loading local storage");
        let base_path = preferences::base_path();

        // let mut music = HashMap::new();
        // for entry in std::fs::read_dir(base_path.join("music"))? {
        //     let entry = entry?;
        //     let path = entry.path();
        //     if !path.is_dir() {
        //         log::error!("Unexpected file in music dir: {:?}", path);
        //         continue;
        //     }

        //     let id: Id = entry
        //         .file_name()
        //         .to_str()
        //         .ok_or(anyhow!("Directory name is not valid UTF-8"))?
        //         .parse()?;

        //     let m = CachedMusic::load(manager, &path).await?;
        //     music.insert(id, Rc::new(m));
        // }

        let mut local = Self {
            manager: manager.clone(),
            music: HashMap::new(),
            groups: Vec::new(),
        };

        for entry in std::fs::read_dir(base_path.join("levels"))? {
            let entry = entry?;
            let path = entry.path();
            if !path.is_dir() {
                log::error!("Unexpected file in levels dir: {:?}", path);
                continue;
            }

            local.load_group_all(&path).await?;
        }

        Ok(local)
    }

    pub async fn load_level(
        &mut self,
        level_path: impl AsRef<std::path::Path>,
    ) -> Result<(Rc<CachedMusic>, Rc<CachedLevel>)> {
        let level_path = level_path.as_ref();
        let (level_path, group_path) = if level_path.is_dir() {
            (
                level_path.join("level.json"),
                level_path
                    .parent()
                    .ok_or(anyhow!("Level expected to be in a folder"))?,
            )
        } else {
            // Assume path to `level.json`
            (
                level_path.to_path_buf(),
                level_path
                    .parent()
                    .ok_or(anyhow!("Level expected to be in a folder"))?
                    .parent()
                    .ok_or(anyhow!("Level expected to be in a folder"))?,
            )
        };

        // TODO: do not load all the group levels
        self.load_group_all(&group_path).await?;

        // If `load_group_empty` succedes, the group is pushed to the end
        let group = self.groups.last().unwrap();

        let music = group
            .music
            .clone()
            .ok_or(anyhow!("Group music not found"))?;

        let level = group
            .levels
            .iter()
            .find(|level| level.path == level_path)
            .ok_or(anyhow!("Specific level not found"))?
            .clone();

        Ok((music, level))
    }

    /// Load the group info at the given path without loading the levels.
    async fn load_group_empty(&mut self, path: impl AsRef<std::path::Path>) -> Result<()> {
        let group_path = path.as_ref().to_path_buf();

        let meta_path = group_path.join("meta.toml");
        let meta: GroupMeta = file::load_detect(&meta_path).await?;

        let music = match self.music.get(&meta.music) {
            Some(music) => Some(music.clone()),
            None => {
                let music_path = preferences::base_path().join(format!("music/{}", meta.music));
                CachedMusic::load(&self.manager, &music_path)
                    .await
                    .ok()
                    .map(|music| {
                        let music = Rc::new(music);
                        self.music.insert(meta.music, music.clone());
                        music
                    })
            }
        };

        let group = CachedGroup {
            path: group_path,
            meta,
            music,
            levels: Vec::new(),
        };
        self.groups.push(group);

        Ok(())
    }

    /// Load the group and all levels from it.
    async fn load_group_all(&mut self, path: impl AsRef<std::path::Path>) -> Result<()> {
        let group_path = path.as_ref();
        self.load_group_empty(group_path).await?;

        // If `load_group_empty` succedes, the group is pushed to the end
        let group = self.groups.last_mut().unwrap();

        let mut levels = Vec::new();
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }

            let level = CachedLevel::load(&self.manager, &path).await?;
            levels.push(Rc::new(level));
        }

        group.levels.extend(levels);
        Ok(())
    }

    pub fn new_group(&mut self, music_id: Id) {
        let music = self.music.get(&music_id).cloned();
        let mut group = CachedGroup::new(GroupMeta {
            id: 0,
            music: music_id,
        });
        group.music = music;
        self.groups.push(group);
        // TODO: write to fs
    }

    pub fn new_level(&mut self, group: usize, meta: LevelInfo) {
        if let Some(group) = self.groups.get_mut(group) {
            let level = CachedLevel::new(meta);
            group.levels.push(Rc::new(level));
            // TODO: write to fs
        }
    }
}

impl CachedMusic {
    pub async fn load(manager: &geng::asset::Manager, path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();

        let meta_path = path.join("meta.toml");
        let meta: MusicInfo = file::load_detect(&meta_path).await?;

        let file_path = path.join("music.mp3");
        let file: geng::Sound = geng::asset::Load::load(
            manager,
            &file_path,
            &geng::asset::SoundOptions { looped: false },
        )
        .await?;

        Ok(Self {
            meta,
            music: Rc::new(file),
        })
    }
}

impl CachedGroup {
    pub fn new(meta: GroupMeta) -> Self {
        Self {
            path: PathBuf::new(), // TODO
            meta,
            music: None,
            levels: Vec::new(),
        }
    }
}

impl CachedLevel {
    pub fn new(meta: LevelInfo) -> Self {
        Self {
            path: PathBuf::new(), // TODO
            meta,
            data: Level::new(),
            hash: String::new(),
        }
    }

    pub async fn load(_manager: &geng::asset::Manager, path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();

        let meta_path = path.join("meta.toml");
        let meta: LevelInfo = file::load_detect(&meta_path).await?;

        let level_path = path.join("level.json");
        let level: Level = file::load_detect(&level_path).await?;

        let hash = {
            use data_encoding::HEXLOWER;
            use sha2::{Digest, Sha256};

            let mut hasher = Sha256::new();
            let mut reader = std::io::BufReader::new(std::fs::File::open(&level_path)?);
            let mut buffer = [0; 1024];
            loop {
                let count = reader.read(&mut buffer)?;
                if count == 0 {
                    break;
                }
                hasher.update(&buffer[..count]);
            }
            HEXLOWER.encode(hasher.finalize().as_ref())
        };

        Ok(Self {
            path: level_path.to_path_buf(),
            meta,
            data: level,
            hash,
        })
    }
}
