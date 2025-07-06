use super::*;

#[derive(Debug, Clone)]
pub enum EditorAction {
    Level(LevelAction),
    SwitchTab(EditorTab),
    ToggleDynamicVisual,
    ToggleShowOnlySelected,
    Save,
    ToggleUI,
    ToggleGrid,
    ToggleGridSnap,
    DeleteLevel(usize),
    NewLevel,
    ChangeLevel(usize),
    MoveLevelLow(usize),
    MoveLevelHigh(usize),
    PopupConfirm(ConfirmAction, Name),
    ClosePopup,
    SetConfig(EditorConfig),
    SetViewZoom(Change<f32>),
    SetGridSize(Coord),
    ScrollTimeBy(ScrollSpeed, i64),
    StartPlaying,
    StopPlaying,
}

impl From<LevelAction> for EditorAction {
    fn from(value: LevelAction) -> Self {
        Self::Level(value)
    }
}

impl Editor {
    pub fn execute(&mut self, action: EditorAction) {
        // log::debug!("action EditorAction::{:?}", action);
        match action {
            EditorAction::Level(action) => {
                if let Some(editor) = &mut self.level_edit {
                    editor.execute(action, self.drag.as_mut());
                } else {
                    log::error!(
                        "Tried performing level editor action, but no level is loaded: {action:?}"
                    );
                }
            }
            EditorAction::SwitchTab(tab) => self.tab = tab,
            EditorAction::ToggleDynamicVisual => self.visualize_beat = !self.visualize_beat,
            EditorAction::ToggleShowOnlySelected => {
                self.show_only_selected = !self.show_only_selected
            }
            EditorAction::Save => self.save(),
            EditorAction::ToggleUI => self.render_options.hide_ui = !self.render_options.hide_ui,
            EditorAction::ToggleGrid => {
                self.render_options.show_grid = !self.render_options.show_grid
            }
            EditorAction::ToggleGridSnap => self.snap_to_grid = !self.snap_to_grid,
            EditorAction::DeleteLevel(i) => {
                self.popup_confirm(ConfirmAction::DeleteLevel(i), "delete this difficulty")
            }
            EditorAction::NewLevel => self.create_new_level(),
            EditorAction::ChangeLevel(i) => self.change_level(i),
            EditorAction::MoveLevelLow(i) => self.move_level_low(i),
            EditorAction::MoveLevelHigh(i) => self.move_level_high(i),
            EditorAction::PopupConfirm(action, message) => self.popup_confirm(action, message),
            EditorAction::ClosePopup => self.confirm_popup = None,
            EditorAction::SetConfig(config) => self.config = config,
            EditorAction::SetViewZoom(change) => {
                // TODO: undupe with ui slider settings
                let mut zoom = self.view_zoom.target;
                change.apply(&mut zoom);
                self.view_zoom.target = zoom.clamp(0.5, 2.0);
            }
            EditorAction::SetGridSize(size) => self.grid.cell_size = size,
            EditorAction::ScrollTimeBy(speed, scroll) => {
                self.scroll_time_by(speed, scroll);
            }
            EditorAction::StopPlaying => {
                if let Some(level_editor) = &mut self.level_edit {
                    if let EditingState::Playing {
                        start_time,
                        start_target_time,
                        playing_time: _,
                        old_state,
                    } = &level_editor.state
                    {
                        level_editor.current_time.snap_to(*start_time);
                        level_editor
                            .current_time
                            .scroll_time(Change::Set(*start_target_time));
                        level_editor.state = *old_state.clone();
                        level_editor.context.music.stop();
                    }
                }
            }
            EditorAction::StartPlaying => {
                if let Some(level_editor) = &mut self.level_edit {
                    level_editor.state = EditingState::Playing {
                        start_time: level_editor.current_time.value,
                        start_target_time: level_editor.current_time.target,
                        playing_time: FloatTime::ZERO,
                        old_state: Box::new(level_editor.state.clone()),
                    };
                    if let Some(music) = &level_editor.static_level.group.music {
                        let time = time_to_seconds(level_editor.current_time.target);
                        self.context
                            .music
                            .play_from(music, time::Duration::from_secs_f64(time.as_f32().into()));
                        self.music_timer = FloatTime::ZERO;
                    }
                }
            }
        }
    }
}
