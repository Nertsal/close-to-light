#[cfg(not(target_arch = "wasm32"))]
mod native;
#[cfg(target_arch = "wasm32")]
mod web;

use super::*;

pub struct Controller {
    #[cfg(target_arch = "wasm32")]
    rexie: rexie::Rexie,
    geng: Geng,
}

impl Controller {
    pub async fn new(geng: &Geng) -> Result<Self> {
        #[cfg(target_arch = "wasm32")]
        {
            let rexie = match web::build_database().await {
                Ok(rexie) => rexie,
                Err(err) => {
                    log::error!("failed to init web file system: {:?}", err);
                    anyhow::bail!("check logs");
                }
            };
            log::info!("Connected to browser indexed db");
            Ok(Self {
                rexie,
                geng: geng.clone(),
            })
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let base_path = base_path();
            std::fs::create_dir_all(base_path)?;
            Ok(Self { geng: geng.clone() })
        }
    }

    pub async fn load_groups_all(&self) -> Result<Vec<LocalGroup>> {
        log::debug!("Loading all local groups");

        #[cfg(target_arch = "wasm32")]
        let groups: Result<Vec<(PathBuf, LevelSet)>> = {
            match web::load_groups_all(&self.rexie).await {
                Ok(items) => Ok(items),
                Err(err) => {
                    log::error!("failed to load groups from web file system: {:?}", err);
                    anyhow::bail!("check logs");
                }
            }
        };

        #[cfg(not(target_arch = "wasm32"))]
        let groups = { native::load_groups_all(&self.geng).await };

        let mut groups = groups?;
        if let Ok(assets) = load_groups_all_assets(&self.geng).await {
            // If a group is already loaded from local storage
            // ignore its presence in assets
            let ids: Vec<_> = groups
                .iter()
                .map(|group| group.data.id)
                .filter(|id| *id != 0)
                .collect();
            groups.extend(
                assets
                    .into_iter()
                    .filter(|group| !ids.contains(&group.data.id)),
            );
        }
        Ok(groups)
    }

    pub async fn save_group(&self, group: &CachedGroup) -> Result<()> {
        log::debug!("Saving group: {}", group.local.data.id);
        #[cfg(target_arch = "wasm32")]
        {
            let id = group.data.id.to_string();
            let id = group
                .path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or(&id);
            if let Err(err) = web::save_group(&self.rexie, group, id).await {
                log::error!("failed to save group into web file system: {:?}", err);
                anyhow::bail!("check logs");
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            native::save_group(group)?;
        }
        Ok(())
    }

    pub async fn remove_group(&self, path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();
        log::debug!("Deleting a group: {:?}", path);
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(id) = path.file_name().and_then(|name| name.to_str()) {
                if let Err(err) = web::remove_group(&self.rexie, id).await {
                    log::error!("failed to remove group from the web file system: {:?}", err);
                    anyhow::bail!("check logs");
                }
            }
            Ok(())
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            std::fs::remove_dir_all(path)?;
            Ok(())
        }
    }
}

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

pub fn all_groups_path() -> PathBuf {
    base_path().join("levels")
}

pub fn generate_group_path(group: Id) -> PathBuf {
    let base_path = all_groups_path();
    if group == 0 {
        // Generate a random string until it is available
        let mut rng = rand::thread_rng();
        loop {
            let name: String = (0..5).map(|_| rng.gen_range('a'..='z')).collect();
            let path = base_path.join(name);
            // TODO: validate on web
            if !path.exists() {
                return path;
            }
        }
    } else {
        base_path.join(format!("{}", group))
    }
}

fn decode_group(bytes: &[u8]) -> Result<LevelSet> {
    match cbor4ii::serde::from_slice(bytes) {
        Ok(value) => Ok(value),
        Err(err) => {
            // Try legacy version, for backwards compatibility
            if let Ok(value) = bincode::deserialize::<ctl_client::core::legacy::v1::LevelSet>(bytes)
            {
                // assuming some default
                // proper conversion has to be done manually
                let beat_time = r32(60.0) / r32(150.0);
                let value = ctl_client::core::legacy::v1::convert_group(beat_time, value);
                return Ok(value);
            }
            Err(err.into())
        }
    }
}

async fn load_groups_all_assets(geng: &Geng) -> Result<Vec<LocalGroup>> {
    log::debug!("Loading groups from assets");
    let groups_path = run_dir().join("assets").join("groups");

    let list: Vec<String> = file::load_detect(groups_path.join("_list.ron")).await?;
    let paths: Vec<_> = list
        .into_iter()
        .map(|entry| groups_path.join(entry))
        .collect();

    let asset_manager = geng.asset_manager();
    let load_group = |path: PathBuf| async move {
        let context = format!("when loading: {:?}", path);

        let result = async move {
            let bytes = file::load_bytes(&path.join("levels.cbor")).await?;
            let group: LevelSet = decode_group(&bytes)?;

            let music: Option<geng::Sound> = geng::asset::Load::load(
                asset_manager,
                &path.join("music.mp3"),
                &geng::asset::SoundOptions { looped: true },
            )
            .await
            .ok();

            let meta: GroupMeta = file::load_detect(path.join("meta.toml")).await?;

            let music_meta = meta.music.clone().unwrap_or_default();
            let music = music.map(|music| Rc::new(LocalMusic::new(music_meta, music)));

            let local = LocalGroup {
                path,
                meta,
                music,
                data: group,
            };

            anyhow::Ok(local)
        };

        result.await.context(context)
    };

    let group_loaders = paths.into_iter().map(load_group);
    let groups = future::join_all(group_loaders).await;

    let mut res = Vec::new();
    for group in groups {
        match group {
            Ok(local) => {
                log::debug!("Loaded group from {:?}", local.path);
                res.push(local);
            }
            Err(err) => {
                log::error!("failed to load group: {:?}", err);
            }
        }
    }

    Ok(res)
}
