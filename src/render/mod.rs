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

    pub fn draw(&mut self, model: &Model, framebuffer: &mut ugli::Framebuffer) {
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
                        Aabb2::point(collider.position.as_f32())
                            .extend_symmetric(vec2(camera.fov * 3.0, width.as_f32()) / 2.0),
                        COLOR_LIGHT,
                    )
                    .rotate(collider.rotation.map(Coord::as_f32)),
                );
            }
            Shape::Rectangle { width, height } => {
                self.geng.draw2d().draw2d(
                    framebuffer,
                    camera,
                    &draw2d::Quad::new(
                        Aabb2::point(collider.position.as_f32())
                            .extend_symmetric(vec2(width.as_f32(), height.as_f32()) / 2.0),
                        COLOR_LIGHT,
                    )
                    .rotate(collider.rotation.map(Coord::as_f32)),
                );
            }
        }
    }
}
