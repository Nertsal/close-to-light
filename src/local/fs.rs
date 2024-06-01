use crate::assets::MusicAssets;

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

    pub fn save_group(group: &CachedGroup) -> Result<()> {
        let path = &group.path;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let writer = std::io::BufWriter::new(std::fs::File::create(path)?);
        bincode::serialize_into(writer, &group.data)?;

        log::debug!("Saved group ({}) successfully", group.data.id);

        Ok(())
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub use native::*;

/// Path to the directory that hold locally saved levels and music.
pub fn base_path() -> PathBuf {
    #[cfg(target_arch = "wasm32")]
    {
        "close-to-light".into()
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let exe = std::env::current_exe().expect("Failed to find current exe");
        let app_name = exe.file_stem().unwrap();
        if let Some(dirs) =
            directories::ProjectDirs::from("", "", app_name.to_str().expect("Exe name is invalid"))
        {
            return dirs.data_dir().to_path_buf();
        }
        if let Some(dir) = exe.parent() {
            return dir.to_path_buf();
        }
        std::env::current_dir().unwrap()
    }
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
            let name: String = (0..5).map(|_| rng.gen_range('a'..='z')).collect();
            let path = base_path.join(name);
            if !path.exists() {
                return path;
            }
        }
    } else {
        base_path.join(format!("{}", group))
    }
}

impl CachedMusic {
    pub async fn load(manager: &geng::asset::Manager, path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let assets: MusicAssets = geng::asset::Load::load(manager, path, &()).await?;
        Ok(Self::new(assets.meta, assets.music))
    }
}

impl CachedGroup {
    pub fn new(data: LevelSet) -> Self {
        let level_hashes = data
            .levels
            .iter()
            .map(|level| level.data.calculate_hash())
            .collect();

        Self {
            path: fs::generate_group_path(data.id),
            music: None,
            hash: data.calculate_hash(),
            origin: None,
            data,
            level_hashes,
        }
    }
}
