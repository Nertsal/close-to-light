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
    game::{PlayGroup, PlayLevel},
    leaderboard::Leaderboard,
    prelude::*,
    render::editor::{EditorRender, RenderOptions},
    ui::{widget::ConfirmPopup, UiContext},
};

#[derive(Debug)]
pub enum ConfirmAction {
    ExitUnsaved,
    ChangeLevelUnsaved(usize),
}

pub struct EditorState {
    context: Context,
    transition: Option<geng::state::Transition>,
    /// Stop music on the next `update` frame. Used when returning from F5 play to stop music.
    stop_music_next_frame: bool,
    render: EditorRender,
    editor: Editor,
    framebuffer_size: vec2<usize>,
    delta_time: Time,
    ui: EditorUI,
    ui_focused: bool,
    ui_context: UiContext,
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

pub struct LevelEditor {
    /// Static (initial) version of the level.
    pub static_level: PlayLevel,
    /// Current state of the level.
    pub level: Level,
    pub name: String,

    /// Simulation model.
    pub model: Model,
    pub level_state: EditorLevelState,
    pub current_beat: Time,
    pub real_time: Time,
    pub selected_light: Option<LightId>,

    /// State that will be saved in the undo stack.
    /// (Not every operation gets saved)
    pub buffer_state: Level,
    pub buffer_label: HistoryLabel,
    pub undo_stack: Vec<Level>,
    pub redo_stack: Vec<Level>,

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

    /// If `Some`, specifies the segment of the level to replay dynamically.
    pub dynamic_segment: Option<Replay>,
}

pub struct Editor {
    pub context: Context,
    pub options: Options,
    pub config: EditorConfig,
    pub render_options: RenderOptions,
    pub cursor_world_pos: vec2<Coord>,
    pub cursor_world_pos_snapped: vec2<Coord>,

    pub confirm_popup: Option<ConfirmPopup<ConfirmAction>>,

    /// Whether to exit the game on the next frame.
    pub exit: bool,

    pub grid_size: Coord,
    pub view_zoom: f32,
    pub music_timer: Time,
    pub snap_to_grid: bool,
    /// Whether to visualize the lights' movement for the current beat.
    pub visualize_beat: bool,

    pub group: PlayGroup,
    pub level_edit: Option<LevelEditor>,
}

#[derive(Debug)]
pub struct Replay {
    pub start_beat: Time,
    pub end_beat: Time,
    pub current_beat: Time,
    pub speed: Time,
}

impl LevelEditor {
    pub fn new(model: Model, level: PlayLevel, visualize_beat: bool) -> Self {
        let mut editor = Self {
            level_state: EditorLevelState::default(),
            current_beat: Time::ZERO,
            real_time: Time::ZERO,
            selected_light: None,
            place_rotation: Angle::ZERO,
            place_scale: Coord::ONE,
            state: State::Idle,
            was_scrolling_time: false,
            scrolling_time: false,
            dynamic_segment: None,
            buffer_state: level.level.data.clone(),
            buffer_label: HistoryLabel::default(),
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            level: level.level.data.clone(),
            name: level.level.meta.name.to_string(),
            static_level: level,
            model,
        };
        editor.render_lights(vec2::ZERO, vec2::ZERO, visualize_beat);
        editor
    }
}

impl EditorState {
    pub fn new_group(
        context: Context,
        config: EditorConfig,
        options: Options,
        group: PlayGroup,
    ) -> Self {
        Self {
            transition: None,
            stop_music_next_frame: true,
            render: EditorRender::new(&context.geng, &context.assets),
            framebuffer_size: vec2(1, 1),
            delta_time: r32(0.1),
            ui: EditorUI::new(&context.geng, &context.assets),
            ui_focused: false,
            ui_context: UiContext::new(&context.geng, options.theme),
            drag: None,
            editor: Editor {
                context: context.clone(),
                render_options: RenderOptions {
                    show_grid: true,
                    hide_ui: false,
                },
                cursor_world_pos: vec2::ZERO,
                cursor_world_pos_snapped: vec2::ZERO,

                confirm_popup: None,

                exit: false,

                grid_size: r32(10.0) / config.grid.height,
                view_zoom: 1.0,
                visualize_beat: true,
                snap_to_grid: true,
                music_timer: Time::ZERO,

                group,
                level_edit: None,
                options,
                config,
            },
            context,
        }
    }

    pub fn new_level(
        context: Context,
        config: EditorConfig,
        options: Options,
        level: PlayLevel,
    ) -> Self {
        let mut editor = Self::new_group(
            context.clone(),
            config,
            options.clone(),
            level.group.clone(),
        );
        let model = Model::empty(context, options, level.clone());
        editor.editor.level_edit = Some(LevelEditor::new(model, level, true));
        editor
    }

    fn snap_pos_grid(&self, pos: vec2<Coord>) -> vec2<Coord> {
        (pos / self.editor.grid_size).map(Coord::round) * self.editor.grid_size
    }

    fn update_level_editor(&mut self, delta_time: Time) {
        let Some(level_editor) = &mut self.editor.level_edit else {
            return;
        };

        self.context
            .music
            .set_volume(level_editor.model.options.volume.music());

        level_editor.real_time += delta_time;

        if self.editor.music_timer > Time::ZERO {
            self.editor.music_timer -= delta_time;
            if self.editor.music_timer <= Time::ZERO {
                self.context.music.stop();
            }
        }

        if let Some(waypoints) = &level_editor.level_state.waypoints {
            if let Some(waypoint) = waypoints.selected {
                if let Some(event) = level_editor.level.events.get(waypoints.event) {
                    if let Event::Light(light) = &event.event {
                        // Set current time to align with the selected waypoint
                        if let Some(time) = light.light.movement.get_time(waypoint) {
                            level_editor.current_beat =
                                event.beat + light.telegraph.precede_time + time;
                        }
                    }
                }
            }
        }

        if level_editor.scrolling_time {
            level_editor.was_scrolling_time = true;
        } else {
            if level_editor.was_scrolling_time {
                // Stopped scrolling
                // Play some music
                self.context.music.play_from_beat(
                    &level_editor.static_level.group.music,
                    level_editor.current_beat,
                );
                self.editor.music_timer = level_editor.static_level.group.music.meta.beat_time()
                    * self.editor.config.playback_duration;
            }
            level_editor.was_scrolling_time = false;
        }

        level_editor.scrolling_time = false;

        if let State::Playing { .. } = level_editor.state {
            level_editor.current_beat =
                level_editor.real_time / level_editor.static_level.group.music.meta.beat_time();
        } else if let Some(replay) = &mut level_editor.dynamic_segment {
            replay.current_beat +=
                replay.speed * delta_time / level_editor.static_level.group.music.meta.beat_time();
            if replay.current_beat > replay.end_beat {
                replay.current_beat = replay.start_beat;
            }
        }

        level_editor.render_lights(
            self.editor.cursor_world_pos,
            self.editor.cursor_world_pos_snapped,
            self.editor.visualize_beat,
        );

        let pos = self.ui_context.cursor.position;
        let pos = pos - self.ui.screen.position.bottom_left();
        let pos = level_editor
            .model
            .camera
            .screen_to_world(self.ui.screen.position.size(), pos)
            .as_r32();
        self.editor.cursor_world_pos = pos;
        self.editor.cursor_world_pos_snapped = if self.editor.snap_to_grid {
            self.snap_pos_grid(pos)
        } else {
            pos
        };
    }

    /// Start playing the game from the current time.
    fn play_game(&mut self) {
        let Some(level_editor) = &self.editor.level_edit else {
            return;
        };

        let level = crate::game::PlayLevel {
            start_time: level_editor.current_beat
                * level_editor.static_level.group.music.meta.beat_time(), // TODO: nonlinear time
            level: Rc::new(LevelFull {
                meta: level_editor.static_level.level.meta.clone(),
                data: level_editor.level.clone(),
            }),
            ..level_editor.static_level.clone()
        };

        self.transition = Some(geng::state::Transition::Push(Box::new(
            crate::game::Game::new(
                self.context.clone(),
                level_editor.model.options.clone(),
                level,
                Leaderboard::new(&self.context.geng, None),
            ),
        )));
        self.stop_music_next_frame = true;
    }
}

impl geng::State for EditorState {
    fn transition(&mut self) -> Option<geng::state::Transition> {
        self.transition.take()
    }

    fn update(&mut self, delta_time: f64) {
        let delta_time = Time::new(delta_time as f32);
        self.delta_time = delta_time;

        if self.transition.is_none() && std::mem::take(&mut self.stop_music_next_frame) {
            self.context.music.stop();
        }

        self.ui_context
            .update(self.context.geng.window(), delta_time.as_f32());

        self.update_drag();

        self.update_level_editor(delta_time);

        if std::mem::take(&mut self.editor.exit) {
            self.transition = Some(geng::state::Transition::Pop);
        }
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
            &mut self.ui_context,
        );
        self.ui_context.frame_end();

        if let Some(level_editor) = &mut self.editor.level_edit {
            level_editor.model.camera.fov = 10.0 / self.editor.view_zoom;
        }
        self.render.draw_editor(&self.editor, &self.ui, framebuffer);
    }
}

impl Editor {
    fn delete_active_level(&mut self) {
        let Some(level_editor) = self.level_edit.take() else {
            return;
        };
        let level_index = level_editor.static_level.level_index;

        if !(0..self.group.cached.data.levels.len()).contains(&level_index) {
            log::error!(
                "Tried to remove a level by an invalid index {}",
                level_index
            );
            return;
        }

        let mut new_group = self.group.cached.data.clone();
        new_group.levels.remove(level_index);

        if let Some(group) =
            self.context
                .local
                .update_group(self.group.group_index, new_group, None)
        {
            self.group.cached = group;
            log::info!("Saved the level successfully");
        } else {
            log::error!("Failed to update the level cache");
        }
    }

    fn create_new_level(&mut self) {
        let mut new_group = self.group.cached.data.clone();
        new_group.levels.push(Rc::new(LevelFull {
            meta: LevelInfo {
                id: 0,
                name: "New Diff".into(),
                authors: Vec::new(),
                hash: String::new(),
            },
            data: Level::new(),
        }));

        if let Some(group) =
            self.context
                .local
                .update_group(self.group.group_index, new_group, None)
        {
            self.group.cached = group;
            log::info!("Saved the level successfully");
        } else {
            log::error!("Failed to update the level cache");
        }
    }

    fn move_level_low(&mut self, level_index: usize) {
        let Some(swap_with) = level_index.checked_sub(1) else {
            return;
        };
        self.swap_levels(level_index, swap_with);
    }

    fn move_level_high(&mut self, level_index: usize) {
        self.swap_levels(level_index, level_index + 1);
    }

    fn swap_levels(&mut self, i: usize, j: usize) {
        let levels = &self.group.cached.data.levels;
        if !(0..levels.len()).contains(&i) || !(0..levels.len()).contains(&j) {
            log::error!("Invalid indices to swap levels");
            return;
        }

        let mut new_group = self.group.cached.data.clone();
        new_group.levels.swap(i, j);

        if let Some(group) =
            self.context
                .local
                .update_group(self.group.group_index, new_group, None)
        {
            if let Some(level_editor) = &mut self.level_edit {
                let active = &mut level_editor.static_level.level_index;
                if i == *active {
                    *active = j;
                } else if j == *active {
                    *active = i;
                }
            }
            self.group.cached = group;
            log::info!("Saved the level successfully");
        } else {
            log::error!("Failed to update the level cache");
        }
    }

    fn change_level(&mut self, level_index: usize) {
        if let Some(_level_editor) = self.level_edit.take() {
            // TODO: check unsaved changes
        }

        if let Some(level) = self.group.cached.data.levels.get(level_index) {
            log::debug!("Changing to level {}", level.meta.name);

            let level = PlayLevel {
                group: self.group.clone(),
                level_index,
                level: level.clone(),
                config: LevelConfig::default(),
                start_time: Time::ZERO,
            };
            let model = Model::empty(self.context.clone(), self.options.clone(), level.clone());
            self.level_edit = Some(LevelEditor::new(model, level, self.visualize_beat));
        }
    }

    /// Exit the editor.
    fn exit(&mut self) {
        // TODO: check unsaved changes
        self.exit = true;
    }

    fn save(&mut self) {
        let Some(level_editor) = &mut self.level_edit else {
            return;
        };

        if let Some((group, level)) = self.context.local.update_level(
            level_editor.static_level.group.group_index,
            level_editor.static_level.level_index,
            level_editor.level.clone(),
            level_editor.name.clone(),
        ) {
            level_editor.model.level.level = level;
            self.group.cached = group;
            log::info!("Saved the level successfully");
        } else {
            log::error!("Failed to update the level cache");
        }
    }

    /// Check whether the level has been changed.
    fn is_changed(&self) -> bool {
        if let Some(level_editor) = &self.level_edit {
            let Some(cached) = self
                .group
                .cached
                .data
                .levels
                .get(level_editor.static_level.level_index)
            else {
                return true;
            };
            let level_changed =
                level_editor.level != cached.data || *level_editor.name != *cached.meta.name;
            if level_changed {
                return true;
            }
        }
        false
    }

    /// Create a popup window with a message for the given action.
    fn popup_confirm(&mut self, action: ConfirmAction, message: impl Into<Name>) {
        self.confirm_popup = Some(ConfirmPopup {
            action,
            title: "Are you sure?".into(),
            message: message.into(),
        });
    }

    /// Confirm the popup action and execute it.
    fn confirm_action(&mut self, _ui: &mut EditorUI) {
        let Some(popup) = self.confirm_popup.take() else {
            return;
        };
        match popup.action {
            ConfirmAction::ExitUnsaved => self.exit(),
            ConfirmAction::ChangeLevelUnsaved(index) => self.change_level(index),
        }
    }
}

impl LevelEditor {
    fn undo(&mut self) {
        match &mut self.state {
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
                if let Some(mut level) = self.undo_stack.pop() {
                    std::mem::swap(&mut level, &mut self.level);
                    self.redo_stack.push(level);
                    self.buffer_state = self.level.clone();
                    self.buffer_label = HistoryLabel::default();
                }
            }
        }
    }

    fn redo(&mut self) {
        match &mut self.state {
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
                if let Some(mut level) = self.redo_stack.pop() {
                    std::mem::swap(&mut level, &mut self.level);
                    self.undo_stack.push(level);
                    self.buffer_state = self.level.clone();
                    self.buffer_label = HistoryLabel::default();
                }
            }
        }
    }

    fn save_state(&mut self, label: HistoryLabel) {
        if self.buffer_label.should_merge(&label) || self.level == self.buffer_state {
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
        self.buffer_label = label;
        let mut state = self.level.clone();
        std::mem::swap(&mut state, &mut self.buffer_state);

        self.undo_stack.push(state);
        // TODO: limit capacity
        self.redo_stack.clear();
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
        let max = margin + self.level.last_beat();
        let target = (self.current_beat + delta).clamp(min, max);

        // Align with quarter beats
        self.current_beat = ((target.as_f32() * 4.0).round() / 4.0).as_r32();

        self.scrolling_time = true;
    }

    pub fn render_lights(
        &mut self,
        cursor_world_pos: vec2<Coord>,
        cursor_world_pos_snapped: vec2<Coord>,
        visualize_beat: bool,
    ) {
        let (static_time, dynamic_time) = if let State::Playing { .. } = self.state {
            // TODO: self.music.play_position()
            (None, Some(self.current_beat))
        } else {
            let time = self.current_beat;
            let dynamic = if visualize_beat {
                if let Some(replay) = &self.dynamic_segment {
                    Some(replay.current_beat)
                } else {
                    Some(
                        time + (self.real_time / self.static_level.group.music.meta.beat_time())
                            .fract(),
                    )
                }
            } else {
                None
            };
            (Some(time), dynamic)
        };

        let static_level = static_time
            .map(|time| LevelState::render(&self.level, &self.model.level.config, time, None));
        let dynamic_level = dynamic_time
            .map(|time| LevelState::render(&self.level, &self.model.level.config, time, None));

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
                    .position(|light| light.collider.contains(cursor_world_pos));
            }
        }

        let mut waypoints = None;
        if let State::Waypoints { event, state } = &self.state {
            let event_id = *event;
            if let Some(event) = self.level.events.get(event_id) {
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
                            (event_time + event.telegraph.precede_time + *time - self.current_beat)
                                .abs(),
                        )
                    });
                    // event.beat + light.telegraph.precede_time + beat
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
                                        translation: cursor_world_pos_snapped,
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
                        point.visible && point.collider.contains(cursor_world_pos)
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
