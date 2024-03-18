use ctl_client::core::types::MusicInfo;
use geng::prelude::*;

#[derive(geng::asset::Load)]
pub struct MusicAssets {
    #[load(postprocess = "looping", ext = "mp3")]
    pub music: geng::Sound,
    pub meta: MusicInfo,
}

#[derive(geng::asset::Load)]
pub struct Assets {
    pub sprites: Sprites,
    pub dither: DitherAssets,
    pub shaders: Shaders,
    pub fonts: Fonts,
}

#[derive(geng::asset::Load)]
pub struct Fonts {
    pub pixel: geng::Font,
}

#[derive(geng::asset::Load)]
pub struct Sprites {
    #[load(postprocess = "pixel")]
    pub title: ugli::Texture,
    #[load(postprocess = "pixel")]
    pub linear_gradient: ugli::Texture,
    #[load(postprocess = "pixel")]
    pub radial_gradient: ugli::Texture,
    #[load(options(filter = "ugli::Filter::Nearest"))]
    pub button_next: Rc<ugli::Texture>,
    #[load(options(filter = "ugli::Filter::Nearest"))]
    pub button_prev: Rc<ugli::Texture>,
    #[load(options(filter = "ugli::Filter::Nearest"))]
    pub button_close: Rc<ugli::Texture>,
    #[load(options(filter = "ugli::Filter::Nearest"))]
    pub border: ugli::Texture,
    #[load(options(filter = "ugli::Filter::Nearest"))]
    pub border_thin: ugli::Texture,
    #[load(options(filter = "ugli::Filter::Nearest"))]
    pub fill: ugli::Texture,
    #[load(options(filter = "ugli::Filter::Nearest"))]
    pub fill_thin: ugli::Texture,
    #[load(options(filter = "ugli::Filter::Nearest"))]
    pub circle: ugli::Texture,
    #[load(options(filter = "ugli::Filter::Nearest"))]
    pub help: Rc<ugli::Texture>,
    #[load(options(filter = "ugli::Filter::Nearest"))]
    pub reset: Rc<ugli::Texture>,
    #[load(options(filter = "ugli::Filter::Nearest"))]
    pub head: Rc<ugli::Texture>,
    #[load(options(filter = "ugli::Filter::Nearest"))]
    pub edit: Rc<ugli::Texture>,
    #[load(options(filter = "ugli::Filter::Nearest"))]
    pub download: Rc<ugli::Texture>,
}

#[derive(geng::asset::Load)]
pub struct Shaders {
    pub sdf: ugli::Program,
    pub solid: ugli::Program,
    pub light: ugli::Program,
    pub masked: ugli::Program,
    pub texture: ugli::Program,
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
