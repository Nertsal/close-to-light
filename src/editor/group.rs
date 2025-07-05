use super::*;

pub struct Editor {
    pub context: Context,
    pub config: EditorConfig,
    pub render_options: RenderOptions,
    pub cursor_world_pos: vec2<Coord>,
    pub cursor_world_pos_snapped: vec2<Coord>,
    pub drag: Option<Drag>,

    pub confirm_popup: Option<ConfirmPopup<ConfirmAction>>,

    pub tab: EditorTab,
    /// Whether to exit the editor on the next frame.
    pub exit: bool,

    pub grid: Grid,
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

#[derive(Debug)]
pub struct Drag {
    /// Whether we just clicked or actually starting moving.
    pub moved: bool,
    pub from_screen: vec2<f32>,
    /// Unsnapped cursor position.
    pub from_world_raw: vec2<Coord>,
    pub from_world: vec2<Coord>,
    pub from_real_time: FloatTime,
    pub from_beat: Time,
    pub target: DragTarget,
}

#[derive(Debug, Clone)]
pub enum DragTarget {
    SelectionArea {
        original: Selection,
        extra: Selection,
    },
    Camera {
        initial_center: vec2<Coord>,
    },
    /// Move the whole light event through time and space.
    Light {
        /// Whether it was the second click on the light.
        /// If the drag is short, waypoints will be toggled.
        double: bool,
        lights: Vec<DragLight>,
    },
    WaypointMove {
        light: LightId,
        waypoint: WaypointId,
        initial_translation: vec2<Coord>,
        initial_time: Time,
    },
    WaypointScale {
        light: LightId,
        waypoint: WaypointId,
        initial_scale: Coord,
        scale_direction: vec2<Coord>,
    },
}

#[derive(Debug, Clone)]
pub struct DragLight {
    pub id: LightId,
    pub initial_time: Time,
    pub initial_translation: vec2<Coord>,
}

impl Editor {
    pub fn delete_level(&mut self, level_index: usize) {
        if let Some(level_editor) = &self.level_edit {
            if level_index == level_editor.static_level.level_index {
                self.level_edit = None;
            }
        }

        if !(0..self.group.cached.local.data.levels.len()).contains(&level_index) {
            log::error!(
                "Tried to remove a level by an invalid index {level_index}"
            );
            return;
        }

        let mut new_group = self.group.cached.local.data.clone();
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

    pub fn create_new_level(&mut self) {
        let mut new_group = self.group.cached.local.data.clone();
        let bpm = r32(120.0);
        new_group.levels.push(Rc::new(LevelFull {
            meta: LevelInfo {
                id: 0,
                name: "New Diff".into(),
                authors: Vec::new(),
                hash: String::new(),
            },
            data: Level::new(bpm),
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

    pub fn move_level_low(&mut self, level_index: usize) {
        let Some(swap_with) = level_index.checked_sub(1) else {
            return;
        };
        self.swap_levels(level_index, swap_with);
    }

    pub fn move_level_high(&mut self, level_index: usize) {
        self.swap_levels(level_index, level_index + 1);
    }

    pub fn swap_levels(&mut self, i: usize, j: usize) {
        let levels = &self.group.cached.local.data.levels;
        if !(0..levels.len()).contains(&i) || !(0..levels.len()).contains(&j) {
            log::error!("Invalid indices to swap levels");
            return;
        }

        let mut new_group = self.group.cached.local.data.clone();
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

    pub fn change_level(&mut self, level_index: usize) {
        if let Some(_level_editor) = self.level_edit.take() {
            // TODO: check unsaved changes
        }

        if let Some(level) = self.group.cached.local.data.levels.get(level_index) {
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

    pub fn save(&mut self) {
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
    pub fn is_changed(&self) -> bool {
        if let Some(level_editor) = &self.level_edit {
            let Some(cached) = self
                .group
                .cached
                .local
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
    pub fn popup_confirm(&mut self, action: ConfirmAction, message: impl Into<Name>) {
        let title = match action {
            ConfirmAction::ExitUnsaved => "Exit the editor?",
            ConfirmAction::ChangeLevelUnsaved(_) => "Switch to another difficulty?",
            ConfirmAction::DeleteLevel(_) => "Delete this difficulty?",
        };
        self.confirm_popup = Some(ConfirmPopup {
            action,
            title: title.into(),
            message: message.into(),
        });
    }

    /// Confirm the popup action and execute it.
    pub fn confirm_action(&mut self, _ui: &mut EditorUi) {
        let Some(popup) = self.confirm_popup.take() else {
            return;
        };
        match popup.action {
            ConfirmAction::ExitUnsaved => self.exit(),
            ConfirmAction::ChangeLevelUnsaved(index) => self.change_level(index),
            ConfirmAction::DeleteLevel(index) => self.delete_level(index),
        }
    }

    pub fn scroll_time_by(&mut self, scroll_speed: ScrollSpeed, scroll: i64) {
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
