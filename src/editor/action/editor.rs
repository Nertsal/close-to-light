use super::*;

#[derive(Debug, Clone)]
pub enum EditorAction {
    Level(LevelAction),
    ToggleDynamicVisual,
    Save,
    ToggleUI,
    ToggleGrid,
    ToggleGridSnap,
}

impl From<LevelAction> for EditorAction {
    fn from(value: LevelAction) -> Self {
        Self::Level(value)
    }
}

impl Editor {
    pub fn execute(&mut self, action: EditorAction) {
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
            EditorAction::ToggleDynamicVisual => {
                self.visualize_beat = !self.visualize_beat;
            }
            EditorAction::Save => self.save(),
            EditorAction::ToggleUI => {
                self.render_options.hide_ui = !self.render_options.hide_ui;
            }
            EditorAction::ToggleGrid => {
                self.render_options.show_grid = !self.render_options.show_grid;
            }
            EditorAction::ToggleGridSnap => {
                self.snap_to_grid = !self.snap_to_grid;
            }
        }
    }
}
