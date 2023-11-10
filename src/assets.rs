use geng::prelude::*;

/// Workaround until <https://github.com/geng-engine/geng/issues/78> is fixed.
#[derive(geng::asset::Load)]
pub struct MusicAssets {
    #[load(postprocess = "looping")]
    pub music: geng::Sound,
}

#[derive(geng::asset::Load)]
pub struct Assets {
    pub sprites: Sprites,
    pub dither: DitherAssets,
    pub shaders: Shaders,
}

#[derive(geng::asset::Load)]
pub struct Sprites {
    #[load(postprocess = "pixel")]
    pub title: ugli::Texture,
    #[load(postprocess = "pixel")]
    pub linear_gradient: ugli::Texture,
    #[load(postprocess = "pixel")]
    pub radial_gradient: ugli::Texture,
}

#[derive(geng::asset::Load)]
pub struct Shaders {
    pub sdf: ugli::Program,
    pub solid: ugli::Program,
    pub light: ugli::Program,
}

#[derive(geng::asset::Load)]
pub struct DitherAssets {
    /// Dither postprocess shader
    pub dither_shader: ugli::Program,
    #[load(postprocess = "dither_pattern")]
    pub dither1: ugli::Texture,
    #[load(postprocess = "dither_pattern")]
    pub dither2: ugli::Texture,
    #[load(postprocess = "dither_pattern")]
    pub dither3: ugli::Texture,
}

fn dither_pattern(texture: &mut ugli::Texture) {
    texture.set_wrap_mode(ugli::WrapMode::Repeat);
    texture.set_filter(ugli::Filter::Nearest);
}

fn looping(sfx: &mut geng::Sound) {
    sfx.set_looped(true)
}

fn pixel(texture: &mut ugli::Texture) {
    texture.set_filter(ugli::Filter::Nearest);
}

impl Assets {
    pub async fn load(manager: &geng::asset::Manager) -> anyhow::Result<Self> {
        geng::asset::Load::load(manager, &run_dir().join("assets"), &())
            .await
            .context("failed to load assets")
    }
}
