use crate::{assets::*, model::*, render::UtilRender};

use geng::prelude::*;
use geng_utils::conversions::Vec2RealConversions;

pub struct Editor {
    geng: Geng,
    assets: Rc<Assets>,
    util_render: UtilRender,
    level: Level,
    /// Simulation model.
    model: Model,
    current_beat: usize,
    real_time: Time,
}

impl Editor {
    pub fn new(geng: Geng, assets: Rc<Assets>, config: Config, level: Level) -> Self {
        Self {
            util_render: UtilRender::new(&geng, &assets),
            geng,
            assets,
            model: Model::new(config, level.clone()),
            level,
            current_beat: 0,
            real_time: Time::ZERO,
        }
    }
}

impl geng::State for Editor {
    fn update(&mut self, delta_time: f64) {
        self.real_time += Time::new(delta_time as f32);
    }

    fn handle_event(&mut self, event: geng::Event) {
        match event {
            geng::Event::KeyPress { key } => match key {
                geng::Key::ArrowLeft => self.current_beat = self.current_beat.saturating_sub(1),
                geng::Key::ArrowRight => self.current_beat += 1,
                _ => {}
            },
            geng::Event::Wheel { delta } => {
                if delta > 0.0 {
                    self.current_beat += 1;
                } else {
                    self.current_beat = self.current_beat.saturating_sub(1);
                }
            }
            geng::Event::MousePress { .. } => {}   // TODO
            geng::Event::MouseRelease { .. } => {} // TODO
            _ => {}
        }
    }

    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        ugli::clear(framebuffer, Some(crate::render::COLOR_DARK), None, None);

        // Level
        let time = (self.real_time / self.level.beat_time()).fract() * self.level.beat_time();
        let time = time + Time::new(self.current_beat as f32) * self.level.beat_time();
        for event in &self.level.events {
            if event.beat.as_f32() <= self.current_beat as f32 {
                match &event.event {
                    Event::Light(event) => {
                        let light = event.light.clone().instantiate(self.level.beat_time());
                        let tele =
                            light.into_telegraph(event.telegraph.clone(), self.level.beat_time());
                        let duration = tele.light.movement.duration();

                        // Telegraph
                        if time < duration {
                            let transform = tele.light.movement.get(time);
                            self.util_render.draw_outline(
                                &tele.light.base_collider.transformed(transform),
                                0.02,
                                &self.model.camera,
                                framebuffer,
                            );
                        }

                        // Light
                        let time = time - tele.spawn_timer;
                        if time > Time::ZERO && time < duration {
                            let transform = tele.light.movement.get(time);
                            self.util_render.draw_collider(
                                &tele.light.base_collider.transformed(transform),
                                &self.model.camera,
                                framebuffer,
                            );
                        }
                    }
                }
            }
        }

        // UI
        let framebuffer_size = framebuffer.size().as_f32();
        let camera = &geng::PixelPerfectCamera;
        let screen = Aabb2::ZERO.extend_positive(framebuffer_size);
        let font_size = framebuffer_size.y * 0.05;
        let font = self.geng.default_font();
        let text_color = crate::render::COLOR_LIGHT;
        // let outline_color = crate::render::COLOR_DARK;
        // let outline_size = 0.05;

        font.draw(
            framebuffer,
            camera,
            &format!("Beat: {}", self.current_beat),
            vec2::splat(geng::TextAlign(0.5)),
            mat3::translate(
                geng_utils::layout::aabb_pos(screen, vec2(0.5, 1.0)) + vec2(0.0, -font_size),
            ) * mat3::scale_uniform(font_size)
                * mat3::translate(vec2(0.0, -0.5)),
            text_color,
        );
    }
}
