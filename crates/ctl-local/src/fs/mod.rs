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
        let groups: Result<Vec<LocalGroup>> = {
            match web::load_groups_all(&self.geng, &self.rexie).await {
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
                .map(|group| group.meta.id)
                .filter(|id| *id != 0)
                .collect();
            groups.extend(
                assets
                    .into_iter()
                    .filter(|group| !ids.contains(&group.meta.id)),
            );
        }
        Ok(groups)
    }

    /// Saves group in the local filesystem.
    /// If `save_music` is false, skips writing music to the filesystem.
    pub async fn save_group(&self, group: &CachedGroup, save_music: bool) -> Result<()> {
        log::debug!("Saving group: {}", group.local.meta.id);
        #[cfg(target_arch = "wasm32")]
        {
            let id = group.local.meta.id.to_string();
            let id = group
                .local
                .path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or(&id);
            let music = group.local.music.as_ref().map(|music| &*music.bytes);
            // TODO: save_music
            if let Err(err) = web::save_group(&self.rexie, group, music, id).await {
                log::error!("failed to save group into web file system: {:?}", err);
                anyhow::bail!("check logs");
            }
            if save_music {
                log::warn!("saving music on web is not supported yet");
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            native::save_group(group, save_music)?;
        }
        Ok(())
    }

    pub async fn copy_music_from(
        &self,
        source: impl AsRef<Path>,
        destination: impl AsRef<Path>,
    ) -> Result<()> {
        #[cfg(target_arch = "wasm32")]
        {
            let _ = source;
            let _ = destination;
            Ok(())
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            std::fs::copy(source, destination)?;
            Ok(())
        }
    }

    pub async fn remove_group(&self, path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();
        log::debug!("Deleting a group: {path:?}");
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

    pub async fn load_local_scores(&self, level_hash: &str) -> Result<Vec<SavedScore>> {
        #[cfg(target_arch = "wasm32")]
        {
            match web::load_local_scores(&self.rexie, level_hash).await {
                Ok(res) => Ok(res),
                Err(err) => {
                    log::error!("failed to load local scores the web file system: {:?}", err);
                    anyhow::bail!("check logs");
                }
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            native::load_local_scores(level_hash)
        }
    }

    pub async fn save_local_scores(&self, level_hash: &str, scores: &[SavedScore]) -> Result<()> {
        #[cfg(target_arch = "wasm32")]
        {
            if let Err(err) = web::save_local_scores(&self.rexie, level_hash, scores).await {
                log::error!(
                    "failed to save local scores to the web file system: {:?}",
                    err
                );
                anyhow::bail!("check logs");
            }
            Ok(())
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            native::save_local_scores(level_hash, scores)
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
        base_path.join(format!("{group}"))
    }
}

fn decode_group(level_bytes: &[u8], meta: &str) -> Result<(LevelSet, LevelSetInfo)> {
    match (
        cbor4ii::serde::from_slice(level_bytes).with_context(|| "when parsing levels data"),
        toml::from_str(meta).with_context(|| "when parsing meta file"),
    ) {
        (Ok(set), Ok(info)) => Ok((set, info)),
        (Ok(_), Err(err)) | (Err(err), _) => {
            // Try legacy version, for backwards compatibility
            match cbor4ii::serde::from_slice::<ctl_client::core::legacy::v2::LevelSet>(level_bytes)
                .with_context(|| "when parsing levels data")
            {
                Ok(value) => {
                    let (set, mut info) = ctl_client::core::legacy::v2::convert_group(value);
                    info.music = toml::from_str::<ctl_client::core::legacy::v2::GroupMeta>(meta)
                        .with_context(|| "when parsing meta file")?
                        .music
                        .unwrap_or_default()
                        .into();
                    return Ok((set, info));
                }
                Err(err) => log::error!("v2 parse failed: {err:?}"),
            }
            match bincode::deserialize::<ctl_client::core::legacy::v1::LevelSet>(level_bytes)
                .with_context(|| "when parsing levels data")
            {
                Ok(value) => {
                    let beat_time = r32(60.0) / r32(150.0);
                    let (set, info) = ctl_client::core::legacy::v1::convert_group(beat_time, value);
                    return Ok((set, info));
                }
                Err(err) => log::error!("v1 parse failed: {err:?}"),
            }
            Err(err)
        }
    }
}

async fn load_groups_all_assets(geng: &Geng) -> Result<Vec<LocalGroup>> {
    log::debug!("Loading groups from assets");
    let groups_path = run_dir().join("assets").join("levels");

    let list: Vec<String> = file::load_detect(groups_path.join("_list.ron")).await?;
    let paths: Vec<_> = list
        .into_iter()
        .map(|entry| groups_path.join(entry))
        .collect();

    let load_group = |path: PathBuf| async move {
        let context = format!("when loading: {path:?}");

        let result = async move {
            let bytes = file::load_bytes(&path.join("levels.cbor")).await?;
            let meta_str = file::load_string(&path.join("meta.toml")).await?;
            let (group, meta) = decode_group(&bytes, &meta_str)?;

            let music_bytes = file::load_bytes(&path.join("music.mp3")).await;
            let music = match music_bytes {
                Ok(bytes) => {
                    let mut music: geng::Sound = geng.audio().decode(bytes.clone()).await?;
                    music.looped = true;
                    Some((music, bytes))
                }
                Err(_) => None,
            };

            let music_meta = meta.music.clone();
            let music = music
                .map(|(music, bytes)| Rc::new(LocalMusic::new(music_meta, music, bytes.into())));

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
                log::error!("failed to load group: {err:?}");
            }
        }
    }

    Ok(res)
}
