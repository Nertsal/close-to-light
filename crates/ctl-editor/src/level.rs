use super::*;

pub struct LevelEditor {
    pub context: Context,
    /// Static (initial) version of the level.
    pub static_level: PlayLevel,
    /// Current state of the level.
    pub level: Level,
    pub name: String,

    /// Simulation model.
    pub model: Model,
    pub level_state: EditorLevelState,
    pub current_time: TimeInterpolation,
    pub timeline_zoom: SecondOrderState<R32>,
    pub real_time: FloatTime,
    pub timeline_light_hover: Option<LightId>,

    pub history: History,
    pub clipboard: Clipboard,
    pub selection: Selection,

    /// At what rotation the objects should be placed.
    pub place_rotation: Angle<Coord>,
    /// The scale at which the objects should be placed.
    pub place_scale: Coord,

    pub state: EditingState,
    /// Whether the last frame was scrolled through time.
    pub was_scrolling_time: bool,
    /// Whether currently scrolling through time.
    /// Used as a hack to not replay the music every frame.
    pub scrolling_time: bool,
}

#[derive(Debug, Clone, Default)]
pub enum Selection {
    #[default]
    Empty,
    Lights(Vec<LightId>),
    Event(usize),
}

impl Selection {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_empty(&self) -> bool {
        match self {
            Selection::Empty => true,
            Selection::Lights(light_ids) => light_ids.is_empty(),
            Selection::Event(_) => false,
        }
    }

    pub fn clear(&mut self) {
        *self = Self::Empty;
    }

    pub fn event_single(&self) -> Option<usize> {
        match self {
            Selection::Empty => None,
            Selection::Lights(_) => None,
            Selection::Event(idx) => Some(*idx),
        }
    }

    pub fn is_event_single(&self, id: usize) -> bool {
        match self {
            Selection::Empty => false,
            Selection::Lights(lights) => lights.len() == 1 && lights.first().unwrap().event == id,
            Selection::Event(idx) => id == *idx,
        }
    }

    pub fn light_single(&self) -> Option<LightId> {
        match self {
            Selection::Empty => None,
            Selection::Lights(lights) => (lights.len() == 1).then(|| *lights.first().unwrap()),
            Selection::Event(_) => None,
        }
    }

    pub fn is_light_single(&self, id: LightId) -> bool {
        self.light_single() == Some(id)
    }

    pub fn is_light_selected(&self, id: LightId) -> bool {
        match self {
            Selection::Empty => false,
            Selection::Lights(lights) => lights.contains(&id),
            Selection::Event(_) => false,
        }
    }

    pub fn add_light(&mut self, id: LightId) {
        match self {
            Selection::Empty => *self = Self::Lights(vec![id]),
            Selection::Lights(lights) => {
                if !lights.contains(&id) {
                    lights.push(id)
                }
            }
            Selection::Event(_) => *self = Self::Lights(vec![id]),
        }
    }

    pub fn remove_light(&mut self, id: LightId) {
        match self {
            Selection::Empty => {}
            Selection::Lights(lights) => {
                if let Some(i) = lights.iter().position(|l| *l == id) {
                    lights.swap_remove(i);
                }
            }
            Selection::Event(_) => {}
        }
    }

    pub fn merge(&mut self, other: Self) {
        match other {
            Selection::Empty => {}
            Selection::Lights(light_ids) => {
                for id in light_ids {
                    self.add_light(id);
                }
            }
            Selection::Event(_) => *self = other,
        }
    }
}

impl LevelEditor {
    pub fn new(
        context: Context,
        model: Model,
        level: PlayLevel,
        visualize_beat: bool,
        show_only_selected: bool,
    ) -> Self {
        let mut editor = Self {
            context,
            level: (*level.level.data).clone(),
            name: level.level.meta.name.to_string(),

            level_state: EditorLevelState::default(),
            current_time: TimeInterpolation::new(),
            timeline_zoom: SecondOrderState::new(SecondOrderDynamics::new(3.0, 1.0, 0.0, r32(0.5))),
            real_time: FloatTime::ZERO,
            timeline_light_hover: None,

            history: History::new(&level.level.data),
            clipboard: Clipboard::new(),
            selection: Selection::new(),

            place_rotation: Angle::ZERO,
            place_scale: Coord::ONE,

            state: EditingState::Idle,
            was_scrolling_time: false,
            scrolling_time: false,

            static_level: level,
            model,
        };
        editor.render_lights(None, None, visualize_beat, show_only_selected);
        editor
    }

    pub fn delete_light(&mut self, id: LightId) {
        if id.event >= self.level.events.len() {
            return;
        }
        self.level.events.swap_remove(id.event);
        self.selection.clear();
        self.save_state(default());
    }

    pub fn delete_waypoint(&mut self, light: LightId, waypoint: WaypointId) {
        let Some(timed_event) = self.level.events.get_mut(light.event) else {
            return;
        };
        let Event::Light(event) = &mut timed_event.event else {
            return;
        };
        match waypoint {
            WaypointId::Initial => {
                match event.movement.waypoints.pop_front() {
                    None => {
                        // No waypoints -> delete the whole event
                        if light.event < self.level.events.len() {
                            self.level.events.swap_remove(light.event);
                            self.level_state.waypoints = None;
                            self.state = EditingState::Idle;
                        }
                    }
                    Some(frame) => {
                        // Make the first frame the initial position
                        timed_event.time += event.movement.initial.lerp_time;
                        event.movement.initial = WaypointInitial {
                            lerp_time: frame.lerp_time,
                            interpolation: frame.interpolation,
                            curve: frame.change_curve.unwrap_or_default(),
                            transform: frame.transform,
                        };
                    }
                }
            }
            WaypointId::Frame(i) => {
                if let Some(frame) = event.movement.waypoints.remove(i) {
                    // Offset the previous one
                    if i == 0 {
                        event.movement.initial.lerp_time += frame.lerp_time;
                    } else if let Some(prev) = event.movement.waypoints.get_mut(i - 1) {
                        prev.lerp_time += frame.lerp_time;
                    }
                }
            }
            WaypointId::Last => {
                match event.movement.waypoints.pop_back() {
                    None => {
                        // No waypoints -> delete the whole event
                        if light.event < self.level.events.len() {
                            self.level.events.swap_remove(light.event);
                            self.level_state.waypoints = None;
                            self.state = EditingState::Idle;
                        }
                    }
                    Some(frame) => {
                        // Make the last frame the last position
                        event.movement.last = frame.transform;
                    }
                }
            }
        }

        if let Some(waypoints) = &mut self.level_state.waypoints {
            waypoints.selected = None;
        }
        self.save_state(default());
    }

    pub fn undo(&mut self) {
        match &mut self.state {
            EditingState::Playing { .. } => {}
            EditingState::Place { .. } => {}
            EditingState::Idle | EditingState::Waypoints { .. } => {
                self.history.undo(&mut self.level);
            }
        }
    }

    pub fn redo(&mut self) {
        match &mut self.state {
            EditingState::Playing { .. } => {}
            EditingState::Place { .. } => {}
            EditingState::Idle | EditingState::Waypoints { .. } => {
                self.history.redo(&mut self.level);
            }
        }
    }

    /// Save level changes to the history.
    #[track_caller]
    pub fn save_state(&mut self, label: HistoryLabel) {
        log::trace!("save_state called by {}", std::panic::Location::caller());
        self.history.save_state(&self.level, label);
    }

    /// Flush all buffered changes to the undo stack, if there are any.
    #[track_caller]
    pub fn flush_changes(&mut self, label: Option<HistoryLabel>) {
        log::trace!("flush_changes called by {}", std::panic::Location::caller());
        self.history.flush(&self.level, label.unwrap_or_default());
    }

    #[track_caller]
    pub fn start_merge_changes(&mut self, label: Option<HistoryLabel>) {
        log::trace!(
            "start_merge_changes called by {}",
            std::panic::Location::caller()
        );
        self.history
            .start_merge(&self.level, label.unwrap_or(HistoryLabel::Merge));
    }

    pub fn copy(&mut self) {
        self.execute(LevelAction::CopySelection(self.selection.clone()), None);
    }

    pub fn paste(&mut self) {
        let Some(item) = self.clipboard.paste() else {
            return;
        };

        match item {
            ClipboardItem::Events(time, events) => {
                let new_ids = (0..events.len())
                    .map(|i| LightId {
                        event: self.level.events.len() + i,
                    })
                    .collect();
                self.level
                    .events
                    .extend(events.into_iter().map(|event| TimedEvent {
                        time: self.current_time.target + event.time - time,
                        event: event.event,
                    }));
                // Change selection to the new lights
                self.selection = Selection::Lights(new_ids);
            }
        }
    }

    // TODO: reimplement with smooth transition or smth
    // /// Swap the palette at current time.
    // fn palette_swap(&mut self) {
    //     // Remove any already existing palette swap event at current time
    //     let mut ids = Vec::new();
    //     for (i, event) in self.level.events.iter().enumerate() {
    //         if event.beat == self.current_beat {
    //             if let Event::PaletteSwap = event.event {
    //                 ids.push(i);
    //             }
    //         }
    //     }

    //     let add = ids.len() % 2 == 0;

    //     // Remove events
    //     for i in ids.into_iter().rev() {
    //         self.level.events.swap_remove(i);
    //     }

    //     if add {
    //         // Add a new palette swap event
    //         self.level.events.push(TimedEvent {
    //             beat: self.current_beat,
    //             event: Event::PaletteSwap,
    //         });
    //     }
    // }

    pub fn new_waypoint(&mut self) {
        self.execute(LevelAction::DeselectWaypoint, None);

        if let EditingState::Waypoints { state, .. } = &mut self.state {
            *state = WaypointsState::New;
        }
    }

    pub fn view_waypoints(&mut self) {
        match self.state {
            EditingState::Idle => {
                if let Some(selected) = self.selection.light_single() {
                    self.state = EditingState::Waypoints {
                        light_id: selected,
                        state: WaypointsState::Idle,
                    };
                }
            }
            EditingState::Waypoints { .. } => {
                self.state = EditingState::Idle;
            }
            _ => (),
        }
    }

    pub fn scroll_time(&mut self, delta: Time) {
        if let EditingState::Playing { .. } = self.state {
            return;
        }

        let margin = 100 * TIME_IN_FLOAT_TIME;
        let min = Time::ZERO;
        let max = margin + self.level.last_time();
        let target = (self.current_time.target + delta).clamp(min, max);

        // TODO: customize snap
        let target_time = self.level.timing.snap_to_beat(target, BeatTime::QUARTER);
        self.current_time.scroll_time(Change::Set(target_time));

        self.scrolling_time = true;

        if let EditingState::Playing { old_state, .. } = &self.state {
            self.state = (**old_state).clone()
        }
    }

    pub fn render_lights(
        &mut self,
        cursor_world_pos: Option<vec2<Coord>>,
        cursor_world_pos_snapped: Option<vec2<Coord>>,
        visualize_beat: bool,
        show_only_selected: bool,
    ) {
        let (static_time, dynamic_time) = if let EditingState::Playing { .. } = self.state {
            // TODO: self.music.play_position()
            (None, Some(self.current_time.value))
        } else {
            let time = self.current_time.value;
            let dynamic = if visualize_beat {
                // TODO: customize dynamic visual
                Some(time + seconds_to_time(self.real_time.fract()))
            } else {
                None
            };
            (Some(time), dynamic)
        };

        let selected_level = show_only_selected
            .then(|| {
                let Selection::Lights(lights) = &self.selection else {
                    return None;
                };
                let events = lights
                    .iter()
                    .flat_map(|id| self.level.events.get(id.event))
                    .cloned()
                    .collect();
                Some(Level {
                    events,
                    timing: self.level.timing.clone(), // TODO: cheaper clone
                })
            })
            .flatten();
        let level = selected_level.as_ref().unwrap_or(&self.level);

        let static_level = static_time.map(|time| {
            LevelState::render(
                level,
                &self.model.level.config,
                time,
                None,
                Some(&mut self.model.vfx),
            )
        });
        let dynamic_level = dynamic_time.map(|time| {
            let vfx = static_time.is_none().then_some(&mut self.model.vfx);
            LevelState::render(level, &self.model.level.config, time, None, vfx)
        });

        let mut hovered_light = self.timeline_light_hover.take();
        if hovered_light.is_none()
            && let EditingState::Idle = self.state
            && let Some(level) = &static_level
        {
            hovered_light = cursor_world_pos.and_then(|cursor| {
                level
                    .lights
                    .iter()
                    .find(|light| light.contains_point(cursor))
                    .and_then(|light| light.event_id)
                    .map(|event| LightId { event })
            });
        }

        let mut waypoints = None;
        if let EditingState::Waypoints { light_id, state } = &self.state {
            let light_id = *light_id;
            if let Some(timed_event) = self.level.events.get(light_id.event)
                && let Event::Light(light_event) = &timed_event.event
            {
                let event_time = timed_event.time;
                // If some waypoints overlap, render the temporaly closest one
                let base_collider = Collider::new(vec2::ZERO, light_event.shape);

                /// Waypoints past this time-distance are not rendered at all
                const MAX_VISIBILITY: Time = 5 * TIME_IN_FLOAT_TIME;
                let visible = |beat: Time| {
                    let d = (event_time + beat - self.current_time.value).abs();
                    d <= MAX_VISIBILITY
                };

                // TODO: use cached
                let curve = light_event.movement.bake();
                let mut points: Vec<_> = light_event
                    .movement
                    .timed_transforms()
                    .map(|(i, trans_control, time)| {
                        let trans_actual = match i {
                            WaypointId::Initial => curve.get(0, FloatTime::ZERO),
                            WaypointId::Frame(i) => curve.get(i, FloatTime::ONE),
                            WaypointId::Last => {
                                curve.get(light_event.movement.waypoints.len(), FloatTime::ONE)
                            }
                        }
                        .unwrap_or(trans_control); // Should be unreachable, but just in case
                        (
                            WaypointEdit {
                                visible: visible(time),
                                original: Some(i),
                                control: base_collider.transformed(trans_control),
                                actual: base_collider.transformed(trans_actual),
                            },
                            time,
                        )
                    })
                    .collect();
                points.sort_by_key(|(point, time)| {
                    (
                        point.control.position.x,
                        point.control.position.y,
                        (event_time + *time - self.current_time.value).abs(),
                    )
                });

                {
                    let mut points = points.iter_mut();
                    if let Some(last) = points.next() {
                        let mut last = last.0.control.position;
                        for (point, _) in points {
                            let pos = point.control.position;
                            if last == pos {
                                point.visible = false;
                            }
                            last = pos;
                        }
                    }
                }
                points.sort_by_key(|(point, _)| point.original); // Restore proper order

                if let WaypointsState::New = state
                    && let Some(cursor_world_pos_snapped) = cursor_world_pos_snapped
                {
                    // NOTE: assuming that positions don't go backwards in time
                    // Insert a new waypoint at current time
                    let new_time = self.current_time.value - event_time;
                    let i = match points.binary_search_by_key(&new_time, |(_, time)| *time) {
                        Ok(i) | Err(i) => i,
                    };
                    let control = base_collider.transformed(TransformLight {
                        translation: cursor_world_pos_snapped,
                        rotation: self.place_rotation,
                        scale: self.place_scale,
                        hollow: r32(-1.0),
                    });
                    points.insert(
                        i,
                        (
                            WaypointEdit {
                                visible: true,
                                original: None,
                                actual: control.clone(),
                                control,
                            },
                            new_time,
                        ),
                    );
                }

                let hovered = cursor_world_pos.and_then(|cursor_world_pos| {
                    points
                        .iter()
                        .enumerate()
                        .filter(|(_, (point, _))| {
                            point.visible
                                && (point.control.contains(cursor_world_pos)
                                    || point.actual.contains(cursor_world_pos))
                        })
                        .min_by_key(|(_, (_, time))| {
                            (self.current_time.value - event_time - *time).abs()
                        })
                        .map(|(i, _)| i)
                });
                let points: Vec<_> = points.into_iter().map(|(point, _)| point).collect();

                waypoints = Some(Waypoints {
                    light: light_id,
                    points,
                    hovered,
                    selected: self
                        .level_state
                        .waypoints
                        .as_ref()
                        .and_then(|waypoints| waypoints.selected),
                });
            }
        }

        self.level_state = EditorLevelState {
            static_level,
            dynamic_level,
            hovered_light,
            waypoints,
        };
    }
}

pub fn commit_light(light: LightEvent) -> LightEvent {
    light
}
