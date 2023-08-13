use crate::{assets::Assets, model::*};

use geng::prelude::*;
use geng_utils::conversions::Vec2RealConversions;

pub const COLOR_LIGHT: Rgba<f32> = Rgba::WHITE;
pub const COLOR_DARK: Rgba<f32> = Rgba::BLACK;

#[allow(dead_code)]
pub struct GameRender {
    geng: Geng,
    assets: Rc<Assets>,
}

impl GameRender {
    pub fn new(geng: &Geng, assets: &Rc<Assets>) -> Self {
        Self {
            geng: geng.clone(),
            assets: assets.clone(),
        }
    }

    pub fn draw_world(&mut self, model: &Model, framebuffer: &mut ugli::Framebuffer) {
        let camera = &model.camera;

        // Lights
        for light in &model.lights {
            self.draw_collider(&light.collider, camera, framebuffer);
        }

        // Player
        self.draw_collider(&model.player.collider, camera, framebuffer);
    }

    fn draw_collider(
        &self,
        collider: &Collider,
        camera: &Camera2d,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        match collider.shape {
            Shape::Circle { radius } => {
                self.geng.draw2d().draw2d(
                    framebuffer,
                    camera,
                    &draw2d::Ellipse::circle(
                        collider.position.as_f32(),
                        radius.as_f32(),
                        COLOR_LIGHT,
                    ),
                );
            }
            Shape::Line { width } => {
                self.geng.draw2d().draw2d(
                    framebuffer,
                    camera,
                    &draw2d::Quad::new(
                        Aabb2::ZERO.extend_symmetric(vec2(camera.fov * 4.0, width.as_f32()) / 2.0),
                        COLOR_LIGHT,
                    )
                    .rotate(collider.rotation.map(Coord::as_f32))
                    .translate(collider.position.as_f32()),
                );
            }
            Shape::Rectangle { width, height } => {
                self.geng.draw2d().draw2d(
                    framebuffer,
                    camera,
                    &draw2d::Quad::new(
                        Aabb2::ZERO.extend_symmetric(vec2(width.as_f32(), height.as_f32()) / 2.0),
                        COLOR_LIGHT,
                    )
                    .rotate(collider.rotation.map(Coord::as_f32))
                    .translate(collider.position.as_f32()),
                );
            }
        }
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
