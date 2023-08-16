use geng::prelude::*;

#[derive(geng::asset::Load)]
pub struct Assets {
    #[load(postprocess = "looping", ext = "mp3")]
    pub music: geng::Sound,

    // Dither postprocess shader
    #[load(ext = "glsl")]
    pub dither_shader: ugli::Program,
    #[load(postprocess = "dither_pattern", ext = "png")]
    pub dither1: ugli::Texture,
    #[load(postprocess = "dither_pattern", ext = "png")]
    pub dither2: ugli::Texture,
    #[load(postprocess = "dither_pattern", ext = "png")]
    pub dither3: ugli::Texture,
}

/// Use in Assets as `#[load(postprocess = "dither_pattern")]`
fn dither_pattern(texture: &mut ugli::Texture) {
    texture.set_wrap_mode(ugli::WrapMode::Repeat);
    texture.set_filter(ugli::Filter::Nearest);
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
