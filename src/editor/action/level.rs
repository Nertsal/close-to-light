use super::*;

#[derive(Debug, Clone)]
pub enum LevelAction {
    // Generic actions
    Undo,
    Redo,
    Cancel,
    SetName(String),
    ToggleWaypointsView,
    StopPlaying,
    StartPlaying,

    // Light actions
    NewLight(Shape),
    ScalePlacement(Coord),
    ToggleDangerPlacement,
    PlaceLight(vec2<Coord>),
    DeleteLight(LightId),
    SelectLight(LightId),
    DeselectLight,
    ToggleDanger(LightId),
    ChangeFadeOut(LightId, Time),
    ChangeFadeIn(LightId, Time),
    MoveLight(LightId, Change<Time>, Change<vec2<Coord>>),

    // Waypoint actions
    NewWaypoint,
    PlaceWaypoint(vec2<Coord>),
    DeleteWaypoint(LightId, WaypointId),
    SelectWaypoint(WaypointId),
    DeselectWaypoint,
    RotateWaypoint(LightId, WaypointId, Angle<Coord>),
    ScaleWaypoint(LightId, WaypointId, Coord),
    SetWaypointFrame(LightId, WaypointId, Transform),
    SetWaypointInterpolation(LightId, WaypointId, MoveInterpolation),
    SetWaypointCurve(LightId, WaypointId, Option<TrajectoryInterpolation>),
    MoveWaypoint(LightId, WaypointId, Change<vec2<Coord>>),
}

#[derive(Debug, Clone, Copy)]
pub enum Change<T> {
    Add(T),
    Set(T),
}

impl<T: Sub<Output = T>> Change<T> {
    pub fn into_delta(self, reference_value: T) -> Self {
        match self {
            Change::Add(_) => self,
            Change::Set(target_value) => Change::Add(target_value.sub(reference_value)),
        }
    }
}

impl<T: Add<Output = T> + Copy> Change<T> {
    pub fn apply(&self, value: &mut T) {
        *value = match *self {
            Change::Add(delta) => value.add(delta),
            Change::Set(value) => value,
        };
    }
}

impl<T: PartialEq> Change<T> {
    pub fn is_noop(&self, zero_delta: &T) -> bool {
        match self {
            Change::Add(delta) => delta == zero_delta,
            Change::Set(_) => false,
        }
    }
}

impl LevelAction {
    /// Whether the action has no effect.
    pub fn is_noop(&self) -> bool {
        match self {
            LevelAction::DeleteLight(..) => false,
            LevelAction::DeleteWaypoint(..) => false,
            LevelAction::Undo => false,
            LevelAction::Redo => false,
            LevelAction::RotateWaypoint(_, _, delta) => *delta == Angle::ZERO,
            LevelAction::ToggleDanger(..) => false,
            LevelAction::ToggleWaypointsView => false,
            LevelAction::Cancel => false,
            LevelAction::StopPlaying => false,
            LevelAction::StartPlaying => false,
            LevelAction::NewLight(_) => false,
            LevelAction::ScalePlacement(delta) => *delta == Coord::ZERO,
            LevelAction::ToggleDangerPlacement => false,
            LevelAction::PlaceLight(_) => false,
            LevelAction::NewWaypoint => false,
            LevelAction::PlaceWaypoint(_) => false,
            LevelAction::ScaleWaypoint(_, _, delta) => *delta == Coord::ZERO,
            LevelAction::ChangeFadeOut(_, delta) => *delta == Coord::ZERO,
            LevelAction::ChangeFadeIn(_, delta) => *delta == Coord::ZERO,
            LevelAction::DeselectLight => false,
            LevelAction::SelectLight(_) => false,
            LevelAction::SelectWaypoint(_) => false,
            LevelAction::DeselectWaypoint => false,
            LevelAction::SetName(_) => false,
            LevelAction::SetWaypointFrame(..) => false,
            LevelAction::SetWaypointCurve(..) => false,
            LevelAction::SetWaypointInterpolation(..) => false,
            LevelAction::MoveLight(_, time, position) => {
                time.is_noop(&Time::ZERO) && position.is_noop(&vec2::ZERO)
            }
            LevelAction::MoveWaypoint(_, _, position) => position.is_noop(&vec2::ZERO),
        }
    }
}

impl LevelEditor {
    pub fn execute(&mut self, action: LevelAction) {
        if action.is_noop() {
            return;
        }

        // log::debug!("action LevelAction::{:?}", action);
        match action {
            LevelAction::DeleteLight(light) => self.delete_light(light),
            LevelAction::DeleteWaypoint(light, waypoint) => self.delete_waypoint(light, waypoint),
            LevelAction::Undo => self.undo(),
            LevelAction::Redo => self.redo(),
            LevelAction::RotateWaypoint(light, waypoint, delta) => {
                self.rotate(light, waypoint, delta)
            }
            LevelAction::ToggleDanger(light) => self.toggle_danger(light),
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
                self.execute(LevelAction::DeselectLight);
                self.state = State::Place {
                    shape,
                    danger: false,
                };
            }
            LevelAction::ScalePlacement(delta) => {
                self.place_scale = (self.place_scale + delta).clamp(r32(0.2), r32(2.0));
            }
            LevelAction::ToggleDangerPlacement => {
                if let State::Place { danger, .. } = &mut self.state {
                    *danger = !*danger;
                }
            }
            LevelAction::PlaceLight(position) => self.place_light(position),
            LevelAction::NewWaypoint => self.new_waypoint(),
            LevelAction::ScaleWaypoint(light_id, waypoint_id, delta) => {
                if let Some(event) = self.level.events.get_mut(light_id.event) {
                    if let Event::Light(light) = &mut event.event {
                        if let Some(frame) = light.light.movement.get_frame_mut(waypoint_id) {
                            frame.scale = (frame.scale + delta).clamp(r32(0.2), r32(2.0));
                            self.save_state(HistoryLabel::Scale(light_id, waypoint_id));
                        }
                    }
                }
            }
            LevelAction::ChangeFadeOut(id, change) => {
                if let Some(event) = self.level.events.get_mut(id.event) {
                    if let Event::Light(light) = &mut event.event {
                        let movement = &mut light.light.movement;
                        movement.change_fade_out(movement.fade_out + change);
                        self.save_state(HistoryLabel::FadeOut(id));
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
                        self.save_state(HistoryLabel::FadeIn(id));
                    }
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
            LevelAction::SelectWaypoint(id) => self.select_waypoint(id),
            LevelAction::DeselectWaypoint => {
                if let Some(waypoints) = &mut self.level_state.waypoints {
                    waypoints.selected = None;
                }
            }
            LevelAction::PlaceWaypoint(position) => self.place_waypoint(position),
            LevelAction::SetName(name) => self.name = name,
            LevelAction::SetWaypointFrame(light_id, waypoint_id, frame) => {
                if let Some(event) = self.level.events.get_mut(light_id.event) {
                    if let Event::Light(light) = &mut event.event {
                        if let Some(old_frame) = light.light.movement.get_frame_mut(waypoint_id) {
                            *old_frame = frame;
                        }
                    }
                }
            }
            LevelAction::SetWaypointCurve(light, waypoint, curve) => {
                self.set_waypoint_curve(light, waypoint, curve)
            }
            LevelAction::SetWaypointInterpolation(light, waypoint, interpolation) => {
                self.set_waypoint_interpolation(light, waypoint, interpolation)
            }
            LevelAction::MoveLight(light, time, pos) => self.move_light(light, time, pos),
            LevelAction::MoveWaypoint(light, waypoint, pos) => {
                self.move_waypoint(light, waypoint, pos)
            }
        }

        // In case some action forgot to save the state,
        // we save it with the default label
        self.save_state(default());
    }

    fn move_light(
        &mut self,
        light_id: LightId,
        change_time: Change<Time>,
        change_pos: Change<vec2<Coord>>,
    ) {
        let Some(timed_event) = self.level.events.get_mut(light_id.event) else {
            return;
        };

        let Event::Light(event) = &mut timed_event.event else {
            return;
        };

        change_time.apply(&mut timed_event.beat);

        let change_pos = change_pos.into_delta(event.light.movement.initial.translation);
        change_pos.apply(&mut event.light.movement.initial.translation);
        for frame in &mut event.light.movement.key_frames {
            change_pos.apply(&mut frame.transform.translation);
        }

        self.save_state(HistoryLabel::MoveLight(light_id));
    }

    fn move_waypoint(
        &mut self,
        light_id: LightId,
        waypoint_id: WaypointId,
        change_pos: Change<vec2<Coord>>,
    ) {
        let Some(timed_event) = self.level.events.get_mut(light_id.event) else {
            return;
        };

        let Event::Light(event) = &mut timed_event.event else {
            return;
        };

        let Some(frame) = event.light.movement.get_frame_mut(waypoint_id) else {
            return;
        };

        change_pos.apply(&mut frame.translation);
        self.save_state(HistoryLabel::MoveWaypoint(light_id, waypoint_id));
    }

    fn set_waypoint_curve(
        &mut self,
        light_id: LightId,
        waypoint_id: WaypointId,
        curve: Option<TrajectoryInterpolation>,
    ) {
        let Some(timed_event) = self.level.events.get_mut(light_id.event) else {
            return;
        };

        let Event::Light(event) = &mut timed_event.event else {
            return;
        };

        match waypoint_id {
            WaypointId::Initial => {
                // Invalid
            }
            WaypointId::Frame(frame) => {
                let Some(frame) = event.light.movement.key_frames.get_mut(frame) else {
                    return;
                };
                frame.change_curve = curve;
                self.save_state(default());
            }
        }
    }

    fn set_waypoint_interpolation(
        &mut self,
        light_id: LightId,
        waypoint_id: WaypointId,
        interpolation: MoveInterpolation,
    ) {
        let Some(timed_event) = self.level.events.get_mut(light_id.event) else {
            return;
        };

        let Event::Light(event) = &mut timed_event.event else {
            return;
        };

        match waypoint_id {
            WaypointId::Initial => {
                // Invalid
            }
            WaypointId::Frame(frame) => {
                let Some(frame) = event.light.movement.key_frames.get_mut(frame) else {
                    return;
                };
                frame.interpolation = interpolation;
                self.save_state(default());
            }
        }
    }

    fn select_waypoint(&mut self, waypoint_id: WaypointId) {
        let Some(light_id) = self.selected_light else {
            return;
        };

        // TODO: validation check
        if let Some(waypoints) = &mut self.level_state.waypoints {
            waypoints.selected = Some(waypoint_id);
        } else {
            self.state = State::Waypoints {
                light_id,
                state: WaypointsState::Idle,
            };
            self.level_state.waypoints = Some(Waypoints {
                light: light_id,
                points: Vec::new(),
                hovered: None,
                selected: Some(waypoint_id),
            });
        }
    }

    fn rotate(&mut self, light_id: LightId, waypoint_id: WaypointId, delta: Angle<Coord>) {
        self.place_rotation += delta;

        let Some(event) = self.level.events.get_mut(light_id.event) else {
            return;
        };
        let Event::Light(event) = &mut event.event else {
            return;
        };
        let Some(frame) = event.light.movement.get_frame_mut(waypoint_id) else {
            return;
        };

        frame.rotation += delta;
        self.save_state(HistoryLabel::Rotate(light_id, waypoint_id));
    }

    fn toggle_danger(&mut self, light_id: LightId) {
        if let Some(event) = self.level.events.get_mut(light_id.event) {
            if let Event::Light(event) = &mut event.event {
                event.light.danger = !event.light.danger;
            }
        }
    }

    fn cancel(&mut self) {
        match &mut self.state {
            State::Idle => {
                // Cancel selection
                self.execute(LevelAction::DeselectLight);
            }
            State::Place { .. } => {
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

    fn place_light(&mut self, position: vec2<Coord>) {
        let State::Place { shape, danger } = self.state else {
            return;
        };

        let movement = Movement {
            initial: Transform {
                translation: position,
                rotation: self.place_rotation.normalized_2pi(),
                scale: self.place_scale,
            },
            ..default()
        };
        let telegraph = Telegraph::default();

        let light = LightEvent {
            light: LightSerde {
                shape,
                movement,
                danger,
            },
            telegraph,
        };

        // extra time for the fade in and telegraph
        let start_beat = self.current_beat;
        let beat = start_beat - light.light.movement.fade_in - light.telegraph.precede_time;
        let event = commit_light(light.clone());
        let event = TimedEvent {
            beat,
            event: Event::Light(event),
        };

        let event_i = self.level.events.len();
        self.level.events.push(event);

        self.state = State::Waypoints {
            light_id: LightId { event: event_i },
            state: WaypointsState::New,
        };
    }

    fn place_waypoint(&mut self, position: vec2<Coord>) {
        let Some(waypoints) = &self.level_state.waypoints else {
            return;
        };

        let Some(event) = self.level.events.get_mut(waypoints.light.event) else {
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
                    interpolation: MoveInterpolation::default(), // TODO: use the same as other waypoints
                    change_curve: None,
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
                            interpolation: MoveInterpolation::default(), // TODO: use the same as other waypoints
                            change_curve: None,
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
