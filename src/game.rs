use crate::{assets::Assets, model::*, render::GameRender};

use geng::prelude::*;

#[allow(dead_code)]
pub struct Game {
    geng: Geng,
    render: GameRender,
    model: Model,
}

impl Game {
    pub fn new(geng: &Geng, assets: &Rc<Assets>) -> Self {
        Self {
            geng: geng.clone(),
            render: GameRender::new(geng, assets),
            model: Model::new(),
        }
    }
}

impl geng::State for Game {
    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        ugli::clear(framebuffer, Some(Rgba::BLACK), None, None);
        self.render.draw(&self.model, framebuffer);
    }

    fn handle_event(&mut self, _event: geng::Event) {}

    fn update(&mut self, delta_time: f64) {
        let delta_time = Time::new(delta_time as _);
        self.model.update(delta_time);
    }
}
