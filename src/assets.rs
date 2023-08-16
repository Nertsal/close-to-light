use geng::prelude::*;

#[derive(geng::asset::Load)]
pub struct Assets {
    #[load(postprocess = "looping", ext = "mp3")]
    pub music: geng::Sound,
    #[load(ext = "glsl")]
    pub dither_shader: ugli::Program,
}

/// Use in Assets as `#[load(postprocess = "looping")]`
fn looping(sfx: &mut geng::Sound) {
    sfx.set_looped(true)
}

/// Use in Assets as `#[load(postprocess = "pixel")]`
#[allow(dead_code)]
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
