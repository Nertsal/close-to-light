mod level;
mod main;

pub use self::{level::*, main::*};

use crate::{
    prelude::*,
    render::{
        dither::DitherRender,
        util::{TextRenderOptions, UtilRender},
    },
};

const PLAYER_NAME_STORAGE: &str = "close-to-light-name";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LevelId {
    pub group: std::path::PathBuf,
    pub level: LevelVariation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LevelVariation {
    Normal,
    Hard,
}

impl LevelId {
    pub fn get_path(&self) -> std::path::PathBuf {
        // let levels_path = run_dir().join("assets").join("levels");
        // let group_path = levels_path.join(&self.group);
        let group_path = &self.group;

        let level_path = match self.level {
            LevelVariation::Normal => "normal",
            LevelVariation::Hard => "hard",
        };
        group_path.join(format!("level_{}.json", level_path))
    }
}

pub async fn load_groups(
    manager: &geng::asset::Manager,
    levels_path: impl AsRef<std::path::Path>,
) -> anyhow::Result<Vec<GroupEntry>> {
    let levels_path = levels_path.as_ref();

    let group_names: Vec<String> = file::load_detect(levels_path.join("_list.ron"))
        .await
        .context("when loading list of levels")?;

    let mut groups = Vec::new();
    for name in group_names {
        let path = levels_path.join(name);

        let meta: GroupMeta = file::load_detect(path.join("meta.toml"))
            .await
            .context(format!("when loading group meta for {:?}", path))?;

        let logo: Option<ugli::Texture> = geng::asset::Load::load(
            manager,
            &path.join("logo.png"),
            &geng::asset::TextureOptions {
                filter: ugli::Filter::Nearest,
                ..default()
            },
        )
        .await
        .ok();

        groups.push(GroupEntry { meta, path, logo });
    }

    Ok(groups)
}

pub async fn load_level(
    manager: &geng::asset::Manager,
    level_path: impl AsRef<std::path::Path>,
) -> anyhow::Result<(GroupMeta, Music, Level)> {
    let level_path = level_path.as_ref();
    let group_path = level_path.parent().expect("level has to be in a folder");

    let meta: GroupMeta = file::load_detect(group_path.join("meta.toml"))
        .await
        .context(format!("when loading level meta at {:?}", level_path))?;

    let level: Level = geng::asset::Load::load(manager, level_path, &())
        .await
        .context(format!("when loading level at {:?}", level_path))?;

    let music: crate::assets::MusicAssets = geng::asset::Load::load(manager, group_path, &())
        .await
        .context(format!("when loading music for level at {:?}", group_path))?;
    let music = Music::new(Rc::new(music.music), meta.music.clone());

    Ok((meta, music, level))
}
