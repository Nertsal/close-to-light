mod config;
mod handle_event;
mod state;
mod ui;

pub use self::{
    config::*,
    state::{State, *},
    ui::*,
};

use crate::{
    game::PlayLevel,
    leaderboard::Leaderboard,
    prelude::*,
    render::editor::{EditorRender, RenderOptions},
    ui::widget::CursorContext,
};

use geng::MouseButton;

pub struct EditorState {
    geng: Geng,
    assets: Rc<Assets>,
    transition: Option<geng::state::Transition>,
    render: EditorRender,
    editor: Editor,
    framebuffer_size: vec2<usize>,
    delta_time: Time,
    cursor: CursorContext,
    ui: EditorUI,
    ui_focused: bool,
    drag: Option<Drag>,
}

#[derive(Debug)]
pub struct Drag {
    /// Whether we just clicked or actually starting moving.
    pub moved: bool,
    pub from_screen: vec2<f32>,
    pub from_world: vec2<Coord>,
    pub from_time: Time,
    pub target: DragTarget,
}

#[derive(Debug)]
pub enum DragTarget {
    /// Move the whole light event through time and space.
    Light {
        event: usize,
        initial_time: Time,
        initial_translation: vec2<Coord>,
    },
    Waypoint {
        event: usize,
        waypoint: WaypointId,
        initial_translation: vec2<Coord>,
    },
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum HistoryLabel {
    #[default]
    Unknown,
    Scroll,
    // Drag,
}

impl HistoryLabel {
    pub fn should_merge(&self, other: &Self) -> bool {
        match self {
            Self::Unknown => false,
            _ => self == other,
        }
    }
}

pub struct Editor {
    pub config: EditorConfig,
    pub render_options: RenderOptions,
    pub cursor_world_pos: vec2<Coord>,

    pub level: PlayLevel,

    /// Simulation model.
    pub model: Model,
    pub level_state: EditorLevelState,
    pub grid_size: Coord,
    pub current_beat: Time,
    pub real_time: Time,
    pub selected_light: Option<LightId>,

    pub buffer_state: Level,
    pub buffer_label: HistoryLabel,
    pub undo_stack: Vec<Level>,
    pub redo_stack: Vec<Level>,

    /// At what rotation the objects should be placed.
    pub place_rotation: Angle<Coord>,
    /// The scale at which the objects should be placed.
    pub place_scale: Coord,

    pub view_zoom: f32,
    pub state: State,
    /// Whether the last frame was scrolled through time.
    pub was_scrolling_time: bool,
    /// Whether currently scrolling through time.
    /// Used as a hack to not replay the music every frame.
    pub scrolling_time: bool,

    pub snap_to_grid: bool,
    /// Whether to visualize the lights' movement for the current beat.
    pub visualize_beat: bool,
    /// If `Some`, specifies the segment of the level to replay dynamically.
    pub dynamic_segment: Option<Replay>,
}

#[derive(Debug)]
pub struct Replay {
    pub start_beat: Time,
    pub end_beat: Time,
    pub current_beat: Time,
    pub speed: Time,
}

impl EditorState {
    pub fn new(
        geng: Geng,
        assets: Rc<Assets>,
        config: EditorConfig,
        options: Options,
        level: PlayLevel,
    ) -> Self {
        let model = Model::empty(&assets, options, level.clone());
        Self {
            transition: None,
            render: EditorRender::new(&geng, &assets),
            framebuffer_size: vec2(1, 1),
            delta_time: r32(0.1),
            cursor: CursorContext::new(),
            ui: EditorUI::new(&assets),
            ui_focused: false,
            drag: None,
            editor: Editor {
                render_options: RenderOptions {
                    show_grid: true,
                    hide_ui: false,
                },
                grid_size: r32(10.0) / config.grid.height,
                cursor_world_pos: vec2::ZERO,
                level_state: EditorLevelState::default(),
                current_beat: Time::ZERO,
                real_time: Time::ZERO,
                selected_light: None,
                place_rotation: Angle::ZERO,
                place_scale: Coord::ONE,
                view_zoom: 1.0,
                state: State::Idle,
                was_scrolling_time: false,
                scrolling_time: false,
                visualize_beat: true,
                dynamic_segment: None,
                snap_to_grid: true,
                buffer_state: level.level.clone(),
                buffer_label: HistoryLabel::default(),
                undo_stack: Vec::new(),
                redo_stack: Vec::new(),
                config,
                model,
                level,
            },
            geng,
            assets,
        }
    }

    fn snap_pos_grid(&self, pos: vec2<Coord>) -> vec2<Coord> {
        (pos / self.editor.grid_size).map(Coord::round) * self.editor.grid_size
    }

    /// Start playing the game from the current time.
    fn play_game(&mut self) {
        let level = crate::game::PlayLevel {
            start_time: self.editor.current_beat * self.editor.level.music.beat_time(), // TODO: nonlinear time
            ..self.editor.level.clone()
        };
        self.transition = Some(geng::state::Transition::Push(Box::new(
            crate::game::Game::new(
                &self.geng,
                &self.assets,
                self.editor.model.options.clone(),
                level,
                Leaderboard::new(None),
                String::new(),
            ),
        )));
    }

    fn undo(&mut self) {
        match &mut self.editor.state {
            State::Playing { .. } => {}
            State::Movement {
                light, redo_stack, ..
            } => {
                if let Some(frame) = light.light.movement.key_frames.pop_back() {
                    redo_stack.push(frame);
                }
            }
            State::Place { .. } => {}
            State::Idle | State::Waypoints { .. } => {
                if let Some(mut level) = self.editor.undo_stack.pop() {
                    std::mem::swap(&mut level, &mut self.editor.level.level);
                    self.editor.redo_stack.push(level);
                    self.editor.buffer_state = self.editor.level.level.clone();
                    self.editor.buffer_label = HistoryLabel::default();
                }
            }
        }
    }

    fn redo(&mut self) {
        match &mut self.editor.state {
            State::Playing { .. } => {}
            State::Movement {
                light, redo_stack, ..
            } => {
                if let Some(frame) = redo_stack.pop() {
                    light.light.movement.key_frames.push_back(frame);
                }
            }
            State::Place { .. } => {}
            State::Idle | State::Waypoints { .. } => {
                if let Some(mut level) = self.editor.redo_stack.pop() {
                    std::mem::swap(&mut level, &mut self.editor.level.level);
                    self.editor.undo_stack.push(level);
                    self.editor.buffer_state = self.editor.level.level.clone();
                    self.editor.buffer_label = HistoryLabel::default();
                }
            }
        }
    }

    fn save_state(&mut self, label: HistoryLabel) {
        if self.editor.buffer_label.should_merge(&label)
            || self.editor.level.level == self.editor.buffer_state
        {
            // State did not change or changes should be merged
            return;
        }

        // if let Some(level) = self.editor.undo_stack.last() {
        //     if level == &self.editor.level {
        //         // State did not change - ignore
        //         return;
        //     }
        // }

        // Push the change
        self.editor.buffer_label = label;
        let mut state = self.editor.level.level.clone();
        std::mem::swap(&mut state, &mut self.editor.buffer_state);

        self.editor.undo_stack.push(state);
        // TODO: limit capacity
        self.editor.redo_stack.clear();
    }

    fn save(&mut self) {
        let path = self.editor.level.level_path();
        let result = (|| -> anyhow::Result<()> {
            // TODO: switch back to ron
            // https://github.com/geng-engine/geng/issues/71
            let level = serde_json::to_string_pretty(&self.editor.level.level)?;
            let mut writer = std::io::BufWriter::new(std::fs::File::create(&path)?);
            write!(writer, "{}", level)?;
            Ok(())
        })();
        match result {
            Ok(()) => {
                self.editor.model.level = self.editor.level.clone();
                log::info!("Saved the level successfully");
            }
            Err(err) => {
                log::error!("Failed to save the level at {:?}: {:?}", path, err);
            }
        }
    }
}

impl geng::State for EditorState {
    fn transition(&mut self) -> Option<geng::state::Transition> {
        self.transition.take()
    }

    fn update(&mut self, delta_time: f64) {
        let delta_time = Time::new(delta_time as f32);
        self.delta_time = delta_time;
        self.editor.real_time += delta_time;

        self.editor
            .level
            .music
            .set_volume(self.editor.model.options.volume.music());

        self.cursor.update(geng_utils::key::is_key_pressed(
            self.geng.window(),
            [MouseButton::Left],
        ));

        self.update_drag();

        if self.editor.level.music.timer > Time::ZERO {
            self.editor.level.music.timer -= delta_time;
            if self.editor.level.music.timer <= Time::ZERO {
                self.editor.level.music.stop();
            }
        }

        if let Some(waypoints) = &self.editor.level_state.waypoints {
            if let Some(waypoint) = waypoints.selected {
                if let Some(event) = self.editor.level.level.events.get(waypoints.event) {
                    if let Event::Light(light) = &event.event {
                        // Set current time to align with the selected waypoint
                        if let Some(time) = light.light.movement.get_time(waypoint) {
                            self.editor.current_beat =
                                event.beat + light.telegraph.precede_time + time;
                        }
                    }
                }
            }
        }

        if self.editor.scrolling_time {
            self.editor.was_scrolling_time = true;
        } else {
            if self.editor.was_scrolling_time {
                // Stopped scrolling
                // Play some music
                self.editor
                    .level
                    .music
                    .play_from(time::Duration::from_secs_f64(
                        (self.editor.current_beat * self.editor.level.music.beat_time()).as_f32()
                            as f64,
                    ));
                self.editor.level.music.timer =
                    self.editor.level.music.beat_time() * self.editor.config.playback_duration;
            }
            self.editor.was_scrolling_time = false;
        }

        self.editor.scrolling_time = false;

        if let State::Playing { .. } = self.editor.state {
            self.editor.current_beat = self.editor.real_time / self.editor.level.music.beat_time();
        } else if let Some(replay) = &mut self.editor.dynamic_segment {
            replay.current_beat += replay.speed * delta_time / self.editor.level.music.beat_time();
            if replay.current_beat > replay.end_beat {
                replay.current_beat = replay.start_beat;
            }
        }

        let pos = self.cursor.position;
        let pos = pos - self.ui.screen.position.bottom_left();
        let pos = self
            .editor
            .model
            .camera
            .screen_to_world(self.ui.screen.position.size(), pos)
            .as_r32();
        self.editor.cursor_world_pos = if self.editor.snap_to_grid {
            self.snap_pos_grid(pos)
        } else {
            pos
        };

        self.editor.render_lights(self.editor.visualize_beat);
    }

    fn handle_event(&mut self, event: geng::Event) {
        self.handle_event(event);
    }

    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        self.framebuffer_size = framebuffer.size();
        ugli::clear(framebuffer, Some(Color::BLACK), None, None);

        self.ui_focused = !self.ui.layout(
            &mut self.editor,
            Aabb2::ZERO.extend_positive(framebuffer.size().as_f32()),
            self.cursor,
            self.delta_time,
            &self.geng,
        );
        self.cursor.scroll = 0.0;
        self.editor.model.camera.fov = 10.0 / self.editor.view_zoom;
        self.render.draw_editor(&self.editor, &self.ui, framebuffer);
    }
}

impl Editor {
    /// Swap the palette at current time.
    fn palette_swap(&mut self) {
        // Remove any already existing palette swap event at current time
        let mut ids = Vec::new();
        for (i, event) in self.level.level.events.iter().enumerate() {
            if event.beat == self.current_beat {
                if let Event::PaletteSwap = event.event {
                    ids.push(i);
                }
            }
        }

        let add = ids.len() % 2 == 0;

        // Remove events
        for i in ids.into_iter().rev() {
            self.level.level.events.swap_remove(i);
        }

        if add {
            // Add a new palette swap event
            self.level.level.events.push(TimedEvent {
                beat: self.current_beat,
                event: Event::PaletteSwap,
            });
        }
    }

    fn new_light_circle(&mut self) {
        self.state = State::Place {
            shape: Shape::Circle { radius: r32(1.3) },
            danger: false,
        };
    }

    fn new_light_line(&mut self) {
        self.state = State::Place {
            shape: Shape::Line { width: r32(1.7) },
            danger: false,
        };
    }

    fn view_waypoints(&mut self) {
        match self.state {
            State::Idle => {
                if let Some(selected) = self.selected_light {
                    self.state = State::Waypoints {
                        event: selected.event,
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

    fn scroll_time(&mut self, delta: Time) {
        let margin = r32(10.0);
        let min = Time::ZERO;
        let max = margin + self.level.level.last_beat();
        let target = (self.current_beat + delta).clamp(min, max);

        // Align with quarter beats
        self.current_beat = ((target.as_f32() * 4.0).round() / 4.0).as_r32();

        self.scrolling_time = true;
    }

    pub fn render_lights(&mut self, visualize_beat: bool) {
        let (static_time, dynamic_time) = if let State::Playing { .. } = self.state {
            // TODO: self.music.play_position()
            (None, Some(self.current_beat))
        } else {
            let time = self.current_beat;
            let dynamic = if visualize_beat {
                if let Some(replay) = &self.dynamic_segment {
                    Some(replay.current_beat)
                } else {
                    Some(time + (self.real_time / self.level.music.beat_time()).fract())
                }
            } else {
                None
            };
            (Some(time), dynamic)
        };

        let static_level = static_time.map(|time| {
            LevelState::render(&self.level.level, &self.model.level.config, time, None)
        });
        let dynamic_level = dynamic_time.map(|time| {
            LevelState::render(&self.level.level, &self.model.level.config, time, None)
        });

        // if let State::Movement {
        //     start_beat, light, ..
        // } = &self.state
        // {
        //     let event = commit_light(light.clone());
        //     let event = TimedEvent {
        //         beat: *start_beat,
        //         event: Event::Light(event),
        //     };
        //     for level in [&mut static_level, &mut dynamic_level]
        //         .into_iter()
        //         .flatten()
        //     {
        //         level.render_event(&event, None);
        //     }
        // }

        let mut hovered_light = None;
        if let State::Idle = self.state {
            if let Some(level) = &static_level {
                hovered_light = level
                    .lights
                    .iter()
                    .position(|light| light.collider.contains(self.cursor_world_pos));
            }
        }

        let mut waypoints = None;
        if let State::Waypoints { event, state } = &self.state {
            let event_id = *event;
            if let Some(event) = self.level.level.events.get(event_id) {
                let event_time = event.beat;
                if let Event::Light(event) = &event.event {
                    // If some waypoints overlap, render the temporaly closest one
                    let base_collider = Collider::new(vec2::ZERO, event.light.shape);
                    let mut points: Vec<_> = event
                        .light
                        .movement
                        .timed_positions()
                        .map(|(i, trans, time)| {
                            (
                                Waypoint {
                                    visible: true,
                                    original: Some(i),
                                    collider: base_collider.transformed(trans),
                                },
                                time,
                            )
                        })
                        .collect();
                    points.sort_by_key(|(point, time)| {
                        (
                            point.collider.position.x,
                            point.collider.position.y,
                            (event_time + *time - self.current_beat).abs(),
                        )
                    });
                    {
                        let mut points = points.iter_mut();
                        if let Some(last) = points.next() {
                            let mut last = last.0.collider.position;
                            for (point, _) in points {
                                let pos = point.collider.position;
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
                        let new_time = self.current_beat - event_time;
                        let i = match points.binary_search_by_key(&new_time, |(_, time)| *time) {
                            Ok(i) | Err(i) => i,
                        };
                        points.insert(
                            i,
                            (
                                Waypoint {
                                    visible: true,
                                    original: None,
                                    collider: base_collider.transformed(Transform {
                                        translation: self.cursor_world_pos,
                                        rotation: self.place_rotation,
                                        scale: self.place_scale,
                                    }),
                                },
                                new_time,
                            ),
                        );
                    }

                    let points: Vec<_> = points.into_iter().map(|(point, _)| point).collect();

                    let hovered = points.iter().position(|point| {
                        point.visible && point.collider.contains(self.cursor_world_pos)
                    });

                    waypoints = Some(Waypoints {
                        event: event_id,
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
