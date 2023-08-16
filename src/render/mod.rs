mod util;

pub use util::*;

use crate::{assets::Assets, model::*};

use geng::prelude::{ugli::Texture, *};
use geng_utils::{
    conversions::Vec2RealConversions, geometry::unit_quad_geometry, texture::draw_texture_fit,
};

pub const COLOR_LIGHT: Rgba<f32> = Rgba::WHITE;
pub const COLOR_DARK: Rgba<f32> = Rgba::BLACK;

#[allow(dead_code)]
pub struct GameRender {
    geng: Geng,
    assets: Rc<Assets>,
    util: UtilRender,
    double_buffer: (Texture, Texture),
}

impl GameRender {
    pub fn new(geng: &Geng, assets: &Rc<Assets>) -> Self {
        Self {
            geng: geng.clone(),
            assets: assets.clone(),
            util: UtilRender::new(geng, assets),
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

    pub fn swap_buffer(&mut self) {
        std::mem::swap(&mut self.double_buffer.0, &mut self.double_buffer.1);
    }

    pub fn get_render_size(&mut self) -> vec2<usize> {
        self.double_buffer.0.size()
    }

    pub fn draw_world(&mut self, model: &Model, framebuffer: &mut ugli::Framebuffer) {
        let (old_framebuffer, mut framebuffer) = (
            framebuffer,
            geng_utils::texture::attach_texture(&mut self.double_buffer.0, self.geng.ugli()),
        );
        ugli::clear(&mut framebuffer, Some(Rgba::BLACK), None, None);

        let camera = &model.camera;

        // Telegraphs
        for tele in &model.telegraphs {
            self.util
                .draw_outline(&tele.light.collider, 0.025, 1.0, camera, &mut framebuffer);
        }

        // Lights
        for light in &model.lights {
            self.util
                .draw_collider(&light.collider, 1.0, camera, &mut framebuffer);
        }

        // Player
        let mut player = model.player.collider.clone();
        player.position += model.player.shake;
        self.util
            .draw_collider(&player, 1.0, camera, &mut framebuffer);

        let mut other_framebuffer =
            geng_utils::texture::attach_texture(&mut self.double_buffer.1, self.geng.ugli());

        ugli::draw(
            &mut other_framebuffer,
            &self.assets.dither_shader,
            ugli::DrawMode::TriangleFan,
            &unit_quad_geometry(self.geng.ugli()),
            ugli::uniforms!(
                u_framebuffer_size: framebuffer.size().as_f32(),
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

        let aabb = Aabb2::ZERO.extend_positive(old_framebuffer.size().as_f32());
        draw_texture_fit(
            &self.double_buffer.0,
            aabb,
            vec2(0.5, 0.5),
            &geng::PixelPerfectCamera,
            &self.geng,
            old_framebuffer,
        );
    }

    pub fn draw_ui(&mut self, model: &Model, framebuffer: &mut ugli::Framebuffer) {
        let camera = &geng::PixelPerfectCamera;
        let screen = Aabb2::ZERO.extend_positive(framebuffer.size().as_f32());

        let font_size = screen.height() * 0.05;

        // Fear meter
        let fear = Aabb2::point(
            geng_utils::layout::aabb_pos(screen, vec2(0.5, 0.0)) + vec2(0.0, 1.0) * font_size,
        )
        .extend_symmetric(vec2(14.0, 0.0) * font_size / 2.0)
        .extend_up(font_size);
        self.geng.draw2d().draw2d(
            framebuffer,
            camera,
            &draw2d::Quad::new(fear.extend_uniform(font_size * 0.1), COLOR_LIGHT),
        );
        self.geng
            .draw2d()
            .draw2d(framebuffer, camera, &draw2d::Quad::new(fear, COLOR_DARK));
        self.geng.draw2d().draw2d(
            framebuffer,
            camera,
            &draw2d::Quad::new(
                fear.extend_symmetric(
                    vec2(
                        -model.player.fear_meter.get_ratio().as_f32() * fear.width(),
                        0.0,
                    ) / 2.0,
                ),
                COLOR_LIGHT,
            ),
        );
    }
}
