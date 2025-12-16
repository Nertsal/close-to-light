pub mod trailer;

use crate::{
    prelude::*,
    render::{
        dither::DitherRender,
        post::{PostRender, PostVfx},
        util::UtilRender,
    },
};

use ctl_render_core::TextRenderOptions;

pub struct MediaState {
    context: Context,
    util_render: UtilRender,
    dither: DitherRender,
    post: PostRender,

    theme: Theme,
    time: FloatTime,
    camera: Camera2d,

    text: String,
    picture: Option<ugli::Texture>,
}

impl MediaState {
    pub fn new(context: Context) -> Self {
        Self {
            util_render: UtilRender::new(context.clone()),
            dither: DitherRender::new(&context.geng, &context.assets),
            post: PostRender::new(context.clone()),

            theme: Theme::default(),
            time: FloatTime::ZERO,
            camera: Camera2d {
                center: vec2::ZERO,
                rotation: Angle::ZERO,
                fov: Camera2dFov::Vertical(10.0),
            },

            text: String::new(),
            picture: None,

            context,
        }
    }

    pub fn set_text(&mut self, text: impl Into<String>) {
        self.text = text.into();
    }

    pub fn set_picture(&mut self, picture: ugli::Texture) {
        self.picture = Some(picture);
    }
}

impl geng::State for MediaState {
    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        ugli::clear(framebuffer, Some(self.theme.dark), None, None);

        let mut dither_buffer = self.dither.start();

        self.util_render.draw_text(
            &self.text,
            vec2(0.0, 0.0),
            TextRenderOptions::new(1.0).color(crate::render::THEME.light),
            &self.camera,
            &mut dither_buffer,
        );

        self.dither.finish(self.time, &self.theme);

        let buffer = &mut self.post.begin(framebuffer.size(), self.theme.dark);
        geng_utils::texture::DrawTexture::new(self.dither.get_buffer())
            .fit_screen(vec2(0.5, 0.5), buffer)
            .draw(&geng::PixelPerfectCamera, &self.context.geng, buffer);
        if let Some(picture) = &self.picture {
            let pixel_scale =
                (buffer.size().as_f32() / picture.size().as_f32()).map(|x| x.floor().max(1.0));
            let pixel_scale = pixel_scale.x.min(pixel_scale.y);
            geng_utils::texture::DrawTexture::new(picture)
                .pixel_perfect(
                    buffer.size().as_f32() / 2.0,
                    vec2(0.5, 0.5),
                    pixel_scale,
                    &geng::PixelPerfectCamera,
                    buffer,
                )
                .draw(&geng::PixelPerfectCamera, &self.context.geng, buffer);
        }
        self.post.post_process(
            PostVfx {
                time: self.time,
                crt: true,
                rgb_split: 0.0,
                saturation: 1.0,
            },
            framebuffer,
        );
    }

    fn update(&mut self, delta_time: f64) {
        let delta_time = FloatTime::new(delta_time as f32);
        self.time += delta_time;
    }
}
