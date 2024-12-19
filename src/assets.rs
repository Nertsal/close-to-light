use std::path::PathBuf;

use ctl_client::core::types::MusicInfo;
use geng::prelude::*;

use crate::prelude::Modifier;

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
    pub title: PixelTexture,
    pub linear_gradient: PixelTexture,
    pub radial_gradient: PixelTexture,
    pub button_next: PixelTexture,
    pub button_next_hollow: PixelTexture,
    pub button_prev: PixelTexture,
    pub button_prev_hollow: PixelTexture,
    pub button_close: PixelTexture,
    pub border: PixelTexture,
    pub border_thin: PixelTexture,
    pub border_thinner: PixelTexture,
    pub fill: PixelTexture,
    pub fill_thin: PixelTexture,
    pub circle: PixelTexture,
    pub help: PixelTexture,
    pub reset: PixelTexture,
    pub head: PixelTexture,
    pub edit: PixelTexture,
    pub download: PixelTexture,
    pub goto: PixelTexture,
    pub trash: PixelTexture,
    pub settings: PixelTexture,
    pub discord: PixelTexture,
    pub star: PixelTexture,
    pub local: PixelTexture,
    pub dotdotdot: PixelTexture,
    pub arrow_up: PixelTexture,
    pub arrow_down: PixelTexture,
    pub arrow_left: PixelTexture,
    pub arrow_right: PixelTexture,
    pub pause: PixelTexture,
    pub confirm: PixelTexture,
    pub discard: PixelTexture,
    pub loading: PixelTexture,
    pub mod_nofail: PixelTexture,
    pub mod_sudden: PixelTexture,
    pub mod_hidden: PixelTexture,
    pub value_knob: PixelTexture,
    pub dropdown: PixelTexture,
    pub timeline: TimelineAssets,
}

#[derive(geng::asset::Load)]
pub struct TimelineAssets {
    pub arrow: PixelTexture,
    pub current_arrow: PixelTexture,
    pub dots: PixelTexture,
    pub waypoint: PixelTexture,

    pub circle: PixelTexture,
    pub circle_fill: PixelTexture,
    pub square: PixelTexture,
    pub square_fill: PixelTexture,
    pub triangle: PixelTexture,
    pub triangle_fill: PixelTexture,

    pub tick_big: PixelTexture,
    pub tick_mid: PixelTexture,
    pub tick_smol: PixelTexture,
    pub tick_tiny: PixelTexture,
}

#[derive(geng::asset::Load)]
pub struct Shaders {
    pub sdf: ugli::Program,
    pub solid: ugli::Program,
    pub light: ugli::Program,
    pub masked: ugli::Program,
    pub texture: ugli::Program,
    pub ellipse: ugli::Program,
    pub solid_ui: ugli::Program,
    pub texture_ui: ugli::Program,
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

#[derive(Clone)]
pub struct PixelTexture {
    pub path: PathBuf,
    pub texture: Rc<ugli::Texture>,
}

impl Deref for PixelTexture {
    type Target = ugli::Texture;

    fn deref(&self) -> &Self::Target {
        &self.texture
    }
}

impl Debug for PixelTexture {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PixelTexture")
            .field("path", &self.path)
            .field("texture", &"<texture data>")
            .finish()
    }
}

impl geng::asset::Load for PixelTexture {
    type Options = <ugli::Texture as geng::asset::Load>::Options;

    fn load(
        manager: &geng::asset::Manager,
        path: &std::path::Path,
        options: &Self::Options,
    ) -> geng::asset::Future<Self> {
        let path = path.to_owned();
        let texture = ugli::Texture::load(manager, &path, options);
        async move {
            let mut texture = texture.await?;
            texture.set_filter(ugli::Filter::Nearest);
            Ok(Self {
                path,
                texture: Rc::new(texture),
            })
        }
        .boxed_local()
    }

    const DEFAULT_EXT: Option<&'static str> = Some("png");
}

fn dither_pattern(texture: &mut ugli::Texture) {
    texture.set_wrap_mode(ugli::WrapMode::Repeat);
    texture.set_filter(ugli::Filter::Nearest);
}

fn looping(sfx: &mut geng::Sound) {
    sfx.looped = true;
}

impl Assets {
    pub async fn load(manager: &geng::asset::Manager) -> anyhow::Result<Self> {
        geng::asset::Load::load(manager, &run_dir().join("assets"), &())
            .await
            .context("failed to load assets")
    }

    pub fn get_modifier(&self, modifier: Modifier) -> &PixelTexture {
        match modifier {
            Modifier::NoFail => &self.sprites.mod_nofail,
            Modifier::Sudden => &self.sprites.mod_sudden,
            Modifier::Hidden => &self.sprites.mod_hidden,
        }
    }
}
