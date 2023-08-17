use crate::{assets::Assets, model::*, render::GameRender};

use geng::prelude::*;
use geng_utils::conversions::Vec2RealConversions;

#[allow(dead_code)]
pub struct Game {
    geng: Geng,
    transition: Option<geng::state::Transition>,
    render: GameRender,
    model: Model,
    framebuffer_size: vec2<usize>,
    /// Cursor position in screen space.
    cursor_pos: vec2<f64>,
    active_touch: Option<u64>,
}

impl Game {
    pub fn new(
        geng: &Geng,
        assets: &Rc<Assets>,
        rules: Config,
        level: Level,
        start_time: Time,
    ) -> Self {
        Self {
            geng: geng.clone(),
            transition: None,
            render: GameRender::new(geng, assets),
            model: Model::new(assets, rules, level, start_time),
            framebuffer_size: vec2(1, 1),
            cursor_pos: vec2::ZERO,
            active_touch: None,
        }
    }
}

impl geng::State for Game {
    fn transition(&mut self) -> Option<geng::state::Transition> {
        self.transition.take()
    }

    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        self.framebuffer_size = framebuffer.size();
        ugli::clear(framebuffer, Some(crate::render::COLOR_DARK), None, None);
        self.render.draw_world(&self.model, framebuffer);
        self.render.draw_ui(&self.model, framebuffer);
    }

    fn handle_event(&mut self, event: geng::Event) {
        match event {
            geng::Event::KeyPress { key } => match key {
                geng::Key::Escape => self.transition = Some(geng::state::Transition::Pop),
                geng::Key::F11 => self.geng.window().toggle_fullscreen(),
                _ => {}
            },
            geng::Event::CursorMove { position } => {
                self.cursor_pos = position;
            }
            geng::Event::TouchStart(touch) if self.active_touch.is_none() => {
                self.active_touch = Some(touch.id);
            }
            geng::Event::TouchMove(touch) if Some(touch.id) == self.active_touch => {
                self.cursor_pos = touch.position;
            }
            geng::Event::TouchEnd(touch) if Some(touch.id) == self.active_touch => {
                self.active_touch = None;
            }
            _ => {}
        }
    }

    fn update(&mut self, delta_time: f64) {
        let delta_time = Time::new(delta_time as _);

        let pos = self.cursor_pos.as_f32();
        let game_pos = geng_utils::layout::fit_aabb(
            self.render.get_render_size().as_f32(),
            Aabb2::ZERO.extend_positive(self.framebuffer_size.as_f32()),
            vec2(0.5, 0.5),
        );
        let pos = pos - game_pos.bottom_left();
        let target_pos = self
            .model
            .camera
            .screen_to_world(game_pos.size(), pos)
            .as_r32();
        self.model.update(target_pos, delta_time);
    }
}
