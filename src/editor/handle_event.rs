use super::*;

impl EditorState {
    pub fn handle_event(&mut self, event: geng::Event) {
        match &event {
            geng::Event::EditText(text) => {
                self.ui_context.text_edit.text.clone_from(text);
            }
            geng::Event::CursorMove { position } => {
                self.ui_context.cursor.cursor_move(position.as_f32());
            }
            geng::Event::Wheel { delta } => {
                self.ui_context.cursor.scroll += *delta as f32;
            }
            _ => (),
        }

        if !self.ui.edit.state.visible {
            return;
        }

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
                    self.editor.scroll_time(-scroll_speed);
                }
                geng::Key::ArrowRight => {
                    self.editor.scroll_time(scroll_speed);
                }
                geng::Key::F => {
                    if ctrl {
                        self.editor.dynamic_segment = None;
                        self.ui.edit.timeline.clear_selection();
                    } else {
                        self.editor.visualize_beat = !self.editor.visualize_beat
                    }
                }
                geng::Key::X => {
                    if let Some(waypoints) = &self.editor.level_state.waypoints {
                        // Delete a waypoint
                        if let Some(index) = waypoints.selected {
                            if let Some(event) = self.editor.level.events.get_mut(waypoints.event) {
                                if let Event::Light(light) = &mut event.event {
                                    match index {
                                        WaypointId::Initial => {
                                            match light.light.movement.key_frames.pop_front() {
                                                None => {
                                                    // No waypoints -> delete the whole event
                                                    self.editor
                                                        .level
                                                        .events
                                                        .swap_remove(waypoints.event);
                                                    self.editor.level_state.waypoints = None;
                                                    self.editor.state = State::Idle;
                                                }
                                                Some(frame) => {
                                                    // Make the first frame the initial position
                                                    light.light.movement.initial = frame.transform;
                                                    event.beat += frame.lerp_time;
                                                }
                                            }
                                        }
                                        WaypointId::Frame(i) => {
                                            if let Some(frame) =
                                                light.light.movement.key_frames.remove(i)
                                            {
                                                // Offset the next one
                                                if let Some(next) =
                                                    light.light.movement.key_frames.get_mut(i)
                                                {
                                                    next.lerp_time += frame.lerp_time;
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    } else if let Some(index) = self.editor.selected_light {
                        self.editor.level.events.swap_remove(index.event);
                    }
                    self.save_state(default());
                }
                geng::Key::S if ctrl => {
                    self.save();
                }
                geng::Key::Q => self.rotate(Angle::from_degrees(r32(15.0))),
                geng::Key::E => self.rotate(Angle::from_degrees(r32(-15.0))),
                geng::Key::Z if ctrl => {
                    if shift {
                        self.redo();
                    } else {
                        self.undo();
                    }
                }
                geng::Key::H => {
                    self.editor.render_options.hide_ui = !self.editor.render_options.hide_ui
                }
                geng::Key::D => {
                    // Toggle danger
                    match &mut self.editor.state {
                        State::Idle => {
                            if let Some(event) = self
                                .editor
                                .selected_light
                                .and_then(|i| self.editor.level.events.get_mut(i.event))
                            {
                                if let Event::Light(event) = &mut event.event {
                                    event.light.danger = !event.light.danger;
                                }
                            }
                        }
                        State::Waypoints { event, .. } => {
                            if let Some(event) = self.editor.level.events.get_mut(*event) {
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
                    self.save_state(default());
                }
                geng::Key::W => self.editor.view_waypoints(),
                geng::Key::Backquote => {
                    if ctrl {
                        self.editor.render_options.show_grid =
                            !self.editor.render_options.show_grid;
                    } else {
                        self.editor.snap_to_grid = !self.editor.snap_to_grid;
                    }
                }
                geng::Key::Escape => {
                    match &mut self.editor.state {
                        State::Idle => {
                            // Cancel selection
                            self.editor.selected_light = None;
                        }
                        State::Movement { .. } | State::Place { .. } => {
                            // Cancel creation
                            self.editor.state = State::Idle;
                        }
                        State::Waypoints { state, .. } => {
                            if let Some(waypoints) = &mut self.editor.level_state.waypoints {
                                if waypoints.selected.take().is_some() {
                                    // Cancel selection
                                    return;
                                }
                            }
                            // Cancel selection
                            match state {
                                WaypointsState::Idle => self.editor.state = State::Idle,
                                WaypointsState::New => *state = WaypointsState::Idle,
                            }
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
                        self.context.music.stop();
                    } else {
                        self.editor.state = State::Playing {
                            start_beat: self.editor.current_beat,
                            old_state: Box::new(self.editor.state.clone()),
                        };
                        // TODO: future proof in case level beat time is not constant
                        self.editor.real_time = self.editor.current_beat
                            * self.editor.static_level.music.meta.beat_time();
                        self.context.music.play_from(
                            &self.editor.static_level.music,
                            time::Duration::from_secs_f64(self.editor.real_time.as_f32() as f64),
                        );
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
                geng::Key::F1 => {
                    self.editor.render_options.hide_ui = !self.editor.render_options.hide_ui
                }
                geng::Key::F5 => self.play_game(),
                geng::Key::F11 => window.toggle_fullscreen(),
                _ => {}
            },
            geng::Event::Wheel { delta } => {
                let delta = delta as f32;
                if !self.ui_focused && self.ui.edit.state.visible {
                    let scroll = r32(delta.signum());
                    if ctrl {
                        if let State::Place { .. }
                        | State::Movement { .. }
                        | State::Waypoints {
                            state: WaypointsState::New,
                            ..
                        } = self.editor.state
                        {
                            // Scale light
                            let delta = scroll * r32(0.1);
                            self.editor.place_scale =
                                (self.editor.place_scale + delta).clamp(r32(0.2), r32(2.0));
                        } else if let Some(waypoints) = &self.editor.level_state.waypoints {
                            if let Some(selected) = waypoints.selected {
                                if let Some(event) =
                                    self.editor.level.events.get_mut(waypoints.event)
                                {
                                    if let Event::Light(light) = &mut event.event {
                                        if let Some(frame) =
                                            light.light.movement.get_frame_mut(selected)
                                        {
                                            let delta = scroll * r32(0.1);
                                            frame.scale =
                                                (frame.scale + delta).clamp(r32(0.2), r32(2.0));
                                        }
                                    }
                                }
                            }
                        } else if let Some(event) = self
                            .editor
                            .selected_light
                            .and_then(|light| self.editor.level.events.get_mut(light.event))
                        {
                            // Control fade time
                            let change = scroll * self.editor.config.scroll_slow;
                            if let Event::Light(light) = &mut event.event {
                                let movement = &mut light.light.movement;
                                if shift {
                                    // Fade out
                                    let change = change.max(-movement.fade_out + r32(0.25));
                                    movement.fade_out =
                                        (movement.fade_out + change).clamp(r32(0.0), r32(10.0));
                                } else {
                                    // Fade in
                                    let change = change.max(-movement.fade_in + r32(0.25));
                                    let target =
                                        (movement.fade_in + change).clamp(r32(0.0), r32(10.0));
                                    event.beat -= target - movement.fade_in;
                                    movement.fade_in = target;
                                }
                            }
                        }
                        self.save_state(HistoryLabel::Scroll);
                    } else {
                        self.scroll_time(scroll * scroll_speed);
                    }
                }
            }
            geng::Event::CursorMove { .. } => {
                if let Some(drag) = &mut self.drag {
                    drag.moved = true;
                }
            }
            geng::Event::MousePress { button } => match button {
                geng::MouseButton::Left => self.cursor_down(),
                geng::MouseButton::Middle => {}
                geng::MouseButton::Right => {
                    if let State::Movement {
                        start_beat, light, ..
                    } = &self.editor.state
                    {
                        // extra time for the fade in and telegraph
                        let beat = *start_beat
                            - light.light.movement.fade_in
                            - light.telegraph.precede_time;
                        let event = commit_light(light.clone());
                        let event = TimedEvent {
                            beat,
                            event: Event::Light(event),
                        };
                        self.editor.level.events.push(event);
                        self.editor.state = State::Idle;
                        self.save_state(default());
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
        match &mut self.editor.state {
            State::Idle | State::Place { .. } => {
                if let Some(&shape) = self
                    .editor
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
            State::Waypoints { state, .. } => {
                // TODO: better key
                *state = WaypointsState::New;
            }
            _ => (),
        }
    }

    fn cursor_down(&mut self) {
        if self
            .ui
            .game
            .position
            .contains(self.ui_context.cursor.position)
            || self.editor.render_options.hide_ui
        {
            self.game_cursor_down();
        }
    }

    fn cursor_up(&mut self) {
        self.end_drag();
    }

    fn scroll_time(&mut self, mut delta: Time) {
        // if let Some(drag) = &self.drag {
        //     let DragTarget::Waypoint {
        //         event, waypoint, ..
        //     } = drag.target;
        //     {
        if let Some(waypoints) = &self.editor.level_state.waypoints {
            if let Some(waypoint) = waypoints.selected {
                // Move waypoint in time
                if let Some(event) = self.editor.level.events.get_mut(waypoints.event) {
                    if let Event::Light(light) = &mut event.event {
                        // Move temporaly
                        if let Some(beat) = light.light.movement.get_time(waypoint) {
                            // let current = self.editor.current_beat
                            //     - (event.beat + light.telegraph.precede_time);
                            // let delta = current - beat;

                            let next_i = match waypoint {
                                WaypointId::Initial => 0,
                                WaypointId::Frame(i) => i + 1,
                            };
                            let next = WaypointId::Frame(next_i);
                            let next_time = light.light.movement.get_time(next);

                            let min_lerp = r32(0.25);
                            let max_delta =
                                next_time.map_or(r32(100.0), |time| time - min_lerp - beat);

                            delta = delta.min(max_delta);

                            match waypoint {
                                WaypointId::Initial => event.beat += delta,
                                WaypointId::Frame(i) => {
                                    if let Some(frame) = light.light.movement.key_frames.get_mut(i)
                                    {
                                        let target = (frame.lerp_time + delta).max(min_lerp);
                                        delta = target - frame.lerp_time;
                                        frame.lerp_time = target;
                                    }
                                }
                            }

                            if let Some(next) = light.light.movement.key_frames.get_mut(next_i) {
                                next.lerp_time -= delta;
                            }
                        }
                    }
                }
                self.save_state(HistoryLabel::Scroll);
                return;
            }
        }

        // Scroll current time
        self.editor.scroll_time(delta);
    }

    fn start_drag(&mut self, target: DragTarget) {
        self.end_drag();
        self.drag = Some(Drag {
            moved: false,
            from_screen: self.ui_context.cursor.position,
            from_world: self.editor.cursor_world_pos,
            from_time: self.editor.current_beat,
            target,
        });
    }

    fn end_drag(&mut self) {
        if let Some(_drag) = self.drag.take() {
            self.save_state(default());
        }
    }

    pub(super) fn update_drag(&mut self) {
        let Some(drag) = &mut self.drag else { return };
        match drag.target {
            DragTarget::Light {
                event,
                initial_time,
                initial_translation,
            } => {
                if let Some(event) = self.editor.level.events.get_mut(event) {
                    if let Event::Light(light) = &mut event.event {
                        // Move temporaly
                        event.beat = self.editor.current_beat - drag.from_time + initial_time;

                        // Move spatially
                        let movement = &mut light.light.movement;
                        let target =
                            initial_translation + self.editor.cursor_world_pos - drag.from_world;
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
                if let Some(event) = self.editor.level.events.get_mut(event) {
                    if let Event::Light(light) = &mut event.event {
                        // Move spatially
                        if let Some(frame) = light.light.movement.get_frame_mut(waypoint) {
                            frame.translation = initial_translation + self.editor.cursor_world_pos
                                - drag.from_world;
                        }
                    }
                }
            }
        }
    }

    fn game_cursor_down(&mut self) {
        match &mut self.editor.state {
            State::Idle => {
                // Select a light
                if let Some(event) = self.editor.level_state.hovered_event() {
                    self.editor.selected_light = Some(LightId { event });
                    if let Some(e) = self.editor.level.events.get(event) {
                        if let Event::Light(light) = &e.event {
                            self.start_drag(DragTarget::Light {
                                event,
                                initial_time: e.beat,
                                initial_translation: light.light.movement.initial.translation,
                            });
                        }
                    }
                } else {
                    // Deselect
                    self.editor.selected_light = None;
                }
            }
            State::Place { shape, danger } => {
                let shape = *shape;
                let danger = *danger;

                // Fade in
                let movement = Movement {
                    initial: Transform {
                        translation: self.editor.cursor_world_pos,
                        rotation: self.editor.place_rotation,
                        scale: self.editor.place_scale,
                    },
                    ..default()
                };
                let telegraph = Telegraph::default();
                self.editor.state = State::Movement {
                    start_beat: self.editor.current_beat,
                    light: LightEvent {
                        light: LightSerde {
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
                let last_beat = *start_beat + light.light.movement.movement_duration();
                light.light.movement.key_frames.push_back(MoveFrame {
                    lerp_time: self.editor.current_beat - last_beat, // in beats
                    transform: Transform {
                        translation: self.editor.cursor_world_pos,
                        rotation: self.editor.place_rotation,
                        scale: self.editor.place_scale,
                    },
                });
                redo_stack.clear();
            }
            State::Playing { .. } => {}
            State::Waypoints { state, .. } => match state {
                WaypointsState::Idle => {
                    if let Some(waypoints) = &mut self.editor.level_state.waypoints {
                        if let Some(hovered) =
                            waypoints.hovered.and_then(|i| waypoints.points.get(i))
                        {
                            if let Some(waypoint) = hovered.original {
                                if let Some(event) =
                                    self.editor.level.events.get_mut(waypoints.event)
                                {
                                    if let Event::Light(event) = &mut event.event {
                                        if let Some(frame) =
                                            event.light.movement.get_frame_mut(waypoint)
                                        {
                                            waypoints.selected = Some(waypoint);
                                            let event = waypoints.event;
                                            let initial_translation = frame.translation;
                                            self.start_drag(DragTarget::Waypoint {
                                                event,
                                                waypoint,
                                                initial_translation,
                                            });
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                WaypointsState::New => {
                    if let Some(waypoints) = &self.editor.level_state.waypoints {
                        if let Some(event) = self.editor.level.events.get_mut(waypoints.event) {
                            if let Event::Light(light) = &mut event.event {
                                if let Some(i) = waypoints
                                    .points
                                    .iter()
                                    .position(|point| point.original.is_none())
                                {
                                    let mut transform = Transform {
                                        translation: self.editor.cursor_world_pos,
                                        rotation: self.editor.place_rotation,
                                        scale: self.editor.place_scale,
                                    };
                                    match i.checked_sub(1) {
                                        None => {
                                            // Replace initial
                                            std::mem::swap(
                                                &mut light.light.movement.initial,
                                                &mut transform,
                                            );

                                            let time = self.editor.current_beat
                                                - light.light.movement.fade_in
                                                - light.telegraph.precede_time; // Extra time for fade in and telegraph
                                            light.light.movement.key_frames.push_front(MoveFrame {
                                                lerp_time: event.beat - time,
                                                transform,
                                            });
                                            event.beat = time;
                                        }
                                        Some(i) => {
                                            // Insert keyframe
                                            let last =
                                                light.light.movement.timed_positions().nth(i);
                                            if let Some((_, _, last_time)) = last {
                                                let last_time = event.beat
                                                    + light.telegraph.precede_time
                                                    + last_time;
                                                let lerp_time =
                                                    self.editor.current_beat - last_time;

                                                light.light.movement.key_frames.insert(
                                                    i,
                                                    MoveFrame {
                                                        lerp_time,
                                                        transform,
                                                    },
                                                );

                                                if let Some(next) =
                                                    light.light.movement.key_frames.get_mut(i + 1)
                                                {
                                                    next.lerp_time -= lerp_time;
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            },
        }
        self.save_state(default());
    }

    fn rotate(&mut self, delta: Angle<Coord>) {
        self.editor.place_rotation += delta;
        if let Some(frame) = self
            .editor
            .level_state
            .waypoints
            .as_ref()
            .and_then(|waypoints| {
                waypoints.selected.and_then(|selected| {
                    self.editor
                        .level
                        .events
                        .get_mut(waypoints.event)
                        .and_then(|event| {
                            if let Event::Light(event) = &mut event.event {
                                event.light.movement.get_frame_mut(selected)
                            } else {
                                None
                            }
                        })
                })
            })
        {
            frame.rotation += delta;
        }
    }
}
