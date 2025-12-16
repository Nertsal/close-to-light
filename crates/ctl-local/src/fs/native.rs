use super::*;

pub async fn load_groups_all(geng: &Geng) -> Result<Vec<LocalGroup>> {
    let groups_path = fs::all_groups_path();
    if !groups_path.exists() {
        return Ok(Vec::new());
    }

    let paths: Vec<_> = std::fs::read_dir(groups_path)?
        .flat_map(|entry| {
            let entry = entry?;
            let path = entry.path();
            if !path.is_dir() {
                log::warn!("Unexpected file inside levels: {path:?}");
                return Ok(None);
            }
            anyhow::Ok(Some(path))
        })
        .flatten()
        .collect();

    let load_group = |path: PathBuf| async move {
        let context = format!("when loading {path:?}");
        async move {
            let bytes = file::load_bytes(&path.join("levels.cbor"))
                .await
                .with_context(|| "when loading file")?;
            let meta_str = file::load_string(&path.join("meta.toml"))
                .await
                .with_context(|| "when loading file")?;
            let (group, meta) =
                decode_group(&bytes, &meta_str).with_context(|| "when deserializing")?;

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
                loaded_from_assets: false,
                meta,
                music,
                data: group,
            };

            anyhow::Ok(local)
        }
        .await
        .with_context(|| context)
    };

    let group_loaders = paths.into_iter().map(load_group);
    let groups = future::join_all(group_loaders).await;

    let mut res = Vec::new();
    for group in groups {
        match group {
            Ok(local) => res.push(local),
            Err(err) => {
                log::error!("failed to load group: {err:?}");
            }
        }
    }

    Ok(res)
}

pub fn save_group(group: &CachedGroup, save_music: bool) -> Result<()> {
    let path = &group.local.path;
    std::fs::create_dir_all(path)?;

    // Save levels
    let writer = std::io::BufWriter::new(std::fs::File::create(path.join("levels.cbor"))?);
    cbor4ii::serde::to_writer(
        writer,
        &ctl_core::legacy::VersionedLevelSet::latest(group.local.data.clone()),
    )?;

    // Save meta
    let mut writer = std::io::BufWriter::new(std::fs::File::create(path.join("meta.toml"))?);
    let s = toml::ser::to_string_pretty(&ctl_core::legacy::VersionedLevelSetInfo::latest(
        group.local.meta.clone(),
    ))?;
    write!(writer, "{s}")?;

    // Save music
    if save_music && let Some(music) = &group.local.music {
        std::fs::write(path.join("music.mp3"), &music.bytes)?;
    }

    log::debug!("Saved group ({}) successfully", group.local.meta.id);

    Ok(())
}

pub fn load_local_highscores() -> Result<HashMap<String, SavedScore>> {
    let dir_path = base_path().join("scores");
    let mut res = HashMap::new();
    for entry in std::fs::read_dir(dir_path)? {
        let process = || -> Result<()> {
            let entry = entry?;
            if entry.metadata()?.is_file() {
                let hash = entry
                    .file_name()
                    .into_string()
                    .map_err(|_| anyhow!("encountered non-unicode file name"))?;
                let reader = std::io::BufReader::new(std::fs::File::open(entry.path())?);
                let scores: Vec<SavedScore> = cbor4ii::serde::from_reader(reader)?;
                if let Some(score) = scores.into_iter().max_by_key(|score| score.score) {
                    res.insert(hash, score);
                }
            }
            Ok(())
        };
        if let Err(err) = process() {
            log::error!("score file error: {err:?}");
        }
    }
    Ok(res)
}

pub fn load_local_scores(level_hash: &str) -> Result<Vec<SavedScore>> {
    let path = local_scores_path(level_hash);
    let reader = std::io::BufReader::new(std::fs::File::open(path)?);
    let scores = cbor4ii::serde::from_reader(reader)?;
    Ok(scores)
}

pub fn save_local_scores(level_hash: &str, scores: &[SavedScore]) -> Result<()> {
    let path = local_scores_path(level_hash);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let writer = std::io::BufWriter::new(std::fs::File::create(path)?);
    cbor4ii::serde::to_writer(writer, &scores)?;
    Ok(())
}

fn local_scores_path(level_hash: &str) -> PathBuf {
    base_path().join("scores").join(level_hash)
}
