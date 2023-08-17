mod game;
mod util;

pub use self::{game::*, util::*};

use crate::{assets::Assets, model::*};

use geng::prelude::{ugli::Texture, *};
use geng_utils::{
    conversions::Vec2RealConversions, geometry::unit_quad_geometry, texture::draw_texture_fit,
};

pub const COLOR_PLAYER: Rgba<f32> = Rgba::WHITE;
pub const COLOR_LIGHT: Rgba<f32> = Rgba::WHITE;
pub const COLOR_DARK: Rgba<f32> = Rgba::BLACK;

pub struct Render {
    geng: Geng,
    assets: Rc<Assets>,
    double_buffer: (Texture, Texture),
}

impl Render {
    pub fn new(geng: &Geng, assets: &Rc<Assets>) -> Self {
        Self {
            geng: geng.clone(),
            assets: assets.clone(),
            double_buffer: {
                let height = 360;
                let size = vec2(height * 16 / 9, height);

                let mut first = geng_utils::texture::new_texture(geng.ugli(), size);
                first.set_filter(ugli::Filter::Nearest);
                let mut second = geng_utils::texture::new_texture(geng.ugli(), size);
                second.set_filter(ugli::Filter::Nearest);

                (first, second)
            },
        }
    }

    fn swap_buffer(&mut self) {
        std::mem::swap(&mut self.double_buffer.0, &mut self.double_buffer.1);
    }

    pub fn get_render_size(&self) -> vec2<usize> {
        self.double_buffer.0.size()
    }

    pub fn get_buffer(&self) -> &ugli::Texture2d<Rgba<f32>> {
        &self.double_buffer.0
    }

    pub fn start(&mut self) -> ugli::Framebuffer {
        let mut framebuffer =
            geng_utils::texture::attach_texture(&mut self.double_buffer.0, self.geng.ugli());
        ugli::clear(&mut framebuffer, Some(Rgba::BLACK), None, None);
        framebuffer
    }

    pub fn dither(&mut self, time: Time, bg_noise: R32) -> ugli::Framebuffer {
        let mut other_framebuffer =
            geng_utils::texture::attach_texture(&mut self.double_buffer.1, self.geng.ugli());

        let t = time.as_f32();
        let t = (t.fract() - 0.5).abs();

        ugli::draw(
            &mut other_framebuffer,
            &self.assets.dither_shader,
            ugli::DrawMode::TriangleFan,
            &unit_quad_geometry(self.geng.ugli()),
            ugli::uniforms!(
                u_time: t,
                u_bg_noise: bg_noise.as_f32(),
                u_framebuffer_size: self.double_buffer.0.size().as_f32(),
                u_pattern_size: self.assets.dither1.size().as_f32(),
                u_texture: &self.double_buffer.0,
                u_dither1: &self.assets.dither1,
                u_dither2: &self.assets.dither2,
                u_dither3: &self.assets.dither3,
            ),
            ugli::DrawParameters {
                blend_mode: Some(ugli::BlendMode::straight_alpha()),
                ..Default::default()
            },
        );
        self.swap_buffer();
        geng_utils::texture::attach_texture(&mut self.double_buffer.0, self.geng.ugli())
    }
}
