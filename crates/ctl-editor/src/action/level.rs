use super::*;

#[derive(Debug, Clone)]
pub enum LevelAction {
    // Generic actions
    List(Option<HistoryLabel>, Vec<LevelAction>), // TODO: smallvec
    Undo,
    Redo,
    Copy,
    CopySelection(Selection),
    SetSelection(Selection),
    Paste,
    FlushChanges(Option<HistoryLabel>),
    Cancel,
    SetName(String),
    ToggleWaypointsView,
    ScalePlacement(Change<Coord>),
    RotatePlacement(Angle<Coord>),
    ScrollTime(Time),
    TimelineZoom(Change<f32>),
    CameraPan(Change<vec2<f32>>),
    TimingUpdate(usize, FloatTime),
    /// Selected a shape, but the specific action is up to interpretation.
    /// If there is a light selected, changes its shape; otherwise creates a new light.
    Shape(Shape),
    Deselect,

    // General event
    SelectEvent(usize),
    DeleteEvent(usize),
    MoveEvent(usize, Change<Time>),

    // Vfx
    NewRgbSplit(Time),
    NewPaletteSwap(Time),
    NewCameraShake(Time),
    ChangeEffectDuration(usize, Change<Time>),
    ChangeCameraShakeIntensity(usize, Change<R32>),

    // Light actions
    NewLight(Shape),
    ToggleDangerPlacement,
    PlaceLight(vec2<Coord>),
    DeleteLight(LightId),
    SelectLight(SelectMode, Vec<LightId>), // TODO: smallvec
    ChangeShape(LightId, Shape),
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
    SetWaypointInterpolation(LightId, WaypointId, MoveInterpolation),
    SetWaypointCurve(LightId, WaypointId, Option<TrajectoryInterpolation>),
    MoveWaypoint(LightId, WaypointId, Change<Time>, Change<vec2<Coord>>),
    ChangeHollow(LightId, WaypointId, Change<R32>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectMode {
    Add,
    Remove,
    Set,
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
    pub fn list(iter: impl IntoIterator<Item = Self>) -> Self {
        Self::List(None, iter.into_iter().collect())
    }

    pub fn list_with(label: HistoryLabel, iter: impl IntoIterator<Item = Self>) -> Self {
        Self::List(Some(label), iter.into_iter().collect())
    }

    /// Whether the action has no effect.
    pub fn is_noop(&self) -> bool {
        match self {
            LevelAction::List(_, list) => list.iter().all(LevelAction::is_noop),
            LevelAction::Undo => false,
            LevelAction::Redo => false,
            LevelAction::Copy => false,
            LevelAction::CopySelection(_) => false,
            LevelAction::SetSelection(_) => false,
            LevelAction::Paste => false,
            LevelAction::FlushChanges(_) => false,
            LevelAction::Cancel => false,
            LevelAction::SetName(_) => false,
            LevelAction::ToggleWaypointsView => false,
            LevelAction::ScalePlacement(delta) => delta.is_noop(&Coord::ZERO),
            LevelAction::RotatePlacement(delta) => *delta == Angle::ZERO,
            LevelAction::ScrollTime(delta) => *delta == Time::ZERO,
            LevelAction::TimelineZoom(zoom) => zoom.is_noop(&0.0),
            LevelAction::CameraPan(delta) => delta.is_noop(&vec2::ZERO),
            LevelAction::TimingUpdate(..) => false,
            LevelAction::Shape(..) => false,

            LevelAction::SelectEvent(_) => false,
            LevelAction::DeleteEvent(_) => false,
            LevelAction::MoveEvent(_, delta) => delta.is_noop(&0),

            LevelAction::NewRgbSplit(_) => false,
            LevelAction::NewPaletteSwap(_) => false,
            LevelAction::NewCameraShake(_) => false,
            LevelAction::ChangeEffectDuration(_, delta) => delta.is_noop(&0),
            LevelAction::ChangeCameraShakeIntensity(_, delta) => delta.is_noop(&R32::ZERO),

            LevelAction::NewLight(_) => false,
            LevelAction::ToggleDangerPlacement => false,
            LevelAction::PlaceLight(_) => false,
            LevelAction::DeleteLight(..) => false,
            LevelAction::SelectLight(mode, lights) => {
                matches!(mode, SelectMode::Add | SelectMode::Remove) && lights.is_empty()
            }
            LevelAction::Deselect => false,
            LevelAction::ChangeShape(_, _) => false,
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
            LevelAction::SetWaypointInterpolation(..) => false,
            LevelAction::SetWaypointCurve(..) => false,
            LevelAction::MoveWaypoint(_, _, time, position) => {
                time.is_noop(&Time::ZERO) && position.is_noop(&vec2::ZERO)
            }
            LevelAction::ChangeHollow(_, _, delta) => delta.is_noop(&R32::ZERO),
        }
    }
}

impl LevelEditor {
    pub fn execute(&mut self, action: LevelAction, mut drag: Option<&mut Drag>) {
        if action.is_noop() {
            return;
        }

        log::trace!("LevelAction::{action:?}");
        match action {
            LevelAction::List(label, list) => {
                self.start_merge_changes(label);
                for action in list {
                    // NOTE: Reborrow with as_deref_mut because Option<&mut T> is silly
                    self.execute(action, drag.as_deref_mut());
                }
                self.flush_changes(label);
                return;
            }
            LevelAction::Undo => {
                self.undo();
                return;
            }
            LevelAction::Redo => {
                self.redo();
                return;
            }
            LevelAction::Copy => self.copy(),
            LevelAction::CopySelection(selection) => match selection {
                Selection::Empty => self.clipboard.clear(),
                Selection::Lights(lights) => {
                    let lights = lights
                        .into_iter()
                        .flat_map(|id| self.level.events.get(id.event).cloned())
                        .collect();
                    self.clipboard
                        .copy(ClipboardItem::Events(self.current_time.target, lights));
                }
                Selection::Event(index) => {
                    let events = self.level.events.get(index).cloned().into_iter().collect();
                    self.clipboard
                        .copy(ClipboardItem::Events(self.current_time.target, events));
                }
            },
            LevelAction::SetSelection(selection) => {
                self.selection = selection;
            }
            LevelAction::Paste => self.paste(),
            LevelAction::FlushChanges(label) => {
                if label.is_none_or(|label| self.history.buffer_label == label) {
                    self.flush_changes(None)
                }
            }
            LevelAction::Cancel => self.cancel(),
            LevelAction::SetName(name) => self.name = name,
            LevelAction::ToggleWaypointsView => self.view_waypoints(),
            LevelAction::ScalePlacement(delta) => {
                delta.apply(&mut self.place_scale);
                self.place_scale = self.place_scale.clamp(r32(0.25), r32(2.0));
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
            LevelAction::CameraPan(delta) => {
                delta.apply(&mut self.model.camera.center);
            }
            LevelAction::TimingUpdate(point, beat_time) => {
                if let Some(point) = self.level.timing.points.get_mut(point) {
                    point.beat_time = beat_time;
                }
            }
            LevelAction::Shape(shape) => {
                if !self.selection.is_empty() {
                    // Change shape of the selected light
                    if let Selection::Lights(ids) = &self.selection {
                        self.execute(
                            LevelAction::list(
                                ids.iter()
                                    .copied()
                                    .map(|id| LevelAction::ChangeShape(id, shape)),
                            ),
                            drag,
                        );
                    }
                } else {
                    // New light
                    self.execute(LevelAction::NewLight(shape), drag);
                }
            }

            LevelAction::SelectEvent(index) => {
                if self.level.events.get(index).is_some() {
                    self.selection = Selection::Event(index);
                }
            }
            LevelAction::DeleteEvent(index) => {
                if self.level.events.get(index).is_some() {
                    self.execute(LevelAction::Deselect, drag);
                    self.level.events.swap_remove(index);
                }
            }
            LevelAction::MoveEvent(index, change) => {
                if let Some(event) = self.level.events.get_mut(index) {
                    change.apply(&mut event.time);
                    self.save_state(HistoryLabel::MoveEvent(index));
                }
            }

            LevelAction::NewRgbSplit(duration) => {
                self.execute(LevelAction::Deselect, drag);
                self.level.events.push(TimedEvent {
                    time: self.current_time.target,
                    event: Event::Effect(EffectEvent::RgbSplit(duration)),
                });
            }
            LevelAction::NewCameraShake(duration) => {
                self.execute(LevelAction::Deselect, drag);
                self.level.events.push(TimedEvent {
                    time: self.current_time.target,
                    event: Event::Effect(EffectEvent::CameraShake(duration, r32(0.25))),
                });
            }
            LevelAction::NewPaletteSwap(duration) => {
                self.execute(LevelAction::Deselect, drag);
                self.level.events.push(TimedEvent {
                    time: self.current_time.target,
                    event: Event::Effect(EffectEvent::PaletteSwap(duration)),
                });
            }
            LevelAction::ChangeEffectDuration(index, change) => {
                if let Some(event) = self.level.events.get_mut(index)
                    && let Event::Effect(effect) = &mut event.event
                {
                    let duration = match effect {
                        EffectEvent::PaletteSwap(duration)
                        | EffectEvent::RgbSplit(duration)
                        | EffectEvent::CameraShake(duration, _) => duration,
                    };
                    change.apply(duration);
                    self.save_state(HistoryLabel::EventDuration(index));
                }
            }
            LevelAction::ChangeCameraShakeIntensity(index, change) => {
                if let Some(event) = self.level.events.get_mut(index)
                    && let Event::Effect(EffectEvent::CameraShake(_, intensity)) = &mut event.event
                {
                    change.apply(intensity);
                    self.save_state(HistoryLabel::CameraShakeIntensity(index));
                }
            }

            LevelAction::NewLight(shape) => {
                self.execute(LevelAction::Deselect, drag);
                self.state = EditingState::Place {
                    shape,
                    danger: false,
                };
            }
            LevelAction::ToggleDangerPlacement => {
                if let EditingState::Place { danger, .. } = &mut self.state {
                    *danger = !*danger;
                }
            }
            LevelAction::PlaceLight(position) => self.place_light(position),
            LevelAction::DeleteLight(light) => self.delete_light(light),
            LevelAction::SelectLight(mode, ids) => self.select_light(mode, ids),
            LevelAction::Deselect => {
                self.execute(LevelAction::DeselectWaypoint, drag);
                self.selection.clear();
            }
            LevelAction::ChangeShape(id, shape) => {
                if let Some(event) = self.level.events.get_mut(id.event)
                    && let Event::Light(light) = &mut event.event
                {
                    light.shape = shape;
                    self.save_state(HistoryLabel::Unknown);
                }
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
                if let Some(event) = self.level.events.get_mut(id.event)
                    && let Event::Light(light) = &mut event.event
                {
                    let movement = &mut light.movement;
                    let mut value = movement.get_fade_out();
                    change.apply(&mut value);
                    movement.change_fade_out(value);
                    self.save_state(HistoryLabel::FadeOut(id));
                }
            }
            LevelAction::ChangeFadeIn(id, change) => {
                if let Some(event) = self.level.events.get_mut(id.event)
                    && let Event::Light(light) = &mut event.event
                {
                    let movement = &mut light.movement;
                    let from = movement.get_fade_in();
                    let change = change.into_delta(from);
                    movement.change_fade_in(from + change);
                    event.time -= movement.get_fade_in() - from;
                    self.save_state(HistoryLabel::FadeIn(id));
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
                if let Some(event) = self.level.events.get_mut(light_id.event)
                    && let Event::Light(light) = &mut event.event
                    && let Some(frame) = light.movement.get_frame_mut(waypoint_id)
                {
                    change.apply(&mut frame.scale);
                    frame.scale = frame.scale.clamp(r32(0.0), r32(10.0));
                    self.save_state(HistoryLabel::Scale(light_id, waypoint_id));
                }
            }
            LevelAction::SetWaypointInterpolation(light, waypoint, interpolation) => {
                self.set_waypoint_interpolation(light, waypoint, interpolation)
            }
            LevelAction::SetWaypointCurve(light, waypoint, curve) => {
                self.set_waypoint_curve(light, waypoint, curve)
            }
            LevelAction::MoveWaypoint(light, waypoint, time, pos) => {
                self.move_waypoint(light, waypoint, pos);
                self.move_waypoint_time(light, waypoint, time, drag);
            }
            LevelAction::ChangeHollow(light, waypoint, change) => {
                self.change_hollow(light, waypoint, change)
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

        let change_pos =
            Change::Add(change_pos.into_delta(event.movement.initial.transform.translation));
        event
            .movement
            .modify_transforms(|transform| change_pos.apply(&mut transform.translation));

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
        drag: Option<&mut Drag>,
    ) {
        let Some(timed_event) = self.level.events.get_mut(light_id.event) else {
            return;
        };

        let Event::Light(event) = &mut timed_event.event else {
            return;
        };

        let mut fix_drag_waypoint_id = self
            .level_state
            .waypoints
            .as_ref()
            .and_then(|waypoints| waypoints.selected)
            .and_then(|selected_waypoint| {
                drag.and_then(|drag| match &mut drag.target {
                    DragTarget::WaypointMove { waypoint, .. } if *waypoint == selected_waypoint => {
                        Some(waypoint)
                    }
                    _ => None,
                })
            });

        // Update time
        let fade_in = event.movement.get_fade_in();
        let fade_out = event.movement.get_fade_out();
        let mut frames: Vec<_> = event
            .movement
            .timed_transforms()
            .map(|(id, transform, mut time)| {
                time += timed_event.time;
                if id == waypoint_id {
                    change_time.apply(&mut time);
                }
                (id, transform, time)
            })
            .collect();

        // Edge (fade in/out) waypoints keep their relative timings unless moved directly
        if matches!(waypoint_id, WaypointId::Frame(_)) {
            let len = frames.len();
            assert!(len >= 2);
            frames[0].2 = frames[1].2 - fade_in;
            frames[len - 1].2 = frames[len - 2].2 + fade_out;
        }

        // Sort frames by absolute time
        frames.sort_by_key(|(_, _, time)| *time);

        // Check if some frames have same time - invalid reorder
        if frames.iter().tuple_windows().any(|(a, b)| a.2 == b.2) {
            // Invalid reorder
            return;
        }

        // Reorder frames to fit the new time
        let mut frames = frames.into_iter().rev(); // NOTE: iterate in reverse because lerp_time has to be calculated based on future frame times
        let mut fixed_movement = event.movement.clone();
        let mut future_time = Time::ZERO; // Time of the next (time-wise) frame
        let mut fix_selection = true;

        // Last frame is treated specially
        if let Some((original_id, transform, time)) = frames.next() {
            let fixed_id = WaypointId::Last;
            if let Some(waypoints) = &mut self.level_state.waypoints
                && fix_selection
                && self.selection.is_light_single(light_id)
                && waypoints.selected == Some(original_id)
            {
                waypoints.selected = Some(fixed_id);
                fix_selection = false;
            }
            if let Some(waypoint) = &mut fix_drag_waypoint_id
                && original_id == **waypoint
            {
                **waypoint = fixed_id;
                fix_drag_waypoint_id = None;
            }

            fixed_movement.last = transform;
            future_time = time;
        }

        // Update all other frames
        for (id, (original_id, transform, time)) in frames.enumerate() {
            // Restore proper index
            let id = match fixed_movement.waypoints.len().checked_sub(id + 1) {
                Some(i) => WaypointId::Frame(i),
                None => WaypointId::Initial,
            };

            if let Some(waypoints) = &mut self.level_state.waypoints
                && fix_selection
                && self.selection.is_light_single(light_id)
                && waypoints.selected == Some(original_id)
            {
                waypoints.selected = Some(id);
                fix_selection = false;
            }
            if let Some(waypoint) = &mut fix_drag_waypoint_id
                && original_id == **waypoint
            {
                **waypoint = id;
                fix_drag_waypoint_id = None;
            }

            // TODO: config option to keep curves at waypoints
            // let (interpolation, curve) = event
            //     .movement
            //     .get_interpolation(fixed_id)
            //     .expect("invalid waypoint id when fixing");
            match id {
                WaypointId::Initial => {
                    let fixed_frame = &mut fixed_movement.initial;
                    fixed_frame.transform = transform;
                    // fixed_frame.interpolation = interpolation;
                    // fixed_frame.curve = curve;
                    fixed_frame.lerp_time = future_time - time;
                    timed_event.time = time;
                }
                WaypointId::Frame(i) => {
                    let fixed_frame = fixed_movement
                        .waypoints
                        .get_mut(i)
                        .expect("invalid waypoint index when fixing");
                    fixed_frame.transform = transform;
                    // fixed_frame.interpolation = interpolation;
                    // fixed_frame.curve = curve;
                    fixed_frame.lerp_time = future_time - time;
                }
                WaypointId::Last => unreachable!(),
            }
            future_time = time;
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
                event.movement.initial.curve = curve.unwrap_or_default();
            }
            WaypointId::Frame(frame) => {
                let Some(frame) = event.movement.waypoints.get_mut(frame) else {
                    return;
                };
                frame.change_curve = curve;
                self.save_state(default());
            }
            WaypointId::Last => {} // Noop
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
                event.movement.initial.interpolation = interpolation;
            }
            WaypointId::Frame(frame) => {
                let Some(frame) = event.movement.waypoints.get_mut(frame) else {
                    return;
                };
                frame.interpolation = interpolation;
                self.save_state(default());
            }
            WaypointId::Last => {} // Noop
        }
    }

    fn select_light(&mut self, mode: SelectMode, ids: Vec<LightId>) {
        self.level_state.waypoints = None;
        self.state = EditingState::Idle;
        match mode {
            SelectMode::Add => {
                for id in ids {
                    self.selection.add_light(id);
                }
            }
            SelectMode::Remove => {
                for id in ids {
                    self.selection.remove_light(id);
                }
            }
            SelectMode::Set => {
                self.selection.clear();
                for id in ids {
                    self.selection.add_light(id);
                }
            }
        }
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
        let Some(light_id) = self.selection.light_single() else {
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
            self.state = EditingState::Waypoints {
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
            self.execute(
                LevelAction::ScrollTime(waypoint_time - self.current_time.target),
                None,
            );
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
        if let Some(event) = self.level.events.get_mut(light_id.event)
            && let Event::Light(event) = &mut event.event
        {
            event.danger = !event.danger;
        }
    }

    fn change_hollow(&mut self, light_id: LightId, waypoint_id: WaypointId, change: Change<R32>) {
        if let Some(event) = self.level.events.get_mut(light_id.event)
            && let Event::Light(event) = &mut event.event
            && let Some(waypoint) = event.movement.get_frame_mut(waypoint_id)
        {
            let mut hollow = waypoint.hollow;
            change.apply(&mut hollow);
            waypoint.hollow = hollow.clamp(-R32::ONE, R32::ONE);
            self.save_state(HistoryLabel::Hollow(light_id, waypoint_id));
        }
    }

    fn cancel(&mut self) {
        match &mut self.state {
            EditingState::Idle => {
                // Cancel selection
                self.execute(LevelAction::Deselect, None);
            }
            EditingState::Place { .. } => {
                // Cancel creation
                self.state = EditingState::Idle;
            }
            EditingState::Waypoints { state, .. } => {
                // Cancel selection
                match state {
                    WaypointsState::Idle => {
                        if let Some(waypoints) = &mut self.level_state.waypoints
                            && waypoints.selected.take().is_some()
                        {
                            return;
                        }
                        self.state = EditingState::Idle
                    }
                    WaypointsState::New => *state = WaypointsState::Idle,
                }
            }
            _ => (),
        }
    }

    fn place_light(&mut self, position: vec2<Coord>) {
        let EditingState::Place { shape, danger } = self.state else {
            return;
        };

        let start_beat = self.current_time.target;
        let timing = self.level.timing.get_timing(start_beat);
        let rotation = self.place_rotation.normalized_2pi();
        self.place_rotation = rotation;
        let movement = Movement::new(
            seconds_to_time(timing.beat_time),
            TransformLight {
                translation: position,
                rotation,
                scale: self.place_scale,
                hollow: r32(-1.0),
            },
        );

        let light = LightEvent {
            shape,
            movement,
            danger,
        };

        let beat = start_beat - light.movement.get_fade_in(); // extra time for the fade in and telegraph
        let event = commit_light(light.clone());
        let event = TimedEvent {
            time: beat,
            event: Event::Light(event),
        };

        let event_i = self.level.events.len();
        self.level.events.push(event);

        self.selection = Selection::Lights(vec![LightId { event: event_i }]);
        self.state = EditingState::Waypoints {
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

        // NOTE: target to make sure it is snapped to the beat
        // assume time interpolation doesn't take long, so not visually weird
        let target_time = self.current_time.target;

        let mut transform = TransformLight {
            translation: position,
            rotation: self.place_rotation,
            scale: self.place_scale,
            hollow: r32(-1.0),
        };
        let mut interpolation = MoveInterpolation::default(); // TODO: use the same as other waypoints
        let mut change_curve = None;
        match i.checked_sub(1) {
            None => {
                // Replace initial
                std::mem::swap(&mut light.movement.initial.transform, &mut transform);
                std::mem::swap(
                    &mut light.movement.initial.interpolation,
                    &mut interpolation,
                );
                change_curve = Some(light.movement.initial.curve);
                light.movement.initial.curve = TrajectoryInterpolation::default();

                let time = target_time - light.movement.get_fade_in(); // Extra time for fade in
                let mut lerp_time = event.time - time;
                std::mem::swap(&mut light.movement.initial.lerp_time, &mut lerp_time);

                light.movement.waypoints.push_front(Waypoint {
                    lerp_time,
                    interpolation,
                    change_curve,
                    transform,
                });
                event.time = time;
            }
            Some(i) if i <= light.movement.waypoints.len() => {
                // Insert keyframe
                let next = light.movement.timed_transforms().nth(i + 1);
                if let Some((_, _, next_time)) = next {
                    let next_time = event.time + next_time;
                    let lerp_time = next_time - target_time;

                    light.movement.waypoints.insert(
                        i,
                        Waypoint {
                            lerp_time,
                            interpolation,
                            change_curve,
                            transform,
                        },
                    );

                    if let Some(prev) = light.movement.waypoints.get_mut(i) {
                        prev.lerp_time -= lerp_time;
                    }
                }
            }
            Some(i) => {
                // Replace last
                std::mem::swap(&mut light.movement.last, &mut transform);

                let lerp_time = target_time - light.movement.timed_transforms().nth(i).unwrap().2;
                light.movement.waypoints.push_back(Waypoint {
                    lerp_time,
                    interpolation,
                    change_curve,
                    transform,
                });
            }
        }
    }
}
