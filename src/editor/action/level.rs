use super::*;

#[derive(Debug, Clone)]
pub enum LevelAction {
    // Generic actions
    Undo,
    Redo,
    Copy,
    Paste,
    FlushChanges(Option<HistoryLabel>),
    Cancel,
    SetName(String),
    ToggleWaypointsView,
    StopPlaying,
    StartPlaying,
    ScalePlacement(Coord),
    RotatePlacement(Angle<Coord>),
    ScrollTime(Time),
    TimelineZoom(Change<f32>),

    // Light actions
    CopyLight(LightId),
    NewLight(Shape),
    ToggleDangerPlacement,
    PlaceLight(vec2<Coord>),
    DeleteLight(LightId),
    SelectLight(LightId),
    DeselectLight,
    RotateLightAround(LightId, vec2<Coord>, Angle<Coord>),
    FlipHorizontal(LightId, vec2<Coord>),
    FlipVertical(LightId, vec2<Coord>),
    ToggleDanger(LightId),
    ChangeFadeOut(LightId, Change<Time>),
    ChangeFadeIn(LightId, Change<Time>),
    MoveLight(LightId, Change<Time>, Change<vec2<Coord>>),
    HoverLight(LightId),

    // Waypoint actions
    NewWaypoint,
    PlaceWaypoint(vec2<Coord>),
    DeleteWaypoint(LightId, WaypointId),
    SelectWaypoint(WaypointId, bool),
    DeselectWaypoint,
    RotateWaypoint(LightId, WaypointId, Change<Angle<Coord>>),
    ScaleWaypoint(LightId, WaypointId, Change<Coord>),
    SetWaypointFrame(LightId, WaypointId, Transform),
    SetWaypointInterpolation(LightId, WaypointId, MoveInterpolation),
    SetWaypointCurve(LightId, WaypointId, Option<TrajectoryInterpolation>),
    MoveWaypoint(LightId, WaypointId, Change<vec2<Coord>>),
    MoveWaypointTime(LightId, WaypointId, Change<Time>),
}

#[derive(Debug, Clone, Copy)]
pub enum Change<T> {
    Add(T),
    Set(T),
}

impl<T: Sub<Output = T>> Change<T> {
    pub fn into_delta(self, reference_value: T) -> T {
        match self {
            Change::Add(delta) => delta,
            Change::Set(target_value) => target_value.sub(reference_value),
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
            LevelAction::Undo => false,
            LevelAction::Redo => false,
            LevelAction::Copy => false,
            LevelAction::Paste => false,
            LevelAction::FlushChanges(_) => false,
            LevelAction::Cancel => false,
            LevelAction::SetName(_) => false,
            LevelAction::ToggleWaypointsView => false,
            LevelAction::StopPlaying => false,
            LevelAction::StartPlaying => false,
            LevelAction::ScalePlacement(delta) => *delta == Coord::ZERO,
            LevelAction::RotatePlacement(delta) => *delta == Angle::ZERO,
            LevelAction::ScrollTime(delta) => *delta == Time::ZERO,
            LevelAction::TimelineZoom(zoom) => zoom.is_noop(&0.0),

            LevelAction::CopyLight(_) => false,
            LevelAction::NewLight(_) => false,
            LevelAction::ToggleDangerPlacement => false,
            LevelAction::PlaceLight(_) => false,
            LevelAction::DeleteLight(..) => false,
            LevelAction::SelectLight(_) => false,
            LevelAction::DeselectLight => false,
            LevelAction::RotateLightAround(_, _, delta) => *delta == Angle::ZERO,
            LevelAction::FlipHorizontal(_, _) => false,
            LevelAction::FlipVertical(_, _) => false,
            LevelAction::ToggleDanger(..) => false,
            LevelAction::ChangeFadeOut(_, delta) => delta.is_noop(&0),
            LevelAction::ChangeFadeIn(_, delta) => delta.is_noop(&0),
            LevelAction::MoveLight(_, time, position) => {
                time.is_noop(&Time::ZERO) && position.is_noop(&vec2::ZERO)
            }
            LevelAction::HoverLight(_) => false,

            LevelAction::NewWaypoint => false,
            LevelAction::PlaceWaypoint(_) => false,
            LevelAction::DeleteWaypoint(..) => false,
            LevelAction::SelectWaypoint(_, _) => false,
            LevelAction::DeselectWaypoint => false,
            LevelAction::RotateWaypoint(_, _, delta) => delta.is_noop(&Angle::ZERO),
            LevelAction::ScaleWaypoint(_, _, delta) => delta.is_noop(&Coord::ZERO),
            LevelAction::SetWaypointFrame(..) => false,
            LevelAction::SetWaypointInterpolation(..) => false,
            LevelAction::SetWaypointCurve(..) => false,
            LevelAction::MoveWaypoint(_, _, position) => position.is_noop(&vec2::ZERO),
            LevelAction::MoveWaypointTime(_, _, time) => time.is_noop(&Time::ZERO),
        }
    }
}

impl LevelEditor {
    pub fn execute(&mut self, action: LevelAction) {
        if action.is_noop() {
            return;
        }

        // log::trace!("LevelAction::{:?}", action);
        match action {
            LevelAction::Undo => self.undo(),
            LevelAction::Redo => self.redo(),
            LevelAction::Copy => self.copy(),
            LevelAction::Paste => self.paste(),
            LevelAction::FlushChanges(label) => {
                if label.map_or(true, |label| self.history.buffer_label == label) {
                    self.flush_changes()
                }
            }
            LevelAction::Cancel => self.cancel(),
            LevelAction::SetName(name) => self.name = name,
            LevelAction::ToggleWaypointsView => self.view_waypoints(),
            LevelAction::StopPlaying => {
                if let State::Playing {
                    start_time,
                    start_target_time,
                    old_state,
                } = &self.state
                {
                    self.current_time.snap_to(*start_time);
                    self.current_time
                        .scroll_time(Change::Set(*start_target_time));
                    self.state = *old_state.clone();
                    self.context.music.stop();
                }
            }
            LevelAction::StartPlaying => {
                self.state = State::Playing {
                    start_time: self.current_time.value,
                    start_target_time: self.current_time.target,
                    old_state: Box::new(self.state.clone()),
                };
                self.real_time = time_to_seconds(self.current_time.value);
                self.context.music.play_from(
                    &self.static_level.group.music,
                    time::Duration::from_secs_f64(self.real_time.as_f32().into()),
                );
            }
            LevelAction::ScalePlacement(delta) => {
                self.place_scale = (self.place_scale + delta).clamp(r32(0.2), r32(2.0));
            }
            LevelAction::RotatePlacement(delta) => {
                self.place_rotation += delta;
            }
            LevelAction::ScrollTime(delta) => {
                self.scroll_time(delta);
            }
            LevelAction::TimelineZoom(change) => {
                let mut zoom = self.timeline_zoom.target.as_f32();
                zoom = match change {
                    Change::Add(delta) => zoom * 2.0.powf(delta),
                    Change::Set(zoom) => zoom,
                };
                let target = zoom.clamp(16.0.recip(), 2.0);
                self.timeline_zoom.target = r32(target);
            }

            LevelAction::CopyLight(id) => {
                if let Some(event) = self.level.events.get(id.event) {
                    if let Event::Light(light) = &event.event {
                        self.clipboard.copy(ClipboardItem::Light(light.clone()));
                    }
                }
            }
            LevelAction::NewLight(shape) => {
                self.execute(LevelAction::DeselectLight);
                self.state = State::Place {
                    shape,
                    danger: false,
                };
            }
            LevelAction::ToggleDangerPlacement => {
                if let State::Place { danger, .. } = &mut self.state {
                    *danger = !*danger;
                }
            }
            LevelAction::PlaceLight(position) => self.place_light(position),
            LevelAction::DeleteLight(light) => self.delete_light(light),
            LevelAction::SelectLight(id) => self.select_light(id),
            LevelAction::DeselectLight => {
                self.execute(LevelAction::DeselectWaypoint);
                self.selected_light = None;
            }
            LevelAction::RotateLightAround(light, anchor, delta) => {
                self.modify_movement(light, |movement| movement.rotate_around(anchor, delta))
            }
            LevelAction::FlipHorizontal(light, anchor) => {
                self.modify_movement(light, |movement| movement.flip_horizontal(anchor))
            }
            LevelAction::FlipVertical(light, anchor) => {
                self.modify_movement(light, |movement| movement.flip_vertical(anchor))
            }
            LevelAction::ToggleDanger(light) => self.toggle_danger(light),
            LevelAction::ChangeFadeOut(id, change) => {
                if let Some(event) = self.level.events.get_mut(id.event) {
                    if let Event::Light(light) = &mut event.event {
                        let movement = &mut light.movement;
                        let mut value = movement.fade_out;
                        change.apply(&mut value);
                        movement.change_fade_out(value);
                        self.save_state(HistoryLabel::FadeOut(id));
                    }
                }
            }
            LevelAction::ChangeFadeIn(id, change) => {
                if let Some(event) = self.level.events.get_mut(id.event) {
                    if let Event::Light(light) = &mut event.event {
                        let movement = &mut light.movement;
                        let from = movement.fade_in;
                        let change = change.into_delta(from);
                        movement.change_fade_in(movement.fade_in + change);
                        event.time -= movement.fade_in - from;
                        self.save_state(HistoryLabel::FadeIn(id));
                    }
                }
            }
            LevelAction::MoveLight(light, time, pos) => self.move_light(light, time, pos),
            LevelAction::HoverLight(light) => {
                self.timeline_light_hover = Some(light);
            }

            LevelAction::NewWaypoint => self.new_waypoint(),
            LevelAction::PlaceWaypoint(position) => self.place_waypoint(position),
            LevelAction::DeleteWaypoint(light, waypoint) => self.delete_waypoint(light, waypoint),
            LevelAction::SelectWaypoint(id, move_time) => self.select_waypoint(id, move_time),
            LevelAction::DeselectWaypoint => {
                if let Some(waypoints) = &mut self.level_state.waypoints {
                    waypoints.selected = None;
                }
            }
            LevelAction::RotateWaypoint(light, waypoint, change) => {
                self.rotate(light, waypoint, change)
            }
            LevelAction::ScaleWaypoint(light_id, waypoint_id, change) => {
                if let Some(event) = self.level.events.get_mut(light_id.event) {
                    if let Event::Light(light) = &mut event.event {
                        if let Some(frame) = light.movement.get_frame_mut(waypoint_id) {
                            change.apply(&mut frame.scale);
                            frame.scale = frame.scale.clamp(r32(0.0), r32(10.0));
                            self.save_state(HistoryLabel::Scale(light_id, waypoint_id));
                        }
                    }
                }
            }
            LevelAction::SetWaypointFrame(light_id, waypoint_id, frame) => {
                if let Some(event) = self.level.events.get_mut(light_id.event) {
                    if let Event::Light(light) = &mut event.event {
                        if let Some(old_frame) = light.movement.get_frame_mut(waypoint_id) {
                            *old_frame = frame;
                        }
                    }
                }
            }
            LevelAction::SetWaypointInterpolation(light, waypoint, interpolation) => {
                self.set_waypoint_interpolation(light, waypoint, interpolation)
            }
            LevelAction::SetWaypointCurve(light, waypoint, curve) => {
                self.set_waypoint_curve(light, waypoint, curve)
            }
            LevelAction::MoveWaypoint(light, waypoint, pos) => {
                self.move_waypoint(light, waypoint, pos)
            }
            LevelAction::MoveWaypointTime(light, waypoint, time) => {
                self.move_waypoint_time(light, waypoint, time)
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

        change_time.apply(&mut timed_event.time);

        let change_pos = Change::Add(change_pos.into_delta(event.movement.initial.translation));
        change_pos.apply(&mut event.movement.initial.translation);
        for frame in &mut event.movement.key_frames {
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

        let Some(frame) = event.movement.get_frame_mut(waypoint_id) else {
            return;
        };

        change_pos.apply(&mut frame.translation);
        self.save_state(HistoryLabel::MoveWaypoint(light_id, waypoint_id));
    }

    fn move_waypoint_time(
        &mut self,
        light_id: LightId,
        waypoint_id: WaypointId,
        change_time: Change<Time>,
    ) {
        let Some(timed_event) = self.level.events.get_mut(light_id.event) else {
            return;
        };

        let Event::Light(event) = &mut timed_event.event else {
            return;
        };

        // Update time
        let mut frames: Vec<_> = event
            .movement
            .timed_positions()
            .map(|(id, transform, mut time)| {
                time += timed_event.time;
                if id == waypoint_id {
                    change_time.apply(&mut time);
                }
                (id, transform, time)
            })
            .collect();

        // Sort frames by absolute time
        frames.sort_by_key(|(_, _, time)| *time);

        // Check if some frames have same time - invalid reorder
        if frames.iter().tuple_windows().any(|(a, b)| a.2 == b.2) {
            // Invalid reorder
            return;
        }

        // Reorder frames to fit the new time
        let mut frames = frames.into_iter();
        let mut fixed_movement = event.movement.clone();
        let mut last_time = Time::ZERO;
        let mut fix_selection = true;

        // Initial frame is treated specially
        if let Some((original_id, transform, time)) = frames.next() {
            let fixed_id = WaypointId::Initial;
            if fix_selection && self.selected_light == Some(light_id) {
                if let Some(waypoints) = &mut self.level_state.waypoints {
                    if waypoints.selected == Some(original_id) {
                        waypoints.selected = Some(fixed_id);
                        fix_selection = false;
                    }
                }
            }

            // TODO: config option to keep curves at waypoints
            // let (interpolation, curve) = event
            //     .movement
            //     .get_interpolation(fixed_id)
            //     .expect("invalid waypoint id when fixing");
            fixed_movement.initial = transform;
            // fixed_movement.interpolation = interpolation;
            // fixed_movement.curve = curve.unwrap_or_default();
            timed_event.time = time - event.movement.fade_in;
            last_time = time;
        }

        // Update all other frames
        for (i, (original_id, transform, time)) in frames.enumerate() {
            let fixed_id = WaypointId::Frame(i);
            if fix_selection && self.selected_light == Some(light_id) {
                if let Some(waypoints) = &mut self.level_state.waypoints {
                    if waypoints.selected == Some(original_id) {
                        waypoints.selected = Some(fixed_id);
                        fix_selection = false;
                    }
                }
            }

            // TODO: config option to keep curves at waypoints
            // let (interpolation, curve) = event
            //     .movement
            //     .get_interpolation(fixed_id)
            //     .expect("invalid waypoint id when fixing");
            let fixed_frame = fixed_movement
                .key_frames
                .get_mut(i)
                .expect("invalid waypoint index when fixing");
            fixed_frame.transform = transform;
            // fixed_frame.interpolation = interpolation;
            // fixed_frame.curve = curve;
            fixed_frame.lerp_time = time - last_time;
            last_time = time;
        }

        event.movement = fixed_movement;

        self.save_state(HistoryLabel::MoveWaypointTime(light_id, waypoint_id));
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
                event.movement.curve = curve.unwrap_or_default();
            }
            WaypointId::Frame(frame) => {
                let Some(frame) = event.movement.key_frames.get_mut(frame) else {
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
                event.movement.interpolation = interpolation;
            }
            WaypointId::Frame(frame) => {
                let Some(frame) = event.movement.key_frames.get_mut(frame) else {
                    return;
                };
                frame.interpolation = interpolation;
                self.save_state(default());
            }
        }
    }

    fn select_light(&mut self, light_id: LightId) {
        self.level_state.waypoints = None;
        self.state = State::Idle;
        self.selected_light = Some(light_id);
    }

    fn modify_movement(&mut self, light_id: LightId, f: impl FnOnce(&mut Movement)) {
        let Some(event) = self.level.events.get_mut(light_id.event) else {
            return;
        };

        let Event::Light(light) = &mut event.event else {
            return;
        };

        f(&mut light.movement);
    }

    fn select_waypoint(&mut self, waypoint_id: WaypointId, move_time: bool) {
        let Some(light_id) = self.selected_light else {
            return;
        };

        let Some(waypoint_time) =
            self.level
                .events
                .get(light_id.event)
                .and_then(|event| match &event.event {
                    Event::Light(light) => light
                        .movement
                        .get_time(waypoint_id)
                        .map(|time| event.time + time),
                    _ => None,
                })
        else {
            // Invalid waypoint id
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

        if move_time {
            self.execute(LevelAction::ScrollTime(
                waypoint_time - self.current_time.target,
            ));
        }
    }

    fn rotate(&mut self, light_id: LightId, waypoint_id: WaypointId, change: Change<Angle<Coord>>) {
        change.apply(&mut self.place_rotation);

        let Some(event) = self.level.events.get_mut(light_id.event) else {
            return;
        };
        let Event::Light(event) = &mut event.event else {
            return;
        };
        let Some(frame) = event.movement.get_frame_mut(waypoint_id) else {
            return;
        };

        change.apply(&mut frame.rotation);
        self.save_state(HistoryLabel::Rotate(light_id, waypoint_id));
    }

    fn toggle_danger(&mut self, light_id: LightId) {
        if let Some(event) = self.level.events.get_mut(light_id.event) {
            if let Event::Light(event) = &mut event.event {
                event.danger = !event.danger;
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

        let start_beat = self.current_time.target;
        let timing = self.level.timing.get_timing(start_beat);
        let movement = Movement::new(
            seconds_to_time(timing.beat_time),
            Transform {
                translation: position,
                rotation: self.place_rotation.normalized_2pi(),
                scale: self.place_scale,
            },
        );

        let light = LightEvent {
            shape,
            movement,
            danger,
        };

        let beat = start_beat - light.movement.fade_in; // extra time for the fade in and telegraph
        let event = commit_light(light.clone());
        let event = TimedEvent {
            time: beat,
            event: Event::Light(event),
        };

        let event_i = self.level.events.len();
        self.level.events.push(event);

        self.selected_light = Some(LightId { event: event_i });
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
        let mut interpolation = MoveInterpolation::default(); // TODO: use the same as other waypoints
        let mut change_curve = None;
        match i.checked_sub(1) {
            None => {
                // Replace initial
                std::mem::swap(&mut light.movement.initial, &mut transform);
                std::mem::swap(&mut light.movement.interpolation, &mut interpolation);
                change_curve = Some(light.movement.curve);
                light.movement.curve = TrajectoryInterpolation::default();

                // NOTE: target to make sure it is snapped to the beat
                // assume time interpolation doesn't take long, so not visually weird
                let time = self.current_time.target - light.movement.fade_in; // Extra time for fade in
                light.movement.key_frames.push_front(MoveFrame {
                    lerp_time: event.time - time,
                    interpolation,
                    change_curve,
                    transform,
                });
                event.time = time;
            }
            Some(i) => {
                // Insert keyframe
                let last = light.movement.timed_positions().nth(i);
                if let Some((_, _, last_time)) = last {
                    let last_time = event.time + last_time;
                    // NOTE: target to make sure it is snapped to the beat
                    // assume time interpolation doesn't take long, so not visually weird
                    let lerp_time = self.current_time.target - last_time;

                    light.movement.key_frames.insert(
                        i,
                        MoveFrame {
                            lerp_time,
                            interpolation,
                            change_curve,
                            transform,
                        },
                    );

                    if let Some(next) = light.movement.key_frames.get_mut(i + 1) {
                        next.lerp_time -= lerp_time;
                    }
                }
            }
        }
    }
}
