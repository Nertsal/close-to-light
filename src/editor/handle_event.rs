use super::*;

impl EditorState {
    pub fn process_event(&self, event: geng::Event) -> Vec<EditorStateAction> {
        let mut actions = vec![];

        match &event {
            geng::Event::KeyPress { key } => {
                if let geng::Key::Escape | geng::Key::Enter = key {
                    if self.ui_context.text_edit.any_active() {
                        actions.push(EditorStateAction::StopTextEdit);
                        return actions;
                    }
                }
            }
            geng::Event::EditText(text) => {
                actions.push(EditorStateAction::UpdateTextEdit(text.clone()));
            }
            geng::Event::CursorMove { position } => {
                actions.push(EditorStateAction::CursorMove(position.as_f32()));
            }
            geng::Event::Wheel { delta } => {
                actions.push(EditorStateAction::WheelScroll(*delta as f32));
            }
            _ => (),
        }

        if self.ui_focused {
            if let geng::Event::Wheel { .. } | geng::Event::MousePress { .. } = &event {
                return actions;
            }
        }

        if !self.ui.edit.state.visible {
            return actions;
        }
        let Some(level_editor) = &self.editor.level_edit else {
            return actions;
        };

        let window = self.context.geng.window();
        let ctrl = window.is_key_pressed(geng::Key::ControlLeft);
        let shift = window.is_key_pressed(geng::Key::ShiftLeft);
        let alt = window.is_key_pressed(geng::Key::AltLeft);

        let scroll_speed = if shift {
            self.editor.config.scroll_slow
        } else if alt {
            self.editor.config.scroll_fast
        } else {
            self.editor.config.scroll_normal
        };

        match event {
            geng::Event::KeyPress { key } => match key {
                geng::Key::ArrowLeft => {
                    actions.push(EditorStateAction::ScrollTime(-scroll_speed));
                }
                geng::Key::ArrowRight => {
                    actions.push(EditorStateAction::ScrollTime(scroll_speed));
                }
                geng::Key::F => {
                    if ctrl {
                        actions.push(EditorStateAction::ClearTimelineSelection);
                    } else {
                        actions.push(EditorAction::ToggleDynamicVisual.into());
                    }
                }
                geng::Key::X => {
                    actions.push(LevelAction::DeleteSelected.into());
                }
                geng::Key::S if ctrl => {
                    actions.push(EditorAction::Save.into());
                }
                geng::Key::Q => {
                    actions.push(LevelAction::Rotate(Angle::from_degrees(r32(15.0))).into());
                }
                geng::Key::E => {
                    actions.push(LevelAction::Rotate(Angle::from_degrees(r32(-15.0))).into());
                }
                geng::Key::Z if ctrl => {
                    if shift {
                        actions.push(LevelAction::Redo.into());
                    } else {
                        actions.push(LevelAction::Undo.into());
                    }
                }
                geng::Key::H => {
                    actions.push(EditorAction::ToggleUI.into());
                }
                geng::Key::D => {
                    actions.push(LevelAction::ToggleDanger.into());
                }
                geng::Key::W => {
                    actions.push(LevelAction::ToggleWaypointsView.into());
                }
                geng::Key::Backquote => {
                    if ctrl {
                        actions.push(EditorAction::ToggleGrid.into());
                    } else {
                        actions.push(EditorAction::ToggleGridSnap.into());
                    }
                }
                geng::Key::Escape => {
                    actions.push(LevelAction::Cancel.into());
                }
                geng::Key::Space => {
                    if let State::Playing { .. } = &level_editor.state {
                        actions.push(LevelAction::StopPlaying.into());
                    } else {
                        actions.push(LevelAction::StartPlaying.into());
                    }
                }
                geng::Key::Digit1 => actions.extend(self.handle_digit(1)),
                geng::Key::Digit2 => actions.extend(self.handle_digit(2)),
                geng::Key::Digit3 => actions.extend(self.handle_digit(3)),
                geng::Key::Digit4 => actions.extend(self.handle_digit(4)),
                geng::Key::Digit5 => actions.extend(self.handle_digit(5)),
                geng::Key::Digit6 => actions.extend(self.handle_digit(6)),
                geng::Key::Digit7 => actions.extend(self.handle_digit(7)),
                geng::Key::Digit8 => actions.extend(self.handle_digit(8)),
                geng::Key::Digit9 => actions.extend(self.handle_digit(9)),
                geng::Key::Digit0 => actions.extend(self.handle_digit(0)),
                geng::Key::F1 => {
                    actions.push(EditorAction::ToggleUI.into());
                }
                geng::Key::F5 => {
                    actions.push(EditorStateAction::StartPlaytest);
                }
                geng::Key::F11 => window.toggle_fullscreen(),
                _ => {}
            },
            geng::Event::Wheel { delta } => {
                let delta = delta as f32;
                if !self.ui_focused && self.ui.edit.state.visible {
                    let scroll = r32(delta.signum());
                    if shift && self.ui.edit.timeline.state.hovered {
                        actions.push(EditorStateAction::TimelineScroll(scroll));
                    } else if ctrl {
                        if self.ui.edit.timeline.state.hovered {
                            // Zoom on the timeline
                            actions.push(EditorStateAction::TimelineZoom(scroll));
                        } else if let State::Place { .. }
                        | State::Waypoints {
                            state: WaypointsState::New,
                            ..
                        } = level_editor.state
                        {
                            // Scale light
                            let delta = scroll * r32(0.1);
                            actions.push(LevelAction::ScaleLight(delta).into());
                        } else if level_editor.level_state.waypoints.is_some() {
                            let delta = scroll * r32(0.1);
                            actions.push(LevelAction::ScaleWaypoint(delta).into());
                        } else if let Some(id) = level_editor.selected_light {
                            // Control fade time
                            let change = scroll * self.editor.config.scroll_slow;
                            let action = if shift {
                                LevelAction::ChangeFadeOut(id, change)
                            } else {
                                LevelAction::ChangeFadeIn(id, change)
                            };
                            actions.push(action.into());
                        }
                    } else {
                        actions.push(EditorStateAction::ScrollTime(scroll * scroll_speed));
                    }
                }
            }
            geng::Event::MousePress { button } => match button {
                geng::MouseButton::Left => actions.extend(self.cursor_down()),
                geng::MouseButton::Middle => {}
                geng::MouseButton::Right => {
                    actions.push(LevelAction::Cancel.into());
                }
            },
            geng::Event::MouseRelease {
                button: geng::MouseButton::Left,
            } => actions.extend(self.cursor_up()),
            _ => {}
        }
        actions
    }

    fn handle_digit(&self, digit: u8) -> Vec<EditorStateAction> {
        let mut actions = vec![];

        let Some(level_editor) = &self.editor.level_edit else {
            return actions;
        };

        match &level_editor.state {
            State::Idle | State::Place { .. } => {
                if let Some(&shape) = self
                    .editor
                    .config
                    .shapes
                    .get((digit as usize).saturating_sub(1))
                {
                    actions.push(LevelAction::NewLight(shape).into());
                }
            }
            State::Waypoints { .. } => {
                // TODO: better key
                actions.push(LevelAction::NewWaypoint.into());
            }
            _ => (),
        }
        actions
    }

    fn cursor_down(&self) -> Vec<EditorStateAction> {
        if self
            .ui
            .game
            .position
            .contains(self.ui_context.cursor.position)
            || self.editor.render_options.hide_ui
        {
            self.game_cursor_down()
        } else {
            vec![]
        }
    }

    fn cursor_up(&self) -> Vec<EditorStateAction> {
        vec![EditorStateAction::EndDrag]
    }

    pub(super) fn update_drag(&mut self) {
        let Some(level_editor) = &mut self.editor.level_edit else {
            return;
        };

        let Some(drag) = &mut self.drag else { return };
        match drag.target {
            DragTarget::Light {
                event,
                initial_time,
                initial_translation,
                ..
            } => {
                if let Some(event) = level_editor.level.events.get_mut(event) {
                    if let Event::Light(light) = &mut event.event {
                        // Move temporaly
                        event.beat = level_editor.current_beat - drag.from_beat + initial_time;

                        // Move spatially
                        let movement = &mut light.light.movement;
                        let target = initial_translation + self.editor.cursor_world_pos_snapped
                            - drag.from_world;
                        let delta = target - movement.initial.translation;
                        movement.initial.translation += delta;
                        for frame in &mut movement.key_frames {
                            frame.transform.translation += delta;
                        }
                    }
                }
            }
            DragTarget::Waypoint {
                event,
                waypoint,
                initial_translation,
            } => {
                if let Some(event) = level_editor.level.events.get_mut(event) {
                    if let Event::Light(light) = &mut event.event {
                        // Move spatially
                        if let Some(frame) = light.light.movement.get_frame_mut(waypoint) {
                            frame.translation = initial_translation
                                + self.editor.cursor_world_pos_snapped
                                - drag.from_world;
                        }
                    }
                }
            }
        }
    }

    fn game_cursor_down(&self) -> Vec<EditorStateAction> {
        let mut actions = vec![];

        let Some(level_editor) = &self.editor.level_edit else {
            return actions;
        };

        match &level_editor.state {
            State::Idle => {
                // Select a light
                if let Some(event) = level_editor.level_state.hovered_event() {
                    let light_id = LightId { event };
                    actions.push(LevelAction::SelectLight(light_id).into());

                    let double = level_editor.selected_light == Some(light_id);
                    if let Some(e) = level_editor.level.events.get(event) {
                        if let Event::Light(light) = &e.event {
                            let target = DragTarget::Light {
                                double,
                                event,
                                initial_time: e.beat,
                                initial_translation: light.light.movement.initial.translation,
                            };
                            actions.push(EditorStateAction::StartDrag(target));
                        }
                    }
                } else {
                    // Deselect
                    actions.push(LevelAction::DeselectLight.into());
                }
            }
            State::Place { .. } => {
                actions.push(LevelAction::PlaceLight(self.editor.cursor_world_pos_snapped).into());
            }
            State::Playing { .. } => {}
            State::Waypoints { state, .. } => match state {
                WaypointsState::Idle => {
                    if let Some(waypoints) = &level_editor.level_state.waypoints {
                        if let Some(hovered) =
                            waypoints.hovered.and_then(|i| waypoints.points.get(i))
                        {
                            if let Some(waypoint) = hovered.original {
                                if let Some(event) = level_editor.level.events.get(waypoints.event)
                                {
                                    if let Event::Light(event) = &event.event {
                                        if let Some(frame) =
                                            event.light.movement.get_frame(waypoint)
                                        {
                                            actions
                                                .push(LevelAction::SelectWaypoint(waypoint).into());
                                            let event = waypoints.event;
                                            let initial_translation = frame.translation;
                                            actions.push(EditorStateAction::StartDrag(
                                                DragTarget::Waypoint {
                                                    event,
                                                    waypoint,
                                                    initial_translation,
                                                },
                                            ));
                                        }
                                    }
                                }
                            }
                        } else {
                            // Deselect
                            actions.push(LevelAction::DeselectWaypoint.into());
                        }
                    }
                }
                WaypointsState::New => {
                    actions.push(
                        LevelAction::PlaceWaypoint(self.editor.cursor_world_pos_snapped).into(),
                    );
                }
            },
        }
        actions
    }
}
