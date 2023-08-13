use crate::{assets::Assets, model::*};

use geng::prelude::*;

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

    pub fn draw(&mut self, _model: &Model, _framebuffer: &mut ugli::Framebuffer) {}
}
