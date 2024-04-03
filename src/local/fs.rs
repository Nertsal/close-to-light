use super::*;

#[cfg(not(target_arch = "wasm32"))]
mod native {
    use super::*;

    pub fn download_music(id: Id, data: Vec<u8>, info: &MusicInfo) -> Result<()> {
        let path = music_path(id);
        std::fs::create_dir_all(&path)?;

        std::fs::write(path.join("music.mp3"), data)?;
        std::fs::write(path.join("meta.toml"), toml::to_string_pretty(&info)?)?;

        Ok(())
    }

    pub fn save_level(level: &CachedLevel) -> Result<()> {
        let path = &level.path;
        std::fs::create_dir_all(path)?;

        std::fs::write(
            path.join("level.json"),
            serde_json::to_vec_pretty(&level.data)?,
        )?;
        std::fs::write(path.join("meta.toml"), toml::to_string_pretty(&level.meta)?)?;

        log::debug!(
            "Saved level ({} - {}) successfully",
            level.meta.id,
            level.meta.name
        );

        Ok(())
    }

    pub fn save_group(group: &CachedGroup) -> Result<()> {
        let path = &group.path;
        std::fs::create_dir_all(path)?;

        std::fs::write(path.join("meta.toml"), toml::to_string_pretty(&group.meta)?)?;

        for level in &group.levels {
            save_level(level)?;
        }

        log::debug!("Saved group ({}) successfully", group.meta.id);

        Ok(())
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub use native::*;

/// Path to the directory that hold locally saved levels and music.
pub fn base_path() -> PathBuf {
    preferences::base_path()
}

pub fn all_music_path() -> PathBuf {
    base_path().join("music")
}

pub fn all_groups_path() -> PathBuf {
    base_path().join("levels")
}

pub fn music_path(music: Id) -> PathBuf {
    all_music_path().join(format!("{}", music))
}

pub fn generate_group_path(group: Id) -> PathBuf {
    let base_path = all_groups_path();
    if group == 0 {
        // Generate a random string until it is available
        let mut rng = rand::thread_rng();
        loop {
            let name: String = (0..3).map(|_| rng.gen_range('a'..='z')).collect();
            let path = base_path.join(name);
            if !path.exists() {
                return path;
            }
        }
    } else {
        base_path.join(format!("{}", group))
    }
}

pub fn generate_level_path(group_path: impl AsRef<Path>, level: Id) -> PathBuf {
    let group_path = group_path.as_ref();
    if level == 0 {
        // Generate a random string until it is available
        let mut rng = rand::thread_rng();
        loop {
            let name: String = (0..3).map(|_| rng.gen_range('a'..='z')).collect();
            let path = group_path.join(name);
            if !path.exists() {
                return path;
            }
        }
    } else {
        group_path.join(format!("{}", level))
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

    pub async fn load(
        _manager: &geng::asset::Manager,
        level_path: impl AsRef<Path>,
    ) -> Result<Self> {
        let level_path = level_path.as_ref();

        let meta_path = level_path.join("meta.toml");
        let meta: LevelInfo = file::load_detect(&meta_path).await?;

        let level: Level = file::load_detect(level_path.join("level.json")).await?;

        let hash = {
            use data_encoding::HEXLOWER;
            use sha2::{Digest, Sha256};

            let mut hasher = Sha256::new();

            // let mut reader = std::io::BufReader::new(std::fs::File::open(&level_path)?);
            // let mut buffer = [0; 1024];
            // loop {
            //     let count = reader.read(&mut buffer)?;
            //     if count == 0 {
            //         break;
            //     }
            //     hasher.update(&buffer[..count]);
            // }

            hasher.update(&bincode::serialize(&level)?);

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
