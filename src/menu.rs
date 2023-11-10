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
