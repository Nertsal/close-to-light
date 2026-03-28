use super::*;

const MAX_SCALE: f32 = 20.0;

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
    SetBeatSnap(BeatTime),
    TimelineZoom(Change<f32>),
    CameraPan(Change<vec2<f32>>),
    /// Selected a shape, but the specific action is up to interpretation.
    /// If there is a light selected, changes its shape; otherwise creates a new light.
    Shape(Shape),
    Deselect,
    DeselectWaypoint,

    // General event
    SelectEvent(EditorEventIdx),
    DeleteEvent(EditorEventIdx),
    MoveEvent(EditorEventIdx, Change<Time>),

    // Timing
    TimingNew(Time, FloatTime),
    TimingUpdate(usize, FloatTime),

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
    SelectWaypoint(SelectMode, LightId, Vec<WaypointId>, bool),
    RotateWaypointAround(LightId, WaypointId, vec2<Coord>, Change<Angle<Coord>>),
    ScaleWaypoint(LightId, WaypointId, Change<Coord>),
    SetWaypointInterpolation(LightId, WaypointId, MoveInterpolation),
    SetWaypointCurve(LightId, WaypointId, Option<TrajectoryInterpolation>),
    MoveWaypoint(LightId, Vec<WaypointId>, Change<Time>, Change<vec2<Coord>>),
    ChangeHollow(LightId, WaypointId, Change<R32>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectMode {
    Add,
    Remove,
    Toggle,
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
            LevelAction::SetBeatSnap(_) => false,
            LevelAction::TimelineZoom(zoom) => zoom.is_noop(&0.0),
            LevelAction::CameraPan(delta) => delta.is_noop(&vec2::ZERO),
            LevelAction::Shape(..) => false,
            LevelAction::Deselect => false,
            LevelAction::DeselectWaypoint => false,

            LevelAction::SelectEvent(_) => false,
            LevelAction::DeleteEvent(_) => false,
            LevelAction::MoveEvent(_, delta) => delta.is_noop(&0),

            LevelAction::TimingNew(..) => false,
            LevelAction::TimingUpdate(..) => false,

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
                matches!(
                    mode,
                    SelectMode::Add | SelectMode::Remove | SelectMode::Toggle
                ) && lights.is_empty()
            }
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
            LevelAction::SelectWaypoint(mode, _, ids, _) => {
                matches!(
                    mode,
                    SelectMode::Add | SelectMode::Remove | SelectMode::Toggle
                ) && ids.is_empty()
            }
            LevelAction::RotateWaypointAround(_, _, _, delta) => delta.is_noop(&Angle::ZERO),
            LevelAction::ScaleWaypoint(_, _, delta) => delta.is_noop(&Coord::ZERO),
            LevelAction::SetWaypointInterpolation(..) => false,
            LevelAction::SetWaypointCurve(..) => false,
            LevelAction::MoveWaypoint(_, ids, time, position) => {
                ids.is_empty() || time.is_noop(&Time::ZERO) && position.is_noop(&vec2::ZERO)
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
                Selection::Waypoints(..) => {
                    // TODO: copy waypoints maybe?
                }
                Selection::Event(index) => {
                    let events = self.level.events.get(index).cloned().into_iter().collect();
                    self.clipboard
                        .copy(ClipboardItem::Events(self.current_time.target, events));
                }
                Selection::Timing(_) => {
                    // TODO: copy timing maybe?
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
                self.place_scale = self.place_scale.clamp(r32(0.25), r32(MAX_SCALE));
            }
            LevelAction::RotatePlacement(delta) => {
                self.place_rotation += delta;
            }
            LevelAction::ScrollTime(delta) => {
                self.scroll_time(delta);
            }
            LevelAction::SetBeatSnap(beat_snap) => self.beat_snap = beat_snap,
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
            LevelAction::Deselect => self.deselect(),
            LevelAction::DeselectWaypoint => {
                if let Selection::Waypoints(light_id, _) = self.selection {
                    self.selection = Selection::Lights(vec![light_id]);
                }
            }

            LevelAction::SelectEvent(index) => match index {
                EditorEventIdx::Event(index) => {
                    if self.level.events.get(index).is_some() {
                        self.selection = Selection::Event(index);
                    }
                }
                EditorEventIdx::Waypoint(light_id, waypoint_id) => {
                    if let Some(event) = self.level.events.get(light_id.event)
                        && let Event::Light(light) = &event.event
                        && light.movement.get_frame(waypoint_id).is_some()
                    {
                        self.selection = Selection::Waypoints(light_id, vec![waypoint_id]);
                    }
                }
                EditorEventIdx::Timing(index) => {
                    if self.level.timing.points.get(index).is_some() {
                        self.selection = Selection::Timing(index);
                    }
                }
            },
            LevelAction::DeleteEvent(index) => match index {
                EditorEventIdx::Event(index) => {
                    if self.level.events.get(index).is_some() {
                        self.execute(LevelAction::Deselect, drag);
                        self.level.events.swap_remove(index);
                    }
                }
                EditorEventIdx::Waypoint(light_id, waypoint_id) => {
                    self.execute(LevelAction::DeleteWaypoint(light_id, waypoint_id), drag);
                }
                EditorEventIdx::Timing(index) => {
                    if self.level.timing.points.get(index).is_some() {
                        self.execute(LevelAction::Deselect, drag);
                        self.level.timing.points.remove(index);
                    }
                }
            },
            LevelAction::MoveEvent(idx, change) => match idx {
                EditorEventIdx::Event(index) => {
                    if let Some(event) = self.level.events.get_mut(index) {
                        change.apply(&mut event.time);
                        self.save_state(HistoryLabel::MoveEvent(idx));
                    }
                }
                EditorEventIdx::Waypoint(light_id, waypoint_id) => {
                    self.execute(
                        LevelAction::MoveWaypoint(
                            light_id,
                            vec![waypoint_id],
                            change,
                            Change::Add(vec2::ZERO),
                        ),
                        drag,
                    );
                }
                EditorEventIdx::Timing(index) => {
                    if let Some(event) = self.level.timing.points.get_mut(index) {
                        change.apply(&mut event.time);
                        self.level.timing.points.sort_by_key(|point| point.time);
                        self.save_state(HistoryLabel::MoveEvent(idx));
                    }
                }
            },

            LevelAction::TimingNew(time, beat_time) => {
                match self
                    .level
                    .timing
                    .points
                    .binary_search_by_key(&time, |point| point.time)
                {
                    Ok(_) => {
                        // Point already exists at this time
                    }
                    Err(i) => self
                        .level
                        .timing
                        .points
                        .insert(i, TimingPoint { time, beat_time }),
                }
            }
            LevelAction::TimingUpdate(point, beat_time) => {
                if let Some(point) = self.level.timing.points.get_mut(point) {
                    point.beat_time = beat_time;
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
            LevelAction::SelectWaypoint(mode, light_id, ids, move_time) => {
                self.select_waypoint(mode, light_id, ids, move_time)
            }
            LevelAction::RotateWaypointAround(light, waypoint, anchor, change) => {
                self.rotate_around(light, waypoint, anchor, change)
            }
            LevelAction::ScaleWaypoint(light_id, waypoint_id, change) => {
                if let Some(event) = self.level.events.get_mut(light_id.event)
                    && let Event::Light(light) = &mut event.event
                    && let Some(frame) = light.movement.get_frame_mut(waypoint_id)
                {
                    change.apply(&mut frame.scale);
                    frame.scale = frame.scale.clamp(r32(0.0), r32(MAX_SCALE));
                    self.save_state(HistoryLabel::Scale(light_id, waypoint_id));
                }
            }
            LevelAction::SetWaypointInterpolation(light, waypoint, interpolation) => {
                self.set_waypoint_interpolation(light, waypoint, interpolation)
            }
            LevelAction::SetWaypointCurve(light, waypoint, curve) => {
                self.set_waypoint_curve(light, waypoint, curve)
            }
            LevelAction::MoveWaypoint(light, waypoints, time, pos) => {
                self.move_waypoint(light, &waypoints, pos);
                self.move_waypoint_time(light, &waypoints, time, drag);
            }
            LevelAction::ChangeHollow(light, waypoint, change) => {
                self.change_hollow(light, waypoint, change)
            }
        }

        // In case some action forgot to save the state,
        // we save it with the default label
        self.save_state(default());
    }

    fn deselect(&mut self) {
        match self.selection {
            Selection::Waypoints(light_id, _) => self.selection = Selection::Lights(vec![light_id]),
            _ => self.selection.clear(),
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
        waypoint_ids: &[WaypointId],
        change_pos: Change<vec2<Coord>>,
    ) {
        if waypoint_ids.is_empty() {
            return;
        };
        let Some(timed_event) = self.level.events.get_mut(light_id.event) else {
            return;
        };

        let Event::Light(event) = &mut timed_event.event else {
            return;
        };

        for &waypoint_id in waypoint_ids {
            let Some(frame) = event.movement.get_frame_mut(waypoint_id) else {
                continue;
            };

            change_pos.apply(&mut frame.translation);
            if let WaypointId::Frame(i) = waypoint_id {
                // Move fade as well
                if i == 0 {
                    change_pos.apply(&mut event.movement.initial.transform.translation);
                }
                if i + 1 == event.movement.waypoints.len() {
                    change_pos.apply(&mut event.movement.last.translation);
                }
            }
        }

        self.save_state(HistoryLabel::MoveWaypoint(
            light_id,
            *waypoint_ids.first().unwrap(),
        ));
    }

    fn move_waypoint_time(
        &mut self,
        light_id: LightId,
        waypoint_ids: &[WaypointId],
        change_time: Change<Time>,
        drag: Option<&mut Drag>,
    ) {
        if waypoint_ids.is_empty() {
            return;
        }
        let Some(timed_event) = self.level.events.get_mut(light_id.event) else {
            return;
        };

        let Event::Light(event) = &mut timed_event.event else {
            return;
        };

        // let mut fix_drag_waypoint_id = self
        //     .level_state
        //     .waypoints
        //     .as_ref()
        //     .and_then(|waypoints| waypoints.selected)
        //     .and_then(|selected_waypoint| {
        //         drag.and_then(|drag| match &mut drag.target {
        //             DragTarget::WaypointMove { waypoint, .. } if *waypoint == selected_waypoint => {
        //                 Some(waypoint)
        //             }
        //             _ => None,
        //         })
        //     });

        // Update time
        let fade_in = event.movement.get_fade_in();
        let fade_out = event.movement.get_fade_out();
        let mut frames: Vec<_> = event
            .movement
            .timed_transforms()
            .map(|(id, transform, mut time)| {
                time += timed_event.time;
                if waypoint_ids.contains(&id) {
                    change_time.apply(&mut time);
                }
                (id, transform, time)
            })
            .collect();

        // Edge (fade in/out) waypoints keep their relative timings unless moved directly
        if !waypoint_ids
            .iter()
            .any(|id| matches!(id, WaypointId::Initial))
        {
            frames[0].2 = frames[1].2 - fade_in;
        }
        if !waypoint_ids.iter().any(|id| matches!(id, WaypointId::Last)) {
            let len = frames.len();
            assert!(len >= 2);
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

        let mut fixed_ids: HashMap<WaypointId, WaypointId> = HashMap::new();

        // Last frame is treated specially
        if let Some((original_id, transform, time)) = frames.next() {
            let fixed_id = WaypointId::Last;
            fixed_ids.insert(original_id, fixed_id);
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
            fixed_ids.insert(original_id, id);

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

        // Fix ids
        let fix_id = |id: &mut WaypointId| {
            if let Some(fixed_id) = fixed_ids.get(id) {
                *id = *fixed_id;
            }
        };
        if let Selection::Waypoints(light, ids) = &mut self.selection
            && *light == light_id
        {
            ids.iter_mut().for_each(fix_id);
        }
        if let Some(drag) = drag
            && let DragTarget::WaypointMove { light, waypoints } = &mut drag.target
            && *light == light_id
        {
            waypoints.iter_mut().for_each(|drag| fix_id(&mut drag.id));
        }

        self.save_state(HistoryLabel::MoveWaypointTime(
            light_id,
            *waypoint_ids.first().unwrap(),
        ));
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
            SelectMode::Toggle => {
                for id in ids {
                    if self.selection.is_light_selected(id) {
                        self.selection.remove_light(id);
                    } else {
                        self.selection.add_light(id);
                    }
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

    fn select_waypoint(
        &mut self,
        mode: SelectMode,
        light_id: LightId,
        ids: Vec<WaypointId>,
        move_time: bool,
    ) {
        // Update selection
        match mode {
            SelectMode::Add => {
                for &id in &ids {
                    self.selection.add_waypoint(light_id, id);
                }
            }
            SelectMode::Remove => {
                for &id in &ids {
                    self.selection.remove_waypoint(light_id, id);
                }
            }
            SelectMode::Toggle => {
                for &id in &ids {
                    if self.selection.is_waypoint_selected(light_id, id) {
                        self.selection.remove_waypoint(light_id, id);
                    } else {
                        self.selection.add_waypoint(light_id, id);
                    }
                }
            }
            SelectMode::Set => {
                self.selection.clear();
                for &id in &ids {
                    self.selection.add_waypoint(light_id, id);
                }
            }
        }

        // Update state
        self.state = EditingState::Waypoints {
            light_id,
            state: WaypointsState::Idle,
        };
        self.level_state.waypoints = Some(Waypoints {
            light: light_id,
            points: Vec::new(),
            hovered: None,
        });

        // Move to waypoint if it's a single one
        if move_time {
            let waypoint_time = (ids.len() == 1)
                .then(|| {
                    let waypoint_id = *ids.first().unwrap();
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
                })
                .flatten();
            if let Some(waypoint_time) = waypoint_time {
                self.execute(
                    LevelAction::ScrollTime(waypoint_time - self.current_time.target),
                    None,
                );
            }
        }
    }

    fn rotate_around(
        &mut self,
        light_id: LightId,
        waypoint_id: WaypointId,
        anchor: vec2<Coord>,
        change: Change<Angle<Coord>>,
    ) {
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

        let rotate = |rotation: &mut Angle<Coord>, position: &mut vec2<Coord>| {
            let delta = *rotation;
            change.apply(rotation);
            let delta = *rotation - delta;
            *position = anchor + (*position - anchor).rotate(delta);
        };

        rotate(&mut frame.rotation, &mut frame.translation);
        if let WaypointId::Frame(i) = waypoint_id {
            // Move fade as well
            if i == 0 {
                rotate(
                    &mut event.movement.initial.transform.rotation,
                    &mut event.movement.initial.transform.translation,
                );
            }
            if i + 1 == event.movement.waypoints.len() {
                rotate(
                    &mut event.movement.last.rotation,
                    &mut event.movement.last.translation,
                );
            }
        }
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
                        if let Selection::Waypoints(..) = self.selection {
                            self.execute(LevelAction::DeselectWaypoint, None);
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

        let prev_frame = i.checked_sub(2).and_then(|i| {
            light
                .movement
                .waypoints
                .get(i)
                .or(light.movement.waypoints.back())
        });
        let prev_transform =
            prev_frame.map_or(light.movement.initial.transform, |frame| frame.transform);
        let transform = TransformLight {
            translation: position,
            rotation: self.place_rotation,
            scale: self.place_scale,
            hollow: prev_transform.hollow,
        };
        let mut interpolation = prev_frame.map_or(light.movement.initial.interpolation, |frame| {
            frame.interpolation
        });
        let mut change_curve = None;
        match i.checked_sub(1) {
            None | Some(0) => {
                // Extend fade in
                let lerp_time = event.time + light.movement.get_fade_in() - target_time;
                let time = event.time - lerp_time;

                light.movement.waypoints.push_front(Waypoint {
                    lerp_time,
                    interpolation,
                    change_curve,
                    transform,
                });
                light.movement.initial.transform = TransformLight {
                    translation: transform.translation,
                    rotation: transform.rotation,
                    ..light.movement.last
                };
                event.time = time;
            }
            Some(i) if i < light.movement.waypoints.len() => {
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

                    if let Some(prev) = i
                        .checked_sub(1)
                        .and_then(|i| light.movement.waypoints.get_mut(i))
                    {
                        prev.lerp_time -= lerp_time;
                    } else {
                        unreachable!()
                        // light.movement.initial.lerp_time -= lerp_time;
                    }
                }
            }
            Some(i) if i >= light.movement.waypoints.len() => {
                // Extend fade out
                let lerp_time = light.movement.get_fade_out();
                let plus_time =
                    target_time - event.time - light.movement.timed_transforms().last().unwrap().2;
                if let Some(prev) = light.movement.waypoints.back_mut() {
                    std::mem::swap(&mut change_curve, &mut prev.change_curve);
                    std::mem::swap(&mut interpolation, &mut prev.interpolation);
                    prev.lerp_time += plus_time;
                } else {
                    let prev = &mut light.movement.initial;
                    prev.lerp_time += plus_time;
                }

                light.movement.waypoints.push_back(Waypoint {
                    lerp_time,
                    interpolation,
                    change_curve,
                    transform,
                });
                light.movement.last = TransformLight {
                    translation: transform.translation,
                    rotation: transform.rotation,
                    ..light.movement.last
                };
            }
            Some(1..) => unreachable!(), // For some reason Rust does not properly check exhaustivenes here
        }
    }
}
