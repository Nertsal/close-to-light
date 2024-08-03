use super::*;

#[derive(Debug, Clone)]
pub enum LevelAction {
    DeleteSelected,
    Undo,
    Redo,
    Rotate(Angle<Coord>),
    ToggleDanger,
    ToggleWaypointsView,
    Cancel,
    StopPlaying,
    StartPlaying,
    NewLight(Shape),
    PlaceLight(vec2<Coord>),
    NewWaypoint,
    PlaceWaypoint(vec2<Coord>),
    ScaleLight(Coord),
    ScaleWaypoint(Coord),
    ChangeFadeOut(LightId, Time),
    ChangeFadeIn(LightId, Time),
    CommitLight,
    DeselectLight,
    SelectLight(LightId),
    SelectWaypoint(WaypointId),
    DeselectWaypoint,
}

impl LevelEditor {
    pub fn execute(&mut self, action: LevelAction) {
        match action {
            LevelAction::DeleteSelected => {
                if !self.delete_waypoint_selected() {
                    self.delete_light_selected();
                }
            }
            LevelAction::Undo => self.undo(),
            LevelAction::Redo => self.redo(),
            LevelAction::Rotate(delta) => self.rotate(delta),
            LevelAction::ToggleDanger => self.toggle_danger(),
            LevelAction::ToggleWaypointsView => self.view_waypoints(),
            LevelAction::Cancel => self.cancel(),
            LevelAction::StopPlaying => {
                if let State::Playing {
                    start_beat,
                    old_state,
                } = &self.state
                {
                    self.current_beat = *start_beat;
                    self.state = *old_state.clone();
                    self.context.music.stop();
                }
            }
            LevelAction::StartPlaying => {
                self.state = State::Playing {
                    start_beat: self.current_beat,
                    old_state: Box::new(self.state.clone()),
                };
                // TODO: future proof in case level beat time is not constant
                self.real_time = self.current_beat * self.static_level.group.music.meta.beat_time();
                self.context.music.play_from(
                    &self.static_level.group.music,
                    time::Duration::from_secs_f64(self.real_time.as_f32() as f64),
                );
            }
            LevelAction::NewLight(shape) => {
                self.state = State::Place {
                    shape,
                    danger: false,
                };
            }
            LevelAction::PlaceLight(position) => {
                if let State::Place { shape, danger } = &self.state {
                    let shape = *shape;
                    let danger = *danger;

                    // Fade in
                    let movement = Movement {
                        initial: Transform {
                            translation: position,
                            rotation: self.place_rotation.normalized_2pi(),
                            scale: self.place_scale,
                        },
                        ..default()
                    };
                    let telegraph = Telegraph::default();
                    self.state = State::Movement {
                        start_beat: self.current_beat,
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
            }
            LevelAction::NewWaypoint => self.new_waypoint(),
            LevelAction::ScaleLight(delta) => {
                self.place_scale = (self.place_scale + delta).clamp(r32(0.2), r32(2.0));
                self.save_state(HistoryLabel::Scroll);
            }
            LevelAction::ScaleWaypoint(delta) => {
                if let Some(waypoints) = &self.level_state.waypoints {
                    if let Some(selected) = waypoints.selected {
                        if let Some(event) = self.level.events.get_mut(waypoints.event) {
                            if let Event::Light(light) = &mut event.event {
                                if let Some(frame) = light.light.movement.get_frame_mut(selected) {
                                    frame.scale = (frame.scale + delta).clamp(r32(0.2), r32(2.0));
                                    self.save_state(HistoryLabel::Scroll);
                                }
                            }
                        }
                    }
                }
            }
            LevelAction::ChangeFadeOut(id, change) => {
                if let Some(event) = self.level.events.get_mut(id.event) {
                    if let Event::Light(light) = &mut event.event {
                        let movement = &mut light.light.movement;
                        movement.change_fade_out(movement.fade_out + change);
                        self.save_state(HistoryLabel::Scroll);
                    }
                }
            }
            LevelAction::ChangeFadeIn(id, change) => {
                if let Some(event) = self.level.events.get_mut(id.event) {
                    if let Event::Light(light) = &mut event.event {
                        let movement = &mut light.light.movement;
                        let from = movement.fade_in;
                        movement.change_fade_in(movement.fade_in + change);
                        event.beat -= movement.fade_in - from;
                        self.save_state(HistoryLabel::Scroll);
                    }
                }
            }
            LevelAction::CommitLight => {
                if let State::Movement {
                    start_beat, light, ..
                } = &self.state
                {
                    // extra time for the fade in and telegraph
                    let beat =
                        *start_beat - light.light.movement.fade_in - light.telegraph.precede_time;
                    let event = commit_light(light.clone());
                    let event = TimedEvent {
                        beat,
                        event: Event::Light(event),
                    };
                    self.level.events.push(event);
                    self.state = State::Idle;
                    self.save_state(default());
                }
            }
            LevelAction::DeselectLight => {
                self.execute(LevelAction::DeselectWaypoint);
                self.selected_light = None;
            }
            LevelAction::SelectLight(id) => {
                self.level_state.waypoints = None;
                self.selected_light = Some(id);
            }
            LevelAction::SelectWaypoint(id) => {
                if let Some(waypoints) = &mut self.level_state.waypoints {
                    // TODO: validation check
                    waypoints.selected = Some(id);
                }
            }
            LevelAction::DeselectWaypoint => {
                if let Some(waypoints) = &mut self.level_state.waypoints {
                    waypoints.selected = None;
                }
            }
            LevelAction::PlaceWaypoint(position) => self.place_waypoint(position),
        }
    }

    fn rotate(&mut self, delta: Angle<Coord>) {
        self.place_rotation += delta;
        if let Some(frame) = self.level_state.waypoints.as_ref().and_then(|waypoints| {
            waypoints.selected.and_then(|selected| {
                self.level
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
        }) {
            frame.rotation += delta;
        }
    }

    fn toggle_danger(&mut self) {
        match &mut self.state {
            State::Idle => {
                if let Some(event) = self
                    .selected_light
                    .and_then(|i| self.level.events.get_mut(i.event))
                {
                    if let Event::Light(event) = &mut event.event {
                        event.light.danger = !event.light.danger;
                    }
                }
            }
            State::Waypoints { event, .. } => {
                if let Some(event) = self.level.events.get_mut(*event) {
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
            _ => return,
        }
        self.save_state(default());
    }

    fn cancel(&mut self) {
        match &mut self.state {
            State::Idle => {
                // Cancel selection
                self.execute(LevelAction::DeselectLight);
            }
            State::Movement { .. } | State::Place { .. } => {
                // Cancel creation
                self.state = State::Idle;
            }
            State::Waypoints { state, .. } => {
                // Cancel selection
                match state {
                    WaypointsState::Idle => {
                        if let Some(waypoints) = &mut self.level_state.waypoints {
                            if waypoints.selected.take().is_some() {
                                return;
                            }
                        }
                        self.state = State::Idle
                    }
                    WaypointsState::New => *state = WaypointsState::Idle,
                }
            }
            _ => (),
        }
    }

    fn place_waypoint(&mut self, position: vec2<Coord>) {
        let Some(waypoints) = &self.level_state.waypoints else {
            return;
        };

        let Some(event) = self.level.events.get_mut(waypoints.event) else {
            return;
        };

        let Event::Light(light) = &mut event.event else {
            return;
        };

        let Some(i) = waypoints
            .points
            .iter()
            .position(|point| point.original.is_none())
        else {
            return;
        };

        let mut transform = Transform {
            translation: position,
            rotation: self.place_rotation,
            scale: self.place_scale,
        };
        match i.checked_sub(1) {
            None => {
                // Replace initial
                std::mem::swap(&mut light.light.movement.initial, &mut transform);

                // Extra time for fade in and telegraph
                let time =
                    self.current_beat - light.light.movement.fade_in - light.telegraph.precede_time;
                light.light.movement.key_frames.push_front(MoveFrame {
                    lerp_time: event.beat - time,
                    transform,
                });
                event.beat = time;
            }
            Some(i) => {
                // Insert keyframe
                let last = light.light.movement.timed_positions().nth(i);
                if let Some((_, _, last_time)) = last {
                    let last_time = event.beat + light.telegraph.precede_time + last_time;
                    let lerp_time = self.current_beat - last_time;

                    light.light.movement.key_frames.insert(
                        i,
                        MoveFrame {
                            lerp_time,
                            transform,
                        },
                    );

                    if let Some(next) = light.light.movement.key_frames.get_mut(i + 1) {
                        next.lerp_time -= lerp_time;
                    }
                }
            }
        }
    }
}
