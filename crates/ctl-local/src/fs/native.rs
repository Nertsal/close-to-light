use super::*;

use zip::{ZipArchive, result::ZipError};

pub async fn load_groups_all(geng: &Geng) -> Result<Vec<LocalGroup>> {
    let mut groups = load_groups_from(geng, &fs::all_groups_path()).await?;
    if cfg!(debug_assertions) || cfg!(feature = "dev") {
        // Demo levels
        groups.extend(
            load_groups_from(
                geng,
                &run_dir()
                    .join("..")
                    .join("close-to-assets")
                    .join("assets-demo")
                    .join("levels"),
            )
            .await?,
        );
        // Full release levels
        groups.extend(
            load_groups_from(
                geng,
                &run_dir()
                    .join("..")
                    .join("close-to-assets")
                    .join("assets-release")
                    .join("levels"),
            )
            .await?,
        );
    }
    Ok(groups)
}

enum LevelFormat {
    Folder,
    Zip,
}

impl LevelFormat {
    pub fn detect(path: &Path) -> Option<LevelFormat> {
        if path.is_dir() {
            Some(Self::Folder)
        } else if path.extension().and_then(|e| e.to_str()) == Some("ctz") {
            Some(Self::Zip)
        } else {
            None
        }
    }
}

async fn load_groups_from(geng: &Geng, groups_path: &PathBuf) -> Result<Vec<LocalGroup>> {
    log::debug!("Looking for levels in {:?}", groups_path);
    if !groups_path.exists() {
        return Ok(Vec::new());
    }

    let paths: Vec<_> = std::fs::read_dir(groups_path)?
        .flat_map(|entry| {
            let entry = entry?;
            let path = entry.path();
            if LevelFormat::detect(&path).is_none() {
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
            match LevelFormat::detect(&path) {
                Some(LevelFormat::Folder) => load_group_from_dir(geng, path).await,
                Some(LevelFormat::Zip) => load_group_from_zip(geng, path).await,
                None => Err(anyhow::anyhow!("Unrecognized level format at: {path:?}")),
            }
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

async fn load_group_from_dir(geng: &Geng, path: PathBuf) -> Result<LocalGroup> {
    let extract_file = |name: &str| -> Result<Vec<u8>> {
        let file = std::fs::File::open(path.join(name))
            .with_context(|| format!("when looking for {:?}", name))?;
        let mut buf = Vec::new();
        let mut reader = std::io::BufReader::new(file);
        reader
            .read_to_end(&mut buf)
            .with_context(|| format!("when reading file {:?}", name))?;
        Ok(buf)
    };
    load_group_with(geng, path.clone(), extract_file).await
}

async fn load_group_from_zip(geng: &Geng, path: PathBuf) -> Result<LocalGroup> {
    let mut archive = std::fs::File::open(&path)
        .map_err(ZipError::from)
        .and_then(ZipArchive::new)?;

    let archive_name = path.file_stem().and_then(|n| n.to_str()).map(String::from);
    let extract_file = |name: &str| -> Result<Vec<u8>> {
        let by_name = if let Some(archive) = &archive_name {
            format!("{}/{}", archive, name)
        } else {
            name.to_owned()
        };
        let file = archive
            .by_name(&by_name)
            .with_context(|| format!("when looking for {:?}", name))?;
        if !file.is_file() {
            return Err(anyhow::anyhow!("expected {:?} to be a file", name));
        }
        let mut buf = Vec::with_capacity(file.size() as usize);
        let mut reader = std::io::BufReader::new(file);
        reader
            .read_to_end(&mut buf)
            .with_context(|| format!("when reading file {:?}", name))?;
        Ok(buf)
    };
    load_group_with(geng, path, extract_file).await
}

async fn load_group_with(
    geng: &Geng,
    path: PathBuf,
    mut extract_file: impl FnMut(&str) -> Result<Vec<u8>>,
) -> Result<LocalGroup> {
    let bytes = extract_file("levels.cbor")?;
    let meta_bytes = extract_file("meta.toml")?;
    let meta_str = String::from_utf8_lossy(&meta_bytes);
    let (group, meta) = decode_group(&bytes, &meta_str).with_context(|| "when deserializing")?;

    let music_bytes = extract_file("music.mp3");
    let music = match music_bytes {
        Ok(bytes) => {
            let music: geng::Sound = geng.audio().decode(bytes.clone()).await?;
            Some((music, bytes))
        }
        Err(_) => None,
    };

    let music_meta = meta.music.clone();
    let music =
        music.map(|(music, bytes)| Rc::new(LocalMusic::new(music_meta, music, bytes.into())));

    let local = LocalGroup {
        path,
        loaded_from_assets: false,
        meta,
        music,
        data: group,
    };

    anyhow::Ok(local)
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

pub fn load_local_highscores() -> Result<HashMap<LocalLevelId, SavedScore>> {
    let dir_path = base_path().join("scores");
    let mut res = HashMap::new();
    for entry in std::fs::read_dir(dir_path)? {
        let process = || -> Result<()> {
            let entry = entry?;
            if entry.metadata()?.is_file() {
                let filename = entry
                    .file_name()
                    .into_string()
                    .map_err(|_| anyhow!("encountered non-unicode file name"))?;
                let reader = std::io::BufReader::new(std::fs::File::open(entry.path())?);
                let scores: Vec<SavedScore> = cbor4ii::serde::from_reader(reader)?;
                if let Some(score) = scores.into_iter().max_by_key(|score| score.score) {
                    let id = LocalLevelId::convert_from_str(&filename);
                    res.insert(id, score);
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

pub fn load_local_scores(level_id: &LocalLevelId) -> Result<Vec<SavedScore>> {
    let path = local_scores_path(level_id);
    let reader = std::io::BufReader::new(std::fs::File::open(path)?);
    let scores = cbor4ii::serde::from_reader(reader)?;
    Ok(scores)
}

pub fn save_local_scores(level_id: &LocalLevelId, scores: &[SavedScore]) -> Result<()> {
    let path = local_scores_path(level_id);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let writer = std::io::BufWriter::new(std::fs::File::create(path)?);
    cbor4ii::serde::to_writer(writer, &scores)?;
    Ok(())
}

fn local_scores_path(level_id: &LocalLevelId) -> PathBuf {
    let scores = base_path().join("scores");
    match &level_id {
        LocalLevelId::Hash(hash) => scores.join(hash),
        LocalLevelId::Id(id) => scores.join(format!("{}", id)),
    }
}
