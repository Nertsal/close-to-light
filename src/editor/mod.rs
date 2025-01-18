mod action;
mod config;
mod handle_event;
mod history;
mod state;
mod ui;

pub use self::{
    action::*,
    config::*,
    history::*,
    state::{State, *},
    ui::*,
};

use crate::{
    game::{PlayGroup, PlayLevel},
    leaderboard::Leaderboard,
    prelude::*,
    render::editor::{EditorRender, RenderOptions},
    ui::{widget::ConfirmPopup, UiContext},
    util::{SecondOrderDynamics, SecondOrderState},
};

#[derive(Debug, Clone)]
pub enum ConfirmAction {
    ExitUnsaved,
    ChangeLevelUnsaved(usize),
    DeleteLevel(usize),
}

pub struct EditorState {
    context: Context,
    transition: Option<geng::state::Transition>,
    /// Stop music on the next `update` frame. Used when returning from F5 play to stop music.
    stop_music_next_frame: bool,
    render: EditorRender,
    editor: Editor,
    framebuffer_size: vec2<usize>,
    delta_time: FloatTime,
    ui: EditorUi,
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
    pub from_real_time: FloatTime,
    pub from_beat: Time,
    pub target: DragTarget,
}

#[derive(Debug, Clone)]
pub enum DragTarget {
    /// Move the whole light event through time and space.
    Light {
        /// Whether it was the second click on the light.
        /// If the drag is short, waypoints will be toggled.
        double: bool,
        light: LightId,
        initial_time: Time,
        initial_translation: vec2<Coord>,
    },
    Waypoint {
        light: LightId,
        waypoint: WaypointId,
        initial_translation: vec2<Coord>,
    },
}

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorTab {
    Edit,
    Config,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ScrollSpeed {
    Slow,
    Normal,
    Fast,
}

pub struct Editor {
    pub context: Context,
    pub config: EditorConfig,
    pub render_options: RenderOptions,
    pub cursor_world_pos: vec2<Coord>,
    pub cursor_world_pos_snapped: vec2<Coord>,

    pub confirm_popup: Option<ConfirmPopup<ConfirmAction>>,

    pub tab: EditorTab,
    /// Whether to exit the editor on the next frame.
    pub exit: bool,

    pub grid_size: Coord,
    pub view_zoom: SecondOrderState<f32>,
    pub music_timer: FloatTime,
    pub snap_to_grid: bool,
    /// Whether to visualize the lights' movement for the current beat.
    pub visualize_beat: bool,
    /// Whether to only render the selected light.
    pub show_only_selected: bool,

    pub group: PlayGroup,
    pub level_edit: Option<LevelEditor>,
}

pub struct TimeInterpolation {
    state: SecondOrderState<FloatTime>,
    pub value: Time,
    pub target: Time,
}

impl TimeInterpolation {
    pub fn new() -> Self {
        let time = Time::ZERO;
        Self {
            state: SecondOrderState::new(SecondOrderDynamics::new(
                3.0,
                1.0,
                0.0,
                time_to_seconds(time),
            )),
            value: time,
            target: time,
        }
    }

    pub fn update(&mut self, delta_time: FloatTime) {
        self.state.update(delta_time.as_f32());
        self.value = seconds_to_time(self.state.current);
    }

    pub fn scroll_time(&mut self, change: Change<Time>) {
        change.apply(&mut self.target);
        self.state.target = time_to_seconds(self.target);
    }

    pub fn snap_to(&mut self, time: Time) {
        self.value = time;
        self.target = time;
        self.state.current = time_to_seconds(self.value);
        self.state.target = time_to_seconds(self.target);
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

    fn delete_light(&mut self, id: LightId) {
        if id.event >= self.level.events.len() {
            return;
        }
        self.level.events.swap_remove(id.event);
        self.selected_light = None;
        self.save_state(default());
    }

    fn delete_waypoint(&mut self, light: LightId, waypoint: WaypointId) {
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
}

impl EditorState {
    pub fn new_group(context: Context, config: EditorConfig, group: PlayGroup) -> Self {
        Self {
            transition: None,
            stop_music_next_frame: true,
            render: EditorRender::new(context.clone()),
            framebuffer_size: vec2(1, 1),
            delta_time: r32(0.1),
            ui: EditorUi::new(context.clone()),
            ui_focused: false,
            ui_context: UiContext::new(context.clone()),
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

                tab: EditorTab::Edit,
                exit: false,

                grid_size: r32(10.0) / config.grid.height,
                view_zoom: SecondOrderState::new(SecondOrderDynamics::new(3.0, 1.0, 1.0, 1.0)),
                visualize_beat: true,
                show_only_selected: false,
                snap_to_grid: true,
                music_timer: FloatTime::ZERO,

                group,
                level_edit: None,
                config,
            },
            context,
        }
    }

    pub fn new_level(context: Context, config: EditorConfig, level: PlayLevel) -> Self {
        let mut editor = Self::new_group(context.clone(), config, level.group.clone());
        let options = context.get_options();
        let model = Model::empty(context.clone(), options, level.clone());
        editor.editor.level_edit = Some(LevelEditor::new(context, model, level, true, false));
        editor
    }

    fn snap_pos_grid(&self, pos: vec2<Coord>) -> vec2<Coord> {
        (pos / self.editor.grid_size).map(Coord::round) * self.editor.grid_size
    }

    fn update_level_editor(&mut self, delta_time: FloatTime) {
        let Some(level_editor) = &mut self.editor.level_edit else {
            return;
        };

        self.context
            .music
            .set_volume(level_editor.model.options.volume.music());

        level_editor.real_time += delta_time;
        level_editor.current_time.update(delta_time);
        level_editor.timeline_zoom.update(delta_time.as_f32());

        if self.editor.music_timer > FloatTime::ZERO {
            self.editor.music_timer -= delta_time;
            if self.editor.music_timer <= FloatTime::ZERO {
                self.context.music.stop();
            }
        }

        // TODO: maybe config option?
        // if let Some(waypoints) = &level_editor.level_state.waypoints {
        //     if let Some(waypoint) = waypoints.selected {
        //         if let Some(event) = level_editor.level.events.get(waypoints.light.event) {
        //             if let Event::Light(light) = &event.event {
        //                 // Set current time to align with the selected waypoint
        //                 if let Some(time) = light.movement.get_time(waypoint) {
        //                     level_editor.current_time = event.time + time;
        //                 }
        //             }
        //         }
        //     }
        // }

        if level_editor.scrolling_time {
            level_editor.was_scrolling_time = true;
        } else {
            if level_editor.was_scrolling_time {
                // Stopped scrolling
                // Play some music
                self.context.music.play_from_beat(
                    &level_editor.static_level.group.music,
                    level_editor.current_time.value,
                );
                self.editor.music_timer = self.editor.config.playback_duration;
            }
            level_editor.was_scrolling_time = false;
        }

        level_editor.scrolling_time = false;

        if let State::Playing { .. } = level_editor.state {
            level_editor
                .current_time
                .snap_to(seconds_to_time(level_editor.real_time));
        }

        level_editor.render_lights(
            self.editor.cursor_world_pos,
            self.editor.cursor_world_pos_snapped,
            self.editor.visualize_beat,
            self.editor.show_only_selected,
        );

        let pos = self.ui_context.cursor.position;
        let pos = pos - self.ui_context.screen.bottom_left();
        let pos = level_editor
            .model
            .camera
            .screen_to_world(self.ui_context.screen.size(), pos)
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
            start_time: level_editor.current_time.value,
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
        let delta_time = FloatTime::new(delta_time as f32);
        self.delta_time = delta_time;

        self.context
            .geng
            .window()
            .set_cursor_type(geng::CursorType::Default);

        if self.transition.is_none() && std::mem::take(&mut self.stop_music_next_frame) {
            self.context.music.stop();
        }

        self.ui_context
            .update(self.context.geng.window(), delta_time.as_f32());
        self.editor.view_zoom.update(delta_time.as_f32());

        for action in self.update_drag() {
            self.execute(action);
        }

        self.update_level_editor(delta_time);

        if std::mem::take(&mut self.editor.exit) {
            self.execute(EditorStateAction::Exit);
        }
    }

    fn handle_event(&mut self, event: geng::Event) {
        let actions = self.process_event(event);
        for action in actions {
            self.execute(action);
        }
    }

    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        self.framebuffer_size = framebuffer.size();
        ugli::clear(framebuffer, Some(Color::BLACK), None, None);

        self.ui_context.state.frame_start();
        self.ui_context.geometry.update(framebuffer.size());
        let (can_focus, actions) = self.ui.layout(
            &self.editor,
            Aabb2::ZERO.extend_positive(framebuffer.size().as_f32()),
            &mut self.ui_context,
        );
        self.ui_focused = !can_focus;
        self.ui_context.frame_end();
        for action in actions {
            self.execute(action);
        }

        if let Some(level_editor) = &mut self.editor.level_edit {
            level_editor.model.camera.fov = 10.0 / self.editor.view_zoom.current;
        }
        self.render
            .draw_editor(&self.editor, &self.ui, &self.ui_context, framebuffer);
    }
}

impl Editor {
    fn delete_level(&mut self, level_index: usize) {
        if let Some(level_editor) = &self.level_edit {
            if level_index == level_editor.static_level.level_index {
                self.level_edit = None;
            }
        }

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
            let model = Model::empty(
                self.context.clone(),
                self.context.get_options(),
                level.clone(),
            );
            self.level_edit = Some(LevelEditor::new(
                self.context.clone(),
                model,
                level,
                self.visualize_beat,
                self.show_only_selected,
            ));
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
    fn confirm_action(&mut self, _ui: &mut EditorUi) {
        let Some(popup) = self.confirm_popup.take() else {
            return;
        };
        match popup.action {
            ConfirmAction::ExitUnsaved => self.exit(),
            ConfirmAction::ChangeLevelUnsaved(index) => self.change_level(index),
            ConfirmAction::DeleteLevel(index) => self.delete_level(index),
        }
    }

    fn scroll_time_by(&mut self, scroll_speed: ScrollSpeed, scroll: i64) {
        let Some(level_editor) = &mut self.level_edit else {
            return;
        };

        let scroll_speed = match scroll_speed {
            ScrollSpeed::Slow => self.config.scroll_slow,
            ScrollSpeed::Normal => self.config.scroll_normal,
            ScrollSpeed::Fast => self.config.scroll_fast,
        };
        let scroll = scroll_speed * scroll;
        let beat_time = level_editor
            .level
            .timing
            .get_timing(level_editor.current_time.target)
            .beat_time;
        let scroll = scroll.as_time(beat_time); // TODO: well beat time may change as we scroll

        level_editor.scroll_time(scroll);
    }
}

impl LevelEditor {
    fn undo(&mut self) {
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

    fn redo(&mut self) {
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
    fn save_state(&mut self, label: HistoryLabel) {
        self.history.save_state(&self.level, label);
        log::trace!("save_state called by {}", std::panic::Location::caller());
    }

    /// Flush all buffered changes to the undo stack, if there are any.
    #[track_caller]
    fn flush_changes(&mut self) {
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

    fn new_waypoint(&mut self) {
        self.execute(LevelAction::DeselectWaypoint);

        if let State::Waypoints { state, .. } = &mut self.state {
            *state = WaypointsState::New;
        }
    }

    fn view_waypoints(&mut self) {
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

    fn scroll_time(&mut self, delta: Time) {
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
