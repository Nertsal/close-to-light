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
    SetViewZoom(f32),
    SetGridSize(Coord),
    ScrollTimeBy(ScrollSpeed, i64),
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
                    editor.execute(action);
                } else {
                    log::error!(
                        "Tried performing level editor action, but no level is loaded: {:?}",
                        action
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
            EditorAction::SetViewZoom(zoom) => self.view_zoom = zoom,
            EditorAction::SetGridSize(size) => self.grid_size = size,
            EditorAction::ScrollTimeBy(speed, scroll) => {
                self.scroll_time_by(speed, scroll);
            }
        }
    }
}
