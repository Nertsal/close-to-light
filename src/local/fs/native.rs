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
                log::warn!("Unexpected file inside levels: {:?}", path);
                return Ok(None);
            }
            anyhow::Ok(Some(path))
        })
        .flatten()
        .collect();

    let asset_manager = geng.asset_manager();
    let load_group = |path: PathBuf| async move {
        let context = format!("when loading {:?}", path);
        async move {
            let bytes = file::load_bytes(&path.join("levels.cbor"))
                .await
                .with_context(|| "when loading file")?;
            let group: LevelSet = decode_group(&bytes).with_context(|| "when deserializing")?;

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
                log::error!("failed to load group: {:?}", err);
            }
        }
    }

    Ok(res)
}

pub fn save_group(group: &CachedGroup) -> Result<()> {
    let path = &group.local.path;
    std::fs::create_dir_all(path)?;

    // Save levels
    let writer = std::io::BufWriter::new(std::fs::File::create(path.join("levels.cbor"))?);
    cbor4ii::serde::to_writer(writer, &group.local.data)?;

    // Save meta
    let mut writer = std::io::BufWriter::new(std::fs::File::create(path.join("meta.toml"))?);
    let s = toml::ser::to_string_pretty(&group.local.meta)?;
    write!(writer, "{}", s)?;

    log::debug!("Saved group ({}) successfully", group.local.data.id);

    Ok(())
}
