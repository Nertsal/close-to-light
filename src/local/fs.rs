use super::*;

/// Path to the directory that hold locally saved levels and music.
pub fn base_path() -> PathBuf {
    preferences::base_path()
}

pub fn all_music_path() -> PathBuf {
    base_path().join("music")
}

pub fn all_levels_path() -> PathBuf {
    base_path().join("levels")
}

pub fn music_path(music: Id) -> PathBuf {
    all_music_path().join(format!("{}", music))
}

pub fn level_path(level: Id) -> PathBuf {
    all_levels_path().join(format!("{}", level))
}

pub fn download_music(id: Id, data: Vec<u8>, info: &MusicInfo) -> Result<()> {
    let music_path = music_path(id);
    std::fs::create_dir_all(&music_path)?;

    std::fs::write(music_path.join("music.mp3"), data)?;
    std::fs::write(music_path.join("meta.toml"), toml::to_string_pretty(&info)?)?;

    Ok(())
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
