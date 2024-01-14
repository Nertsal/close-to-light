mod level;
mod main;
mod splash;

pub use self::{level::*, main::*, splash::*};

use crate::{
    prelude::*,
    render::{
        dither::DitherRender,
        util::{TextRenderOptions, UtilRender},
    },
    OPTIONS_STORAGE, PLAYER_NAME_STORAGE,
};

pub async fn load_groups(
    manager: &geng::asset::Manager,
    groups_path: impl AsRef<std::path::Path>,
) -> anyhow::Result<Vec<GroupEntry>> {
    let groups_path = groups_path.as_ref();

    let group_names: Vec<String> = file::load_detect(groups_path.join("_list.ron"))
        .await
        .context("when loading list of groups")?;

    let mut groups = Vec::new();
    for name in group_names {
        let group_path = groups_path.join(name);

        let meta: GroupMeta = file::load_detect(group_path.join("meta.toml"))
            .await
            .context(format!("when loading group meta for {:?}", group_path))?;

        let logo: Option<ugli::Texture> = geng::asset::Load::load(
            manager,
            &group_path.join("logo.png"),
            &geng::asset::TextureOptions {
                filter: ugli::Filter::Nearest,
                ..default()
            },
        )
        .await
        .ok();

        let levels_path = group_path.join("levels");
        let levels_list: Vec<String> = file::load_detect(levels_path.join("_list.ron"))
            .await
            .context("when loading list of levels")?;
        let mut levels = Vec::new();
        for name in levels_list {
            let level_path = levels_path.join(name);
            let meta: LevelMeta = file::load_detect(level_path.join("meta.toml"))
                .await
                .context(format!("when loading level meta for {:?}", level_path))?;
            levels.push((level_path, meta));
        }

        groups.push(GroupEntry { meta, levels, logo });
    }

    Ok(groups)
}

pub async fn load_level(
    manager: &geng::asset::Manager,
    level_path: impl AsRef<std::path::Path>,
) -> anyhow::Result<(GroupMeta, LevelMeta, Music, Level)> {
    let level_path = level_path.as_ref();
    log::debug!("loading level at {:?}", level_path);

    let group_path = level_path
        .parent()
        .expect("level has to be in a folder")
        .parent()
        .expect("level has to be in a group");

    let group_meta: GroupMeta = file::load_detect(group_path.join("meta.toml"))
        .await
        .context(format!("when loading group meta at {:?}", group_path))?;

    let level_meta: LevelMeta = file::load_detect(level_path.join("meta.toml"))
        .await
        .context(format!("when loading level meta at {:?}", level_path))?;

    let level_path = &level_path.join("level.json");
    let level: Level = geng::asset::Load::load(manager, level_path, &())
        .await
        .context(format!("when loading level at {:?}", level_path))?;

    let music: crate::assets::MusicAssets = geng::asset::Load::load(manager, group_path, &())
        .await
        .context(format!("when loading music for level at {:?}", group_path))?;
    let music = Music::new(Rc::new(music.music), group_meta.music.clone());

    Ok((group_meta, level_meta, music, level))
}
