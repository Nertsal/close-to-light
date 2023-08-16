use crate::{assets::*, model::*, render::UtilRender};

use geng::prelude::*;
use geng_utils::conversions::Vec2RealConversions;

#[derive(Debug, Clone)]
enum State {
    /// Place a new light.
    Place,
    /// Specify a movement path for the light.
    Movement {
        start_beat: Time,
        light: LightEvent,
    },
    Playing {
        start_beat: usize,
    },
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
    /// Lights (with transparency and hover) ready for visualization and hover detection.
    rendered_lights: Vec<(Light, f32, bool)>,
    /// Telegraphs (with transparency and hover) ready for visualization.
    rendered_telegraphs: Vec<(LightTelegraph, f32, bool)>,
    /// Index of the hovered light in the `level.events`.
    hovered_light: Option<usize>,
    current_beat: usize,
    time: Time,
    /// Whether to visualize the lights' movement for the current beat.
    visualize_beat: bool,
    selected_shape: usize,
    state: State,
    music: geng::SoundEffect,
}

impl Editor {
    pub fn new(geng: Geng, assets: Rc<Assets>, config: Config, level: Level) -> Self {
        let mut texture = geng_utils::texture::new_texture(geng.ugli(), vec2(360 * 16 / 9, 360));
        texture.set_filter(ugli::Filter::Nearest);
        Self {
            util_render: UtilRender::new(&geng, &assets),
            texture,
            framebuffer_size: vec2(1, 1),
            cursor_pos: vec2::ZERO,
            cursor_world_pos: vec2::ZERO,
            model: Model::new(config, level.clone()),
            rendered_lights: vec![],
            rendered_telegraphs: vec![],
            hovered_light: None,
            current_beat: 0,
            time: Time::ZERO,
            visualize_beat: true,
            selected_shape: 0,
            state: State::Place,
            music: assets.music.effect(),
            geng,
            assets,
            level,
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
            State::Playing { .. } => {}
        }
    }

    fn cursor_up(&mut self) {}

    fn render_lights(&mut self) {
        self.rendered_lights.clear();
        self.rendered_telegraphs.clear();
        self.hovered_light = None;

        let (static_time, dynamic_time) = if let State::Playing { .. } = self.state {
            // TODO: self.music.play_position()
            (None, Some(self.time))
        } else {
            let time = Time::new(self.current_beat as f32) * self.level.beat_time();
            let dynamic = if self.visualize_beat {
                Some((self.time / self.level.beat_time()).fract() * self.level.beat_time() + time)
            } else {
                None
            };
            (Some(time), dynamic)
        };

        let mut render_light = |index: Option<usize>, event: &TimedEvent, transparency: f32| {
            if event.beat.as_f32() <= self.current_beat as f32 {
                let start = event.beat * self.level.beat_time();
                let static_time = static_time.map(|t| t - start);
                let dynamic_time = dynamic_time.map(|t| t - start);

                match &event.event {
                    Event::Light(event) => {
                        let light = event.light.clone().instantiate(self.level.beat_time());
                        let mut tele =
                            light.into_telegraph(event.telegraph.clone(), self.level.beat_time());
                        let duration = tele.light.movement.duration();

                        let static_light = static_time.and_then(|time| {
                            let time = time - tele.spawn_timer;
                            (time > Time::ZERO && time < duration).then(|| {
                                let transform = tele.light.movement.get(time);
                                tele.light.collider =
                                    tele.light.base_collider.transformed(transform);
                                tele.light.clone()
                            })
                        });

                        let hover = self.hovered_light.is_none()
                            && index.is_some()
                            && static_light
                                .as_ref()
                                .map(|light| light.collider.contains(self.cursor_world_pos))
                                .unwrap_or(false);
                        if hover {
                            self.hovered_light = index;
                        }

                        if let Some(time) = dynamic_time {
                            // Telegraph
                            if time < duration {
                                let transform = tele.light.movement.get(time);
                                tele.light.collider =
                                    tele.light.base_collider.transformed(transform);
                                self.rendered_telegraphs.push((
                                    tele.clone(),
                                    transparency * 0.5,
                                    hover,
                                ));
                            }

                            // Light
                            let time = time - tele.spawn_timer;
                            if time > Time::ZERO && time < duration {
                                let transform = tele.light.movement.get(time);
                                tele.light.collider =
                                    tele.light.base_collider.transformed(transform);
                                self.rendered_lights.push((
                                    tele.light.clone(),
                                    transparency * 0.5,
                                    hover,
                                ));
                            }
                        }

                        if let Some(time) = static_time {
                            // Telegraph
                            if time < duration {
                                let transform = tele.light.movement.get(time);
                                tele.light.collider =
                                    tele.light.base_collider.transformed(transform);
                                self.rendered_telegraphs
                                    .push((tele.clone(), transparency, hover));
                            }
                        }
                        if let Some(light) = static_light {
                            self.rendered_lights.push((light, transparency, hover));
                        }
                    }
                }
            }
        };

        for (i, e) in self.level.events.iter().enumerate() {
            let transparency = if let State::Movement { .. } = &self.state {
                0.5
            } else {
                1.0
            };
            render_light(Some(i), e, transparency);
        }
        if let State::Movement { start_beat, light } = &self.state {
            render_light(None, &commit_light(*start_beat, light.clone()), 1.0);
        };
    }
}

impl geng::State for Editor {
    fn update(&mut self, delta_time: f64) {
        let delta_time = Time::new(delta_time as f32);
        self.time += delta_time;

        if let State::Playing { .. } = self.state {
            self.current_beat = (self.time / self.level.beat_time()).floor().as_f32() as usize;
        }

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

        self.render_lights();
    }

    fn handle_event(&mut self, event: geng::Event) {
        match event {
            geng::Event::KeyPress { key } => match key {
                geng::Key::ArrowLeft => self.current_beat = self.current_beat.saturating_sub(1),
                geng::Key::ArrowRight => self.current_beat += 1,
                geng::Key::F => self.visualize_beat = !self.visualize_beat,
                geng::Key::X => {
                    if let Some(index) = self.hovered_light {
                        self.level.events.swap_remove(index);
                    }
                }
                geng::Key::Space => {
                    if let State::Playing { start_beat } = &self.state {
                        self.current_beat = *start_beat;
                        self.state = State::Place;
                        self.music.stop();
                    } else {
                        self.state = State::Playing {
                            start_beat: self.current_beat,
                        };
                        self.music.stop();
                        self.music = self.assets.music.effect();
                        let time = self.current_beat as f32 * self.level.beat_time().as_f32();
                        self.time = Time::new(time);
                        self.music
                            .play_from(time::Duration::from_secs_f64(time as f64));
                    }
                }
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
                if self.geng.window().is_key_pressed(geng::Key::ControlLeft) {
                    if let Some(event) = self
                        .hovered_light
                        .and_then(|light| self.level.events.get_mut(light))
                    {
                        let change = Time::new(delta.signum() as f32 * 0.25); // Change by quarter beats
                        let Event::Light(light) = &mut event.event;
                        if self.geng.window().is_key_pressed(geng::Key::ShiftLeft) {
                            // Fade out
                            if let Some(frame) = light.light.movement.key_frames.back_mut() {
                                let change = change.max(-frame.lerp_time + r32(0.25));
                                frame.lerp_time += change;
                            }
                        } else {
                            // Fade in
                            if let Some(frame) = light.light.movement.key_frames.get_mut(1) {
                                let change = change.max(-frame.lerp_time + r32(0.25));
                                event.beat -= change;
                                frame.lerp_time += change;
                            }
                        }
                        return;
                    }
                }
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
        for (tele, transparency, hover) in &self.rendered_telegraphs {
            let color = if *hover {
                Rgba::CYAN
            } else {
                crate::render::COLOR_LIGHT
            };
            self.util_render.draw_outline(
                &tele.light.collider,
                0.02,
                crate::util::with_alpha(color, *transparency),
                &self.model.camera,
                &mut pixel_buffer,
            );
        }
        for (light, transparency, hover) in &self.rendered_lights {
            let color = if *hover {
                Rgba::CYAN
            } else {
                crate::render::COLOR_LIGHT
            };
            self.util_render.draw_collider(
                &light.collider,
                crate::util::with_alpha(color, *transparency),
                &self.model.camera,
                &mut pixel_buffer,
            );
        }

        // Current action
        if !matches!(self.state, State::Playing { .. }) {
            if let Some(&selected_shape) = self.model.config.shapes.get(self.selected_shape) {
                let collider = Collider {
                    position: self.cursor_world_pos,
                    rotation: Angle::ZERO,
                    shape: selected_shape,
                };
                self.util_render.draw_outline(
                    &collider,
                    0.05,
                    crate::render::COLOR_LIGHT,
                    &self.model.camera,
                    &mut pixel_buffer,
                );
            }
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

        // Current beat / Fade in/out
        let mut text = format!("Beat: {}", self.current_beat);
        if self.geng.window().is_key_pressed(geng::Key::ControlLeft) {
            if let Some(event) = self
                .hovered_light
                .and_then(|light| self.level.events.get_mut(light))
            {
                let Event::Light(light) = &mut event.event;
                if self.geng.window().is_key_pressed(geng::Key::ShiftLeft) {
                    if let Some(frame) = light.light.movement.key_frames.back_mut() {
                        text = format!("Fade out time: {}", frame.lerp_time);
                    }
                } else if let Some(frame) = light.light.movement.key_frames.get(1) {
                    text = format!("Fade in time: {}", frame.lerp_time);
                }
            }
        }
        font.draw(
            screen_buffer,
            camera,
            &text,
            vec2::splat(geng::TextAlign(0.5)),
            mat3::translate(
                geng_utils::layout::aabb_pos(screen, vec2(0.5, 1.0)) + vec2(0.0, -font_size),
            ) * mat3::scale_uniform(font_size)
                * mat3::translate(vec2(0.0, -0.5)),
            text_color,
        );

        // Help
        let text =
            "Scroll or arrow keys to go forward or backward in time\nSpace to play the music\nF to pause movement";
        font.draw(
            screen_buffer,
            camera,
            text,
            vec2::splat(geng::TextAlign::RIGHT),
            mat3::translate(
                geng_utils::layout::aabb_pos(screen, vec2(1.0, 1.0)) + vec2(-1.0, -1.0) * font_size,
            ) * mat3::scale_uniform(font_size * 0.5)
                * mat3::translate(vec2(0.0, -0.5)),
            text_color,
        );

        // Status
        let text = if self.hovered_light.is_some() {
            "X to delete the light\nCtrl + scroll to change fade in time\nCtrl + Shift + scroll to change fade out time"
        } else {
            match &self.state {
                State::Place => "Click to create a new light\n1/2 to select different types",
                State::Movement { .. } => {
                    "Left click to create a new waypoint\nRight click to finish"
                }
                State::Playing { .. } => "Playing the music...\nSpace to stop",
            }
        };
        font.draw(
            screen_buffer,
            camera,
            text,
            vec2(geng::TextAlign::CENTER, geng::TextAlign::BOTTOM),
            mat3::translate(
                geng_utils::layout::aabb_pos(screen, vec2(0.5, 0.0)) + vec2(0.0, 1.5 * font_size),
            ) * mat3::scale_uniform(font_size)
                * mat3::translate(vec2(0.0, 1.0)),
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
