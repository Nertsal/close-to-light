use super::*;

impl Editor {
    pub fn handle_event(&mut self, event: geng::Event) {
        let ctrl = self.geng.window().is_key_pressed(geng::Key::ControlLeft);
        let shift = self.geng.window().is_key_pressed(geng::Key::ShiftLeft);
        let alt = self.geng.window().is_key_pressed(geng::Key::AltLeft);

        let scroll_speed = if shift {
            self.config.scroll_slow
        } else if alt {
            self.config.scroll_fast
        } else {
            Time::ONE
        };

        match event {
            geng::Event::KeyPress { key } => match key {
                geng::Key::ArrowLeft => {
                    self.scroll_time(-scroll_speed);
                }
                geng::Key::ArrowRight => {
                    self.scroll_time(scroll_speed);
                }
                geng::Key::F => self.visualize_beat = !self.visualize_beat,
                geng::Key::X => {
                    if let Some(index) = self.level_state.hovered_event() {
                        self.level.events.swap_remove(index);
                    }
                }
                geng::Key::S if ctrl => {
                    self.save();
                }
                geng::Key::Q => self.place_rotation += Angle::from_degrees(r32(15.0)),
                geng::Key::E => self.place_rotation += Angle::from_degrees(r32(-15.0)),
                geng::Key::Z if ctrl => {
                    if shift {
                        self.redo();
                    } else {
                        self.undo();
                    }
                }
                geng::Key::H => self.hide_ui = !self.hide_ui,
                geng::Key::Backquote => {
                    if ctrl {
                        self.show_grid = !self.show_grid;
                    } else {
                        self.snap_to_grid = !self.snap_to_grid;
                    }
                }
                geng::Key::Escape => {
                    // Cancel creation
                    if let State::Movement { .. } = self.state {
                        self.state = State::Place;
                    }
                }
                geng::Key::Space => {
                    if let State::Playing {
                        start_beat,
                        old_state,
                    } = &self.state
                    {
                        self.current_beat = *start_beat;
                        self.state = *old_state.clone();
                        self.music.stop();
                    } else {
                        self.state = State::Playing {
                            start_beat: self.current_beat,
                            old_state: Box::new(self.state.clone()),
                        };
                        self.music.stop();
                        self.music = self.assets.music.effect();
                        // TODO: future proof in case level beat time is not constant
                        self.real_time = self.current_beat * self.level.beat_time();
                        self.music.play_from(time::Duration::from_secs_f64(
                            self.real_time.as_f32() as f64,
                        ));
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
                geng::Key::F5 => self.play_game(),
                geng::Key::F11 => self.geng.window().toggle_fullscreen(),
                _ => {}
            },
            geng::Event::Wheel { delta } => {
                if ctrl {
                    if let Some(event) = self
                        .level_state
                        .hovered_event()
                        .and_then(|light| self.level.events.get_mut(light))
                    {
                        // Control fade time
                        let change = Time::new(delta.signum() as f32) * self.config.scroll_slow;
                        if let Event::Light(light) = &mut event.event {
                            if shift {
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
                        }
                    }
                } else {
                    self.scroll_time(Time::new(delta.signum() as f32) * scroll_speed);
                }
            }
            geng::Event::CursorMove { position } => {
                self.cursor_pos = position;
            }
            geng::Event::MousePress { button } => match button {
                geng::MouseButton::Left => self.cursor_down(),
                geng::MouseButton::Middle => {}
                geng::MouseButton::Right => {
                    if let State::Movement {
                        start_beat, light, ..
                    } = &self.state
                    {
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
                        start_beat: self.current_beat
                            - movement.duration()
                            - telegraph.precede_time, // extra time for the fade and telegraph
                        light: LightEvent {
                            light: LightSerde {
                                position: self.cursor_world_pos,
                                rotation: self.place_rotation.as_degrees(),
                                shape,
                                movement,
                                danger: false, // TODO
                            },
                            telegraph,
                        },
                        redo_stack: Vec::new(),
                    };
                }
            }
            State::Movement {
                start_beat,
                light,
                redo_stack,
            } => {
                // TODO: check negative time
                let last_beat =
                    *start_beat + light.light.movement.duration() + light.telegraph.precede_time;
                let mut last_pos = light.light.movement.get_finish();
                last_pos.translation += light.light.position;
                last_pos.rotation += Angle::from_degrees(light.light.rotation);
                light.light.movement.key_frames.push_back(MoveFrame {
                    lerp_time: self.current_beat - last_beat, // in beats
                    transform: Transform {
                        translation: self.cursor_world_pos - last_pos.translation,
                        rotation: last_pos.rotation.angle_to(self.place_rotation),
                        ..default()
                    },
                });
                redo_stack.clear();
            }
            State::Playing { .. } => {}
        }
    }

    fn cursor_up(&mut self) {}
}
