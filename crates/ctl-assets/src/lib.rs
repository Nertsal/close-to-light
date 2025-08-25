mod options;

pub use self::options::*;

use std::path::PathBuf;

use ctl_core::prelude::{Color, Modifier};
pub use ctl_font::Font;
use ctl_render_core::SubTexture;
use geng::prelude::*;
use geng_utils::gif::GifFrame;

#[derive(geng::asset::Load)]
pub struct LoadingAssets {
    #[load(path = "sprites/title.png", options(filter = "ugli::Filter::Nearest"))]
    pub title: ugli::Texture,
    #[load(path = "fonts/pixel.ttf")]
    pub font: Font,
    #[load(load_with = "load_gif(&manager, &base_path.join(\"sprites/loading_background.gif\"))")]
    pub background: Vec<GifFrame>,
    #[load(path = "shaders/replace_colors.glsl")]
    pub background_shader: ugli::Program,
    #[load(path = "shaders/crt.glsl")]
    pub crt_shader: ugli::Program,
}

fn load_gif(
    manager: &geng::asset::Manager,
    path: &std::path::Path,
) -> geng::asset::Future<Vec<GifFrame>> {
    let manager = manager.clone();
    let path = path.to_owned();
    async move {
        geng_utils::gif::load_gif(
            &manager,
            &path,
            geng_utils::gif::GifOptions {
                frame: geng::asset::TextureOptions {
                    filter: ugli::Filter::Nearest,
                    ..Default::default()
                },
            },
        )
        .await
    }
    .boxed_local()
}

#[derive(geng::asset::Load)]
pub struct Assets {
    pub sounds: Sounds,
    pub sprites: Sprites,
    pub atlas: Rc<SpritesAtlas>,
    pub dither: DitherAssets,
    pub shaders: Shaders,
    pub fonts: Fonts,
}

#[derive(geng::asset::Load)]
pub struct Fonts {
    pub default: Rc<Font>,
    pub pixel: Rc<Font>,
}

#[derive(geng::asset::Load)]
pub struct Sounds {
    pub ui_hover: Rc<geng::Sound>,
    pub ui_click: Rc<geng::Sound>,
}

#[derive(geng::asset::Load)]
pub struct Sprites {
    pub title: PixelTexture,
    pub linear_gradient: PixelTexture,
    pub radial_gradient: PixelTexture,
    pub square_gradient: PixelTexture,
    pub border: PixelTexture,
    pub border_thin: PixelTexture,
    pub border_thinner: PixelTexture,
    pub fill: PixelTexture,
    pub fill_thin: PixelTexture,
    pub circle: PixelTexture,
}

ctl_derive::texture_atlas!(pub SpritesAtlas {
    white,
    title,
    linear_gradient,
    radial_gradient,
    button_next,
    button_next_hollow,
    button_prev,
    button_prev_hollow,
    button_close,
    border,
    border_thin,
    border_thinner,
    fill,
    fill_thin,
    fill_thinner,
    circle,
    help,
    reset,
    head,
    edit,
    download,
    goto,
    trash,
    settings,
    discord,
    star,
    local,
    dotdotdot,
    arrow_up,
    arrow_down,
    arrow_left,
    arrow_right,
    pause,
    confirm,
    discard,
    loading,
    mod_nofail,
    mod_sudden,
    mod_hidden,
    value_knob,
    dropdown,

    timeline: {
        arrow,
        current_arrow,
        dots,
        waypoint,
        time_mark,

        circle,
        circle_fill,
        square,
        square_fill,
        triangle,
        triangle_fill,

        tick_big,
        tick_mid,
        tick_smol,
        tick_tiny
    }
});

#[derive(geng::asset::Load)]
pub struct Shaders {
    pub solid: ugli::Program,
    pub light: ugli::Program,
    pub masked: ugli::Program,
    pub texture: ugli::Program,
    pub ellipse: ugli::Program,
    pub texture_ui: ugli::Program,
    pub crt: ugli::Program,
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

impl Assets {
    pub async fn load(manager: &geng::asset::Manager) -> anyhow::Result<Self> {
        geng::asset::Load::load(manager, &run_dir().join("assets"), &())
            .await
            .context("failed to load assets")
    }

    pub fn get_modifier(&self, modifier: Modifier) -> SubTexture {
        match modifier {
            Modifier::NoFail => self.atlas.mod_nofail(),
            Modifier::Sudden => self.atlas.mod_sudden(),
            Modifier::Hidden => self.atlas.mod_hidden(),
        }
    }
}
