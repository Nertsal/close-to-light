use crate::prelude::*;

use std::path::{Path, PathBuf};

use anyhow::Result;

pub struct LevelCache {
    manager: geng::asset::Manager,
    pub music: HashMap<Id, Rc<CachedMusic>>,
    pub groups: Vec<Rc<CachedGroup>>,
}

pub struct CachedMusic {
    pub meta: MusicMeta,
    pub music: Rc<geng::Sound>,
}

#[derive(Debug)]
pub struct CachedGroup {
    pub meta: GroupMeta,
    pub music: Option<Rc<CachedMusic>>,
    pub levels: Vec<Rc<CachedLevel>>,
}

#[derive(Debug)]
pub struct CachedLevel {
    pub path: PathBuf,
    pub meta: LevelMeta,
    // TODO: Rc
    pub data: Level,
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
        #[cfg(target_arch = "wasm32")]
        {
            return Ok(Self::new(manager));
        }

        log::info!("Loading local storage");
        let base_path = preferences::base_path();

        let mut music = HashMap::new();
        for entry in std::fs::read_dir(base_path.join("music"))? {
            let entry = entry?;
            let path = entry.path();
            if !path.is_dir() {
                log::error!("Unexpected file in music dir: {:?}", path);
                continue;
            }

            let id: Id = entry
                .file_name()
                .to_str()
                .ok_or(anyhow!("Directory name is not valid UTF-8"))?
                .parse()?;

            let m = CachedMusic::load(manager, &path).await?;
            music.insert(id, Rc::new(m));
        }

        let mut groups = Vec::new();
        for entry in std::fs::read_dir(base_path.join("levels"))? {
            let entry = entry?;
            let path = entry.path();
            if !path.is_dir() {
                log::error!("Unexpected file in levels dir: {:?}", path);
                continue;
            }

            let mut group = CachedGroup::load(manager, &path).await?;
            group.music = music.get(&group.meta.music).cloned();
            groups.push(Rc::new(group));
        }

        Ok(Self {
            manager: manager.clone(),
            music,
            groups,
        })
    }

    pub async fn load_level(
        &mut self,
        level_path: impl AsRef<std::path::Path>,
    ) -> Result<(Rc<CachedGroup>, Rc<CachedLevel>)> {
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
        let mut group = CachedGroup::load(&self.manager, &group_path).await?;

        let music = match self.music.get(&group.meta.music) {
            Some(music) => music.clone(),
            None => {
                let music_path =
                    preferences::base_path().join(format!("music/{}", group.meta.music));
                let music = Rc::new(CachedMusic::load(&self.manager, &music_path).await?);
                self.music.insert(group.meta.music, music.clone());
                music
            }
        };
        group.music = Some(music.clone());

        let group = Rc::new(group);
        self.groups.push(group.clone());

        let level = group
            .levels
            .iter()
            .find(|level| level.path == level_path)
            .ok_or(anyhow!("Specific level not found"))?
            .clone();

        Ok((group, level))
    }
}

impl CachedMusic {
    pub async fn load(manager: &geng::asset::Manager, path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();

        let meta_path = path.join("meta.toml");
        let meta: MusicMeta = file::load_detect(&meta_path).await?;

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
    pub async fn load(manager: &geng::asset::Manager, path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();

        let meta_path = path.join("meta.toml");
        let meta: GroupMeta = file::load_detect(&meta_path).await?;

        let mut levels = Vec::new();
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }

            let level = CachedLevel::load(manager, &path).await?;
            levels.push(Rc::new(level));
        }

        Ok(Self {
            meta,
            music: None,
            levels,
        })
    }
}

impl CachedLevel {
    pub async fn load(_manager: &geng::asset::Manager, path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();

        let meta_path = path.join("meta.toml");
        let meta: LevelMeta = file::load_detect(&meta_path).await?;

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
