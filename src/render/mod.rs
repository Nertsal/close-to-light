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
        // Lights
        for light in &model.lights {
            self.geng.draw2d().draw2d(
                framebuffer,
                &model.camera,
                &draw2d::Ellipse::circle(
                    light.position.as_f32(),
                    light.radius.as_f32(),
                    COLOR_LIGHT,
                ),
            );
        }

        // Player
        self.geng.draw2d().draw2d(
            framebuffer,
            &model.camera,
            &draw2d::Ellipse::circle(
                model.player.position.as_f32(),
                model.player.radius.as_f32(),
                COLOR_LIGHT,
            ),
        );
    }
}
