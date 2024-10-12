use super::*;

pub async fn load_music_all(geng: &Geng) -> Result<Vec<CachedMusic>> {
    let music_path = fs::all_music_path();
    if !music_path.exists() {
        return Ok(Vec::new());
    }

    let paths: Vec<_> = std::fs::read_dir(music_path)?
        .flat_map(|entry| {
            let entry = entry?;
            let path = entry.path();
            if !path.is_dir() {
                log::error!("Unexpected file in music dir: {:?}", path);
                return Ok(None);
            }
            anyhow::Ok(Some(path))
        })
        .flatten()
        .collect();
    let music_loaders = paths.into_iter().map(|path| load_music(geng, path));
    let music = future::join_all(music_loaders).await;

    let mut res = Vec::new();
    for music in music {
        match music {
            Ok(music) => res.push(music),
            Err(err) => {
                log::error!("failed to load music: {:?}", err);
            }
        }
    }

    Ok(res)
}

async fn load_music(geng: &Geng, path: PathBuf) -> Result<CachedMusic> {
    let res = async {
        log::debug!("loading music at {:?}", &path);
        let music = CachedMusic::load(geng.asset_manager(), &path).await?;
        Ok(music)
    }
    .await;
    if let Err(err) = &res {
        log::error!("failed to load music: {:?}", err);
    }
    res
}

pub async fn load_groups_all() -> Result<Vec<(PathBuf, LevelSet)>> {
    let groups_path = fs::all_groups_path();
    if !groups_path.exists() {
        return Ok(Vec::new());
    }

    let paths: Vec<_> = std::fs::read_dir(groups_path)?
        .flat_map(|entry| {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                log::error!("Unexpected directory inside levels: {:?}", path);
                return Ok(None);
            }
            anyhow::Ok(Some(path))
        })
        .flatten()
        .collect();

    let load_group = |path| async move {
        let context = format!("when loading {:?}", path);
        async move {
            let bytes = file::load_bytes(&path).await?;
            let group: LevelSet = bincode::deserialize(&bytes)?;
            anyhow::Ok((path, group))
        }
        .await
        .with_context(|| context)
    };

    let group_loaders = paths.into_iter().map(load_group);
    let groups = future::join_all(group_loaders).await;

    let mut res = Vec::new();
    for group in groups {
        match group {
            Ok((path, group)) => res.push((path, group)),
            Err(err) => {
                log::error!("failed to load group: {:?}", err);
            }
        }
    }

    Ok(res)
}

pub fn save_music(id: Id, data: &[u8], info: &MusicInfo) -> Result<()> {
    let path = music_path(id);
    std::fs::create_dir_all(&path)?;

    std::fs::write(path.join("music.mp3"), data)?;
    std::fs::write(path.join("meta.toml"), toml::to_string_pretty(&info)?)?;

    Ok(())
}

pub fn save_group(group: &CachedGroup) -> Result<()> {
    use ron::extensions::Extensions;

    let path = &group.path;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let writer = std::io::BufWriter::new(std::fs::File::create(path)?);
    let extensions = Extensions::UNWRAP_NEWTYPES
        | Extensions::IMPLICIT_SOME
        | Extensions::UNWRAP_VARIANT_NEWTYPES;
    let config = ron::ser::PrettyConfig::new()
        .struct_names(false)
        .separate_tuple_members(true)
        .enumerate_arrays(true)
        .compact_arrays(true)
        .extensions(extensions);
    ron::ser::to_writer_pretty(writer, &group.data, config)?;

    log::debug!("Saved group ({}) successfully", group.data.id);

    Ok(())
}
