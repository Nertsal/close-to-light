use crate::prelude::*;

use std::path::{Path, PathBuf};

use anyhow::Result;

pub struct LevelCache {
    pub music: HashMap<Id, Rc<CachedMusic>>,
    pub groups: Vec<Rc<CachedGroup>>,
}

pub struct CachedMusic {
    pub meta: MusicMeta,
    pub music: Rc<geng::Sound>,
}

pub struct CachedGroup {
    pub meta: GroupMeta,
    pub music: Option<Rc<CachedMusic>>,
    pub levels: Vec<Rc<CachedLevel>>,
}

pub struct CachedLevel {
    pub path: PathBuf,
    pub meta: LevelMeta,
    // TODO: Rc
    pub level: Level,
    /// Hash code of the level.
    pub hash: String,
}

impl LevelCache {
    /// Load from the local storage.
    pub async fn load(manager: &geng::asset::Manager) -> Result<Self> {
        #[cfg(target_arch = "wasm32")]
        {
            return Ok(Self {
                music: HashMap::new(),
                groups: Vec::new(),
            });
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
        for entry in std::fs::read_dir(base_path.join("levels/local"))? {
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

        Ok(Self { music, groups })
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
            path: path.to_path_buf(),
            meta,
            level,
            hash,
        })
    }
}
