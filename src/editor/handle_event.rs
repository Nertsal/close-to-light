use super::*;

impl EditorState {
    pub fn handle_event(&mut self, event: geng::Event) {
        let ctrl = self.geng.window().is_key_pressed(geng::Key::ControlLeft);
        let shift = self.geng.window().is_key_pressed(geng::Key::ShiftLeft);
        let alt = self.geng.window().is_key_pressed(geng::Key::AltLeft);

        let scroll_speed = if shift {
            self.editor.config.scroll_slow
        } else if alt {
            self.editor.config.scroll_fast
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
                geng::Key::F => self.editor.visualize_beat = !self.editor.visualize_beat,
                geng::Key::X => {
                    if let Some(index) = self
                        .editor
                        .selected_light
                        .and_then(|i| self.editor.level_state.light_event(i))
                    {
                        self.editor.level.events.swap_remove(index);
                    }
                }
                geng::Key::S if ctrl => {
                    self.save();
                }
                geng::Key::Q => self.editor.place_rotation += Angle::from_degrees(r32(15.0)),
                geng::Key::E => self.editor.place_rotation += Angle::from_degrees(r32(-15.0)),
                geng::Key::Z if ctrl => {
                    if shift {
                        self.redo();
                    } else {
                        self.undo();
                    }
                }
                geng::Key::H => self.render_options.hide_ui = !self.render_options.hide_ui,
                geng::Key::D => {
                    // Toggle danger
                    match &mut self.editor.state {
                        State::Idle => {
                            if let Some(event) = self
                                .editor
                                .selected_light
                                .and_then(|i| self.editor.level_state.light_event(i))
                                .and_then(|i| self.editor.level.events.get_mut(i))
                            {
                                if let Event::Light(event) = &mut event.event {
                                    event.light.danger = !event.light.danger;
                                }
                            }
                        }
                        State::Place { danger, .. } => {
                            *danger = !*danger;
                        }
                        State::Movement { light, .. } => {
                            light.light.danger = !light.light.danger;
                        }
                        _ => {}
                    }
                }
                geng::Key::Backquote => {
                    if ctrl {
                        self.render_options.show_grid = !self.render_options.show_grid;
                    } else {
                        self.editor.snap_to_grid = !self.editor.snap_to_grid;
                    }
                }
                geng::Key::Escape => {
                    // Cancel creation
                    match self.editor.state {
                        State::Movement { .. } | State::Place { .. } => {
                            self.editor.state = State::Idle;
                        }
                        State::Idle => {
                            self.editor.selected_light = None;
                        }
                        _ => (),
                    }
                }
                geng::Key::Space => {
                    if let State::Playing {
                        start_beat,
                        old_state,
                    } = &self.editor.state
                    {
                        self.editor.current_beat = *start_beat;
                        self.editor.state = *old_state.clone();
                        self.editor.music.stop();
                    } else {
                        self.editor.state = State::Playing {
                            start_beat: self.editor.current_beat,
                            old_state: Box::new(self.editor.state.clone()),
                        };
                        self.editor.music.stop();
                        self.editor.music = self.assets.music.effect();
                        // TODO: future proof in case level beat time is not constant
                        self.editor.real_time =
                            self.editor.current_beat * self.editor.level.beat_time();
                        self.editor.music.play_from(time::Duration::from_secs_f64(
                            self.editor.real_time.as_f32() as f64,
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
                let scroll = r32(delta.signum() as f32);
                if ctrl {
                    if let State::Place { .. } | State::Movement { .. } = self.editor.state {
                        // Scale light
                        let delta = scroll * r32(0.1);
                        self.editor.place_scale =
                            (self.editor.place_scale + delta).clamp(r32(0.2), r32(2.0));
                    } else if let Some(event) = self
                        .editor
                        .selected_light
                        .and_then(|light| self.editor.level.events.get_mut(light))
                    {
                        // Control fade time
                        let change = scroll * self.editor.config.scroll_slow;
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
                    self.scroll_time(scroll * scroll_speed);
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
                    } = &self.editor.state
                    {
                        self.editor
                            .level
                            .events
                            .push(commit_light(*start_beat, light.clone()));
                        self.editor.state = State::Idle;
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
        if let State::Idle | State::Place { .. } = self.editor.state {
            if let Some(&shape) = self
                .editor
                .model
                .config
                .shapes
                .get((digit as usize).saturating_sub(1))
            {
                self.editor.state = State::Place {
                    shape,
                    danger: false,
                };
            }
        }
    }

    fn cursor_down(&mut self) {
        if self.ui.game.position.contains(self.cursor_pos.as_f32()) {
            match &mut self.editor.state {
                State::Idle => {
                    // Select a light
                    if let Some(hovered) = self.editor.level_state.hovered_light {
                        self.editor.selected_light = Some(hovered);
                    }
                }
                State::Place { shape, danger } => {
                    let shape = *shape;
                    let danger = *danger;

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
                                transform: Transform {
                                    scale: self.editor.place_scale,
                                    ..default()
                                },
                            },
                        ]
                        .into(),
                    };
                    let telegraph = Telegraph::default();
                    self.editor.state = State::Movement {
                        start_beat: self.editor.current_beat
                            - movement.duration()
                            - telegraph.precede_time, // extra time for the fade and telegraph
                        light: LightEvent {
                            light: LightSerde {
                                position: self.editor.cursor_world_pos,
                                rotation: self.editor.place_rotation.as_degrees(),
                                shape,
                                movement,
                                danger,
                            },
                            telegraph,
                        },
                        redo_stack: Vec::new(),
                    };
                }
                State::Movement {
                    start_beat,
                    light,
                    redo_stack,
                } => {
                    // TODO: check negative time
                    let last_beat = *start_beat
                        + light.light.movement.duration()
                        + light.telegraph.precede_time;
                    let mut last_pos = light.light.movement.get_finish();
                    last_pos.translation += light.light.position;
                    last_pos.rotation += Angle::from_degrees(light.light.rotation);
                    light.light.movement.key_frames.push_back(MoveFrame {
                        lerp_time: self.editor.current_beat - last_beat, // in beats
                        transform: Transform {
                            translation: self.editor.cursor_world_pos - last_pos.translation,
                            rotation: last_pos.rotation.angle_to(self.editor.place_rotation),
                            scale: self.editor.place_scale,
                        },
                    });
                    redo_stack.clear();
                }
                State::Playing { .. } => {}
            }
        }
    }

    fn cursor_up(&mut self) {}
}
