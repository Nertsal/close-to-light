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
    pub selected_light: Option<LightId>,
    pub timeline_light_hover: Option<LightId>,

    pub history: History,

    /// At what rotation the objects should be placed.
    pub place_rotation: Angle<Coord>,
    /// The scale at which the objects should be placed.
    pub place_scale: Coord,

    pub state: State,
    /// Whether the last frame was scrolled through time.
    pub was_scrolling_time: bool,
    /// Whether currently scrolling through time.
    /// Used as a hack to not replay the music every frame.
    pub scrolling_time: bool,
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
            level_state: EditorLevelState::default(),
            current_time: TimeInterpolation::new(),
            timeline_zoom: SecondOrderState::new(SecondOrderDynamics::new(3.0, 1.0, 0.0, r32(0.5))),
            real_time: FloatTime::ZERO,
            selected_light: None,
            timeline_light_hover: None,
            place_rotation: Angle::ZERO,
            place_scale: Coord::ONE,
            state: State::Idle,
            was_scrolling_time: false,
            scrolling_time: false,
            history: History::new(&level.level.data),
            level: level.level.data.clone(),
            name: level.level.meta.name.to_string(),
            static_level: level,
            model,
        };
        editor.render_lights(vec2::ZERO, vec2::ZERO, visualize_beat, show_only_selected);
        editor
    }

    pub fn delete_light(&mut self, id: LightId) {
        if id.event >= self.level.events.len() {
            return;
        }
        self.level.events.swap_remove(id.event);
        self.selected_light = None;
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
                match event.movement.key_frames.pop_front() {
                    None => {
                        // No waypoints -> delete the whole event
                        if light.event < self.level.events.len() {
                            self.level.events.swap_remove(light.event);
                            self.level_state.waypoints = None;
                            self.state = State::Idle;
                        }
                    }
                    Some(frame) => {
                        // Make the first frame the initial position
                        event.movement.initial = frame.transform;
                        timed_event.time += frame.lerp_time;
                    }
                }
            }
            WaypointId::Frame(i) => {
                if let Some(frame) = event.movement.key_frames.remove(i) {
                    // Offset the next one
                    if let Some(next) = event.movement.key_frames.get_mut(i) {
                        next.lerp_time += frame.lerp_time;
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
            State::Playing { .. } => {}
            State::Place { .. } => {}
            State::Idle | State::Waypoints { .. } => {
                if let Some(mut level) = self.history.undo_stack.pop() {
                    std::mem::swap(&mut level, &mut self.level);
                    self.history.redo_stack.push(level);
                    self.history.buffer_state = self.level.clone();
                    self.history.buffer_label = HistoryLabel::default();
                }
            }
        }
    }

    pub fn redo(&mut self) {
        match &mut self.state {
            State::Playing { .. } => {}
            State::Place { .. } => {}
            State::Idle | State::Waypoints { .. } => {
                if let Some(mut level) = self.history.redo_stack.pop() {
                    std::mem::swap(&mut level, &mut self.level);
                    self.history.undo_stack.push(level);
                    self.history.buffer_state = self.level.clone();
                    self.history.buffer_label = HistoryLabel::default();
                }
            }
        }
    }

    /// Save level changes to the history.
    #[track_caller]
    pub fn save_state(&mut self, label: HistoryLabel) {
        self.history.save_state(&self.level, label);
        log::trace!("save_state called by {}", std::panic::Location::caller());
    }

    /// Flush all buffered changes to the undo stack, if there are any.
    #[track_caller]
    pub fn flush_changes(&mut self) {
        self.history.flush(&self.level);
        log::trace!("flush_changes called by {}", std::panic::Location::caller());
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
        self.execute(LevelAction::DeselectWaypoint);

        if let State::Waypoints { state, .. } = &mut self.state {
            *state = WaypointsState::New;
        }
    }

    pub fn view_waypoints(&mut self) {
        match self.state {
            State::Idle => {
                if let Some(selected) = self.selected_light {
                    self.state = State::Waypoints {
                        light_id: selected,
                        state: WaypointsState::Idle,
                    };
                }
            }
            State::Waypoints { .. } => {
                self.state = State::Idle;
            }
            _ => (),
        }
    }

    pub fn scroll_time(&mut self, delta: Time) {
        let margin = 100 * TIME_IN_FLOAT_TIME;
        let min = Time::ZERO;
        let max = margin + self.level.last_time();
        let target = (self.current_time.target + delta).clamp(min, max);

        // TODO: customize snap
        self.current_time.scroll_time(Change::Set(
            self.level.timing.snap_to_beat(target, BeatTime::QUARTER),
        ));

        self.scrolling_time = true;

        if let State::Playing { old_state, .. } = &self.state {
            self.state = (**old_state).clone()
        }
    }

    pub fn render_lights(
        &mut self,
        cursor_world_pos: vec2<Coord>,
        cursor_world_pos_snapped: vec2<Coord>,
        visualize_beat: bool,
        show_only_selected: bool,
    ) {
        let (static_time, dynamic_time) = if let State::Playing { .. } = self.state {
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
                self.selected_light
                    .and_then(|id| self.level.events.get(id.event))
                    .map(|selected| Level {
                        events: vec![selected.clone()],
                        timing: self.level.timing.clone(), // TODO: cheaper clone
                    })
            })
            .flatten();
        let level = selected_level.as_ref().unwrap_or(&self.level);

        let static_level =
            static_time.map(|time| LevelState::render(level, &self.model.level.config, time, None));
        let dynamic_level = dynamic_time
            .map(|time| LevelState::render(level, &self.model.level.config, time, None));

        let mut hovered_light = self.timeline_light_hover.take();
        if hovered_light.is_none() {
            if let State::Idle = self.state {
                if let Some(level) = &static_level {
                    hovered_light = level
                        .lights
                        .iter()
                        .find(|light| light.collider.contains(cursor_world_pos))
                        .and_then(|light| light.event_id)
                        .map(|event| LightId { event });
                }
            }
        }

        let mut waypoints = None;
        if let State::Waypoints { light_id, state } = &self.state {
            let light_id = *light_id;
            if let Some(timed_event) = self.level.events.get(light_id.event) {
                if let Event::Light(light_event) = &timed_event.event {
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
                        .timed_positions()
                        .map(|(i, trans_control, time)| {
                            let trans_actual = match i {
                                WaypointId::Initial => curve.get(0, FloatTime::ZERO),
                                WaypointId::Frame(i) => curve.get(i, FloatTime::ONE),
                            }
                            .unwrap_or(trans_control); // Should be unreachable, but just in case
                            (
                                Waypoint {
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

                    if let WaypointsState::New = state {
                        // NOTE: assuming that positions don't go backwards in time
                        // Insert a new waypoint at current time
                        let new_time = self.current_time.value - event_time;
                        let i = match points.binary_search_by_key(&new_time, |(_, time)| *time) {
                            Ok(i) | Err(i) => i,
                        };
                        let control = base_collider.transformed(Transform {
                            translation: cursor_world_pos_snapped,
                            rotation: self.place_rotation,
                            scale: self.place_scale,
                        });
                        points.insert(
                            i,
                            (
                                Waypoint {
                                    visible: true,
                                    original: None,
                                    actual: control.clone(),
                                    control,
                                },
                                new_time,
                            ),
                        );
                    }

                    let hovered = points
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
                        .map(|(i, _)| i);
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
