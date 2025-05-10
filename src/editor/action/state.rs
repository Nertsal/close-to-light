use super::*;

#[derive(Debug, Clone)]
pub enum EditorStateAction {
    Exit,
    Editor(EditorAction),
    Cancel,
    StopTextEdit,
    UpdateTextEdit(String),
    CursorMove(vec2<f32>),
    WheelScroll(f32),
    StartPlaytest,
    EndDrag,
    StartDrag(DragTarget),
    ConfirmPopupAction,
    ContextMenu(vec2<f32>, Vec<(Name, EditorStateAction)>),
    CloseContextMenu,
    SelectMusicFile(std::path::PathBuf),
    SetGroupName(String),
}

impl From<EditorAction> for EditorStateAction {
    fn from(value: EditorAction) -> Self {
        Self::Editor(value)
    }
}

impl From<LevelAction> for EditorStateAction {
    fn from(value: LevelAction) -> Self {
        Self::Editor(EditorAction::Level(value))
    }
}

impl EditorState {
    pub fn execute(&mut self, action: EditorStateAction) {
        // log::debug!("action EditorStateAction::{:?}", action);
        match action {
            EditorStateAction::Exit => {
                self.transition = Some(geng::state::Transition::Pop);
            }
            EditorStateAction::Editor(action) => self.editor.execute(action),
            EditorStateAction::Cancel => self.cancel(),
            EditorStateAction::StopTextEdit => {
                self.ui_context.text_edit.stop();
            }
            EditorStateAction::UpdateTextEdit(text) => {
                self.ui_context.text_edit.set_text(text);
            }
            EditorStateAction::CursorMove(position) => {
                self.ui_context.cursor.cursor_move(position);
                if let Some(drag) = &mut self.editor.drag {
                    drag.moved = true;
                }
            }
            EditorStateAction::WheelScroll(delta) => {
                self.ui_context.cursor.scroll += delta;
            }
            EditorStateAction::StartPlaytest => self.play_game(),
            EditorStateAction::EndDrag => self.end_drag(),
            EditorStateAction::StartDrag(target) => self.start_drag(target),
            EditorStateAction::ConfirmPopupAction => self.editor.confirm_action(&mut self.ui),
            EditorStateAction::ContextMenu(position, options) => {
                self.ui.context_menu = ContextMenuWidget::new(position, options);
            }
            EditorStateAction::CloseContextMenu => {
                self.ui.context_menu.close();
            }
            EditorStateAction::SelectMusicFile(path) => {
                let group = self.editor.group.group_index;
                match self.context.local.select_music_file(group, path) {
                    Err(err) => {
                        log::error!("Failed to select music: {:?}", err);
                    }
                    Ok(group) => {
                        self.editor.group.music = group.local.music.clone();
                        self.editor.group.cached = group;
                    }
                }
            }
            EditorStateAction::SetGroupName(name) => {
                let group = self.editor.group.group_index;
                let mut meta = self.editor.group.cached.local.meta.clone();

                let mut music_meta = meta.music.unwrap_or_default();
                log::debug!("Renaming music into {:?}", name);
                music_meta.name = name.into();
                music_meta.romanized = music_meta.name.clone(); // TODO: separate config

                meta.music = Some(music_meta);
                match self.context.local.update_group_meta(group, meta) {
                    None => {
                        log::error!("Failed to rename level");
                    }
                    Some(group) => {
                        self.editor.group.music = group.local.music.clone();
                        self.editor.group.cached = group;
                    }
                }
            }
        }
    }

    fn end_drag(&mut self) {
        let Some(level_editor) = &mut self.editor.level_edit else {
            return;
        };

        if let Some(drag) = self.editor.drag.take() {
            match drag.target {
                DragTarget::SelectionArea {
                    mut original,
                    extra,
                } => {
                    if drag.from_screen == self.ui_context.cursor.position {
                        // Click - select hovered light
                        if let Some(hovered_light) = level_editor.level_state.hovered_light {
                            let mode = if original.is_light_selected(hovered_light) {
                                SelectMode::Remove
                            } else {
                                SelectMode::Add
                            };
                            level_editor
                                .execute(LevelAction::SelectLight(mode, vec![hovered_light]), None);
                        }
                    } else {
                        original.merge(extra);
                        level_editor.selection = original;
                    }
                }
                DragTarget::Light { double, .. } => {
                    if double
                        && drag.from_world == self.editor.cursor_world_pos_snapped
                        && level_editor.real_time - drag.from_real_time < r32(0.5)
                    {
                        // See waypoints
                        level_editor.view_waypoints();
                    }
                }
                _ => (),
            }

            level_editor.flush_changes(None);
        }
    }

    fn start_drag(&mut self, target: DragTarget) {
        self.end_drag();
        log::debug!("Dragging: {:?}", target);

        let Some(level_editor) = &mut self.editor.level_edit else {
            return;
        };

        self.editor.drag = Some(Drag {
            moved: false,
            from_screen: self.ui_context.cursor.position,
            from_world_raw: self.editor.cursor_world_pos,
            from_world: self.editor.cursor_world_pos_snapped,
            from_real_time: level_editor.real_time,
            from_beat: level_editor.current_time.target,
            target,
        });
    }

    fn cancel(&mut self) {
        if self.editor.confirm_popup.is_some() {
            self.execute(EditorAction::ClosePopup.into());
        } else if self.ui_context.is_totally_focused() {
            self.ui_context.cancel_total_focus();
        } else if self.ui.context_menu.is_open() {
            self.ui.context_menu.close();
        } else {
            self.execute(LevelAction::Cancel.into());
        }
    }
}
