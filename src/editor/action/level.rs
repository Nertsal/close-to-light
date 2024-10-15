use super::*;

// TODO: maybe make every action mostly not depend on the editor state,
// so include LightId in every relevant action
#[derive(Debug, Clone)]
pub enum LevelAction {
    DeleteSelected,
    DeleteSelectedLight,
    DeleteSelectedWaypoint,
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
    DeselectLight,
    SelectLight(LightId),
    SelectWaypoint(WaypointId),
    DeselectWaypoint,
    SetName(String),
    SetSelectedFrame(Transform),
    SetWaypointCurve(LightId, WaypointId, Option<TrajectoryInterpolation>),
    MoveLight(LightId, Change<Time>, Change<vec2<Coord>>),
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
            LevelAction::DeleteSelected => false,
            LevelAction::DeleteSelectedLight => false,
            LevelAction::DeleteSelectedWaypoint => false,
            LevelAction::Undo => false,
            LevelAction::Redo => false,
            LevelAction::Rotate(delta) => *delta == Angle::ZERO,
            LevelAction::ToggleDanger => false,
            LevelAction::ToggleWaypointsView => false,
            LevelAction::Cancel => false,
            LevelAction::StopPlaying => false,
            LevelAction::StartPlaying => false,
            LevelAction::NewLight(_) => false,
            LevelAction::PlaceLight(_) => false,
            LevelAction::NewWaypoint => false,
            LevelAction::PlaceWaypoint(_) => false,
            LevelAction::ScaleLight(delta) => *delta == Coord::ZERO,
            LevelAction::ScaleWaypoint(delta) => *delta == Coord::ZERO,
            LevelAction::ChangeFadeOut(_, delta) => *delta == Coord::ZERO,
            LevelAction::ChangeFadeIn(_, delta) => *delta == Coord::ZERO,
            LevelAction::DeselectLight => false,
            LevelAction::SelectLight(_) => false,
            LevelAction::SelectWaypoint(_) => false,
            LevelAction::DeselectWaypoint => false,
            LevelAction::SetName(_) => false,
            LevelAction::SetSelectedFrame(_) => false,
            LevelAction::SetWaypointCurve(..) => false,
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
            LevelAction::DeleteSelected => {
                if !self.delete_waypoint_selected() {
                    self.delete_light_selected();
                }
            }
            LevelAction::DeleteSelectedLight => {
                self.delete_light_selected();
            }
            LevelAction::DeleteSelectedWaypoint => {
                self.delete_waypoint_selected();
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
            LevelAction::PlaceLight(position) => self.place_light(position),
            LevelAction::NewWaypoint => self.new_waypoint(),
            LevelAction::ScaleLight(delta) => {
                self.place_scale = (self.place_scale + delta).clamp(r32(0.2), r32(2.0));
                self.save_state(HistoryLabel::Scroll);
            }
            LevelAction::ScaleWaypoint(delta) => {
                if let Some(waypoints) = &self.level_state.waypoints {
                    if let Some(selected) = waypoints.selected {
                        if let Some(event) = self.level.events.get_mut(waypoints.light.event) {
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
            LevelAction::SetSelectedFrame(frame) => {
                if let Some(waypoints) = &self.level_state.waypoints {
                    if let Some(selected) = waypoints.selected {
                        if let Some(event) = self.level.events.get_mut(waypoints.light.event) {
                            if let Event::Light(light) = &mut event.event {
                                if let Some(old_frame) =
                                    light.light.movement.get_frame_mut(selected)
                                {
                                    *old_frame = frame;
                                }
                            }
                        }
                    }
                }
            }
            LevelAction::SetWaypointCurve(light, waypoint, curve) => {
                self.set_waypoint_curve(light, waypoint, curve)
            }
            LevelAction::MoveLight(light, time, pos) => self.move_light(light, time, pos),
            LevelAction::MoveWaypoint(light, waypoint, pos) => {
                self.move_waypoint(light, waypoint, pos)
            }
        }
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
                if let Some(curve) = curve {
                    event.light.movement.curve = curve;
                }
            }
            WaypointId::Frame(frame) => {
                let Some(frame) = event.light.movement.key_frames.get_mut(frame) else {
                    return;
                };
                frame.change_curve = curve;
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

    fn rotate(&mut self, delta: Angle<Coord>) {
        self.place_rotation += delta;
        if let Some(frame) = self.level_state.waypoints.as_ref().and_then(|waypoints| {
            waypoints.selected.and_then(|selected| {
                self.level
                    .events
                    .get_mut(waypoints.light.event)
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
            State::Waypoints { light_id, .. } => {
                if let Some(event) = self.level.events.get_mut(light_id.event) {
                    if let Event::Light(event) = &mut event.event {
                        event.light.danger = !event.light.danger;
                    }
                }
            }
            State::Place { danger, .. } => {
                *danger = !*danger;
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

        self.save_state(default());
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
