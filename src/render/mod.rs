mod util;

pub use util::*;

use crate::{assets::Assets, model::*};

use geng::prelude::{ugli::Texture, *};
use geng_utils::{conversions::Vec2RealConversions, texture::draw_texture_fit};

pub const COLOR_LIGHT: Rgba<f32> = Rgba::WHITE;
pub const COLOR_DARK: Rgba<f32> = Rgba::BLACK;

#[allow(dead_code)]
pub struct GameRender {
    geng: Geng,
    assets: Rc<Assets>,
    util: UtilRender,
    pub texture: Texture,
}

impl GameRender {
    pub fn new(geng: &Geng, assets: &Rc<Assets>) -> Self {
        let mut texture = geng_utils::texture::new_texture(geng.ugli(), vec2(360 * 16 / 9, 360));
        texture.set_filter(ugli::Filter::Nearest);
        Self {
            geng: geng.clone(),
            assets: assets.clone(),
            util: UtilRender::new(geng, assets),
            texture,
        }
    }

    pub fn draw_world(&mut self, model: &Model, framebuffer: &mut ugli::Framebuffer) {
        let (mut old_framebuffer, mut framebuffer) = (
            framebuffer,
            geng_utils::texture::attach_texture(&mut self.texture, self.geng.ugli()),
        );
        ugli::clear(&mut framebuffer, Some(Rgba::BLACK), None, None);

        let camera = &model.camera;

        // Telegraphs
        for tele in &model.telegraphs {
            self.util
                .draw_outline(&tele.light.collider, 0.02, camera, &mut framebuffer);
        }

        // Lights
        for light in &model.lights {
            self.util
                .draw_collider(&light.collider, camera, &mut framebuffer);
        }

        // Player
        let mut player = model.player.collider.clone();
        player.position += model.player.shake;
        self.util.draw_collider(&player, camera, &mut framebuffer);

        let aabb = Aabb2::ZERO.extend_positive(old_framebuffer.size().as_f32());
        draw_texture_fit(
            &self.texture,
            aabb,
            vec2(0.5, 0.5),
            &geng::PixelPerfectCamera,
            &self.geng,
            &mut old_framebuffer,
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
        .extend_symmetric(vec2(7.0, 0.0) * font_size / 2.0)
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
