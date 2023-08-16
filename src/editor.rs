use crate::{assets::*, model::*, render::UtilRender};

use geng::prelude::*;
use geng_utils::conversions::Vec2RealConversions;

#[derive(Debug, Clone)]
enum State {
    /// Place a new light.
    Place,
    /// Specify a movement path for the light.
    Movement { start_beat: Time, light: LightEvent },
}

pub struct Editor {
    geng: Geng,
    assets: Rc<Assets>,
    util_render: UtilRender,
    texture: ugli::Texture,
    framebuffer_size: vec2<usize>,
    cursor_pos: vec2<f64>,
    cursor_world_pos: vec2<Coord>,
    level: Level,
    /// Simulation model.
    model: Model,
    current_beat: usize,
    real_time: Time,
    /// Whether to visualize the lights' movement for the current beat.
    visualize_beat: bool,
    selected_shape: usize,
    state: State,
}

impl Editor {
    pub fn new(geng: Geng, assets: Rc<Assets>, config: Config, level: Level) -> Self {
        let mut texture = geng_utils::texture::new_texture(geng.ugli(), vec2(360 * 16 / 9, 360));
        texture.set_filter(ugli::Filter::Nearest);
        Self {
            util_render: UtilRender::new(&geng, &assets),
            texture,
            geng,
            assets,
            framebuffer_size: vec2(1, 1),
            cursor_pos: vec2::ZERO,
            cursor_world_pos: vec2::ZERO,
            model: Model::new(config, level.clone()),
            level,
            current_beat: 0,
            real_time: Time::ZERO,
            visualize_beat: true,
            selected_shape: 0,
            state: State::Place,
        }
    }

    fn handle_digit(&mut self, digit: u8) {
        self.selected_shape = (digit as usize)
            .min(self.model.config.shapes.len())
            .saturating_sub(1);
    }

    fn cursor_down(&mut self) {
        match &mut self.state {
            State::Place => {
                // Fade in
                let movement = Movement {
                    key_frames: vec![
                        MoveFrame {
                            lerp_time: Time::ZERO, // in beats
                            transform: Transform {
                                scale: Coord::ZERO,
                                ..default()
                            },
                        },
                        MoveFrame {
                            lerp_time: Time::ONE, // in beats
                            transform: Transform::identity(),
                        },
                    ]
                    .into(),
                };
                let telegraph = Telegraph::default();
                if let Some(&shape) = self.model.config.shapes.get(self.selected_shape) {
                    self.state = State::Movement {
                        start_beat: r32(self.current_beat as f32)
                            - movement.duration()
                            - telegraph.precede_time, // extra time for the fade and telegraph
                        light: LightEvent {
                            light: LightSerde {
                                position: self.cursor_world_pos,
                                rotation: Coord::ZERO, // TODO
                                shape,
                                movement,
                            },
                            telegraph,
                        },
                    };
                }
            }
            State::Movement { start_beat, light } => {
                // TODO: check negative time
                let last_beat =
                    *start_beat + light.light.movement.duration() + light.telegraph.precede_time;
                let mut last_pos = light.light.movement.get_finish();
                last_pos.translation += light.light.position;
                light.light.movement.key_frames.push_back(MoveFrame {
                    lerp_time: r32(self.current_beat as f32) - last_beat, // in beats
                    transform: Transform {
                        translation: self.cursor_world_pos - last_pos.translation,
                        ..default()
                    },
                });
            }
        }
    }

    fn cursor_up(&mut self) {}
}

impl geng::State for Editor {
    fn update(&mut self, delta_time: f64) {
        let delta_time = Time::new(delta_time as f32);
        self.real_time += delta_time;

        let pos = self.cursor_pos.as_f32();
        let game_pos = geng_utils::layout::fit_aabb(
            self.texture.size().as_f32(),
            Aabb2::ZERO.extend_positive(self.framebuffer_size.as_f32()),
            vec2(0.5, 0.5),
        );
        let pos = pos - game_pos.bottom_left();
        self.cursor_world_pos = self
            .model
            .camera
            .screen_to_world(game_pos.size(), pos)
            .as_r32();
    }

    fn handle_event(&mut self, event: geng::Event) {
        match event {
            geng::Event::KeyPress { key } => match key {
                geng::Key::ArrowLeft => self.current_beat = self.current_beat.saturating_sub(1),
                geng::Key::ArrowRight => self.current_beat += 1,
                geng::Key::Space => self.visualize_beat = !self.visualize_beat,
                geng::Key::Digit1 => self.handle_digit(1),
                geng::Key::Digit2 => self.handle_digit(2),
                geng::Key::Digit3 => self.handle_digit(3),
                geng::Key::Digit4 => self.handle_digit(4),
                geng::Key::Digit5 => self.handle_digit(5),
                geng::Key::Digit6 => self.handle_digit(6),
                geng::Key::Digit7 => self.handle_digit(7),
                geng::Key::Digit8 => self.handle_digit(8),
                geng::Key::Digit9 => self.handle_digit(9),
                geng::Key::Digit0 => self.handle_digit(0),
                _ => {}
            },
            geng::Event::Wheel { delta } => {
                if delta > 0.0 {
                    self.current_beat += 1;
                } else {
                    self.current_beat = self.current_beat.saturating_sub(1);
                }
            }
            geng::Event::CursorMove { position } => {
                self.cursor_pos = position;
            }
            geng::Event::MousePress { button } => match button {
                geng::MouseButton::Left => self.cursor_down(),
                geng::MouseButton::Middle => {}
                geng::MouseButton::Right => {
                    if let State::Movement { start_beat, light } = &self.state {
                        self.level
                            .events
                            .push(commit_light(*start_beat, light.clone()));
                        self.state = State::Place;
                    }
                }
            },
            geng::Event::MouseRelease {
                button: geng::MouseButton::Left,
            } => self.cursor_up(),
            _ => {}
        }
    }

    fn draw(&mut self, screen_buffer: &mut ugli::Framebuffer) {
        self.framebuffer_size = screen_buffer.size();
        ugli::clear(screen_buffer, Some(crate::render::COLOR_DARK), None, None);

        let mut pixel_buffer =
            geng_utils::texture::attach_texture(&mut self.texture, self.geng.ugli());
        ugli::clear(&mut pixel_buffer, Some(Rgba::BLACK), None, None);

        // Level
        let time = if self.visualize_beat {
            (self.real_time / self.level.beat_time()).fract() * self.level.beat_time()
        } else {
            Time::ZERO
        };
        let time = time + Time::new(self.current_beat as f32) * self.level.beat_time();

        let mut draw_event = |event: &TimedEvent, transparency: f32| {
            if event.beat.as_f32() <= self.current_beat as f32 {
                let time = time - event.beat * self.level.beat_time();
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
                                transparency,
                                &self.model.camera,
                                &mut pixel_buffer,
                            );
                        }

                        // Light
                        let time = time - tele.spawn_timer;
                        if time > Time::ZERO && time < duration {
                            let transform = tele.light.movement.get(time);
                            self.util_render.draw_collider(
                                &tele.light.base_collider.transformed(transform),
                                transparency,
                                &self.model.camera,
                                &mut pixel_buffer,
                            );
                        }
                    }
                }
            }
        };

        for event in &self.level.events {
            // TODO: transparent if State::Movement
            draw_event(event, 1.0);
        }

        if let State::Movement { start_beat, light } = &self.state {
            draw_event(&commit_light(*start_beat, light.clone()), 1.0);
        };

        // Current action
        if let Some(&selected_shape) = self.model.config.shapes.get(self.selected_shape) {
            let collider = Collider {
                position: self.cursor_world_pos,
                rotation: Angle::ZERO,
                shape: selected_shape,
            };
            self.util_render.draw_outline(
                &collider,
                0.05,
                1.0,
                &self.model.camera,
                &mut pixel_buffer,
            );
        }

        let aabb = Aabb2::ZERO.extend_positive(screen_buffer.size().as_f32());
        geng_utils::texture::draw_texture_fit(
            &self.texture,
            aabb,
            vec2(0.5, 0.5),
            &geng::PixelPerfectCamera,
            &self.geng,
            screen_buffer,
        );

        // UI
        let framebuffer_size = screen_buffer.size().as_f32();
        let camera = &geng::PixelPerfectCamera;
        let screen = Aabb2::ZERO.extend_positive(framebuffer_size);
        let font_size = framebuffer_size.y * 0.05;
        let font = self.geng.default_font();
        let text_color = crate::render::COLOR_LIGHT;
        // let outline_color = crate::render::COLOR_DARK;
        // let outline_size = 0.05;

        font.draw(
            screen_buffer,
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

fn commit_light(start_beat: Time, mut light: LightEvent) -> TimedEvent {
    // Add fade out
    light.light.movement.key_frames.push_back(MoveFrame {
        lerp_time: Time::ONE, // in beats
        transform: Transform {
            scale: Coord::ZERO,
            ..default()
        },
    });

    // Commit event
    TimedEvent {
        beat: start_beat,
        event: Event::Light(light),
    }
}
