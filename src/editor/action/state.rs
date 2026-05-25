use super::*;

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
            EditorStateAction::ConfirmPopupAction => self.editor.confirm_action(),
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
                        log::error!("Failed to select music: {err:?}");
                    }
                    Ok(group) => {
                        self.editor.group.music = group.local.music.clone();
                        self.editor.group.cached = group;
                    }
                }
            }
            EditorStateAction::SetGroupName(name) => {
                self.update_group_meta(|meta| {
                    log::debug!("Renaming music into {name:?}");
                    meta.music.name = name.into();
                    meta.music.romanized = meta.music.name.clone(); // TODO: separate config
                });
            }
            EditorStateAction::AddMusicAuthor(author) => {
                self.update_group_meta(|meta| {
                    log::debug!("Adding music author {author:?}");
                    meta.music.authors.push(author);
                });
            }
            EditorStateAction::UpdateMusicAuthor(i, author) => {
                self.update_group_meta(|meta| {
                    log::debug!("Changing music author {i} to {author:?}");
                    if let Some(old) = meta.music.authors.get_mut(i) {
                        *old = author;
                    }
                });
            }
            EditorStateAction::RemoveMusicAuthor(i) => {
                self.update_group_meta(|meta| {
                    if let Some(author) =
                        (i < meta.music.authors.len()).then(|| meta.music.authors.remove(i))
                    {
                        log::debug!("Removed music author {i}: {author:?}");
                    }
                });
            }
            EditorStateAction::SetLevelName(level, name) => {
                self.update_diff_meta(level, |meta| {
                    meta.name = name;
                });
            }
            EditorStateAction::AddLevelAuthor(level, author) => {
                self.update_diff_meta(level, |meta| {
                    log::debug!("Adding level {level} author {author:?}");
                    meta.authors.push(author);
                });
            }
            EditorStateAction::UpdateLevelAuthor(level, author_idx, author) => {
                self.update_diff_meta(level, |meta| {
                    log::debug!("Changing level {level} author {author_idx} to {author:?}");
                    if let Some(old) = meta.authors.get_mut(author_idx) {
                        *old = author;
                    }
                });
            }
            EditorStateAction::RemoveLevelAuthor(level, i) => {
                self.update_diff_meta(level, |meta| {
                    if let Some(author) = (i < meta.authors.len()).then(|| meta.authors.remove(i)) {
                        log::debug!("Removed music author {i}: {author:?}");
                    }
                });
            }
        }
    }

    fn update_group_meta(&mut self, f: impl FnOnce(&mut LevelSetInfo)) {
        let group = self.editor.group.group_index;
        let mut meta = self.editor.group.cached.local.meta.clone();

        f(&mut meta);

        match self.context.local.update_group_meta(group, meta, false) {
            None => {
                log::error!("Failed to update level meta");
            }
            Some(group) => {
                self.editor.group.music = group.local.music.clone();
                self.editor.group.cached = group;
            }
        }
    }

    fn update_diff_meta(&mut self, index: usize, f: impl FnOnce(&mut LevelInfo)) {
        let group = self.editor.group.group_index;
        let mut meta = self.editor.group.cached.local.meta.clone();
        if let Some(level_meta) = meta.levels.get_mut(index) {
            f(level_meta);

            match self.context.local.update_group_meta(group, meta, false) {
                None => {
                    log::error!("Failed to update level difficulty meta");
                }
                Some(group) => {
                    self.editor.group.music = group.local.music.clone();
                    self.editor.group.cached = group;
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
                        if let Some(waypoints) = &level_editor.level_state.waypoints {
                            // Hovered waypoint
                            if let Some(i) = waypoints.hovered
                                && let Some(point) = waypoints.points.get(i)
                                && let Some(i) = point.original
                            {
                                let mode = if original.is_waypoint_selected(waypoints.light, i) {
                                    SelectMode::Remove
                                } else {
                                    SelectMode::Add
                                };
                                level_editor.execute(
                                    LevelAction::SelectWaypoint(
                                        mode,
                                        waypoints.light,
                                        vec![i],
                                        false,
                                    ),
                                    None,
                                );
                            }
                        } else if let Some(hovered_light) = level_editor.level_state.hovered_light {
                            // Hovered light
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
        log::debug!("Dragging: {target:?}");

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
        } else if self.ui_context.is_totally_focused().is_some() {
            self.ui_context.cancel_total_focus();
        } else if self.ui.context_menu.is_open() {
            self.ui.context_menu.close();
        } else {
            self.execute(LevelAction::Cancel.into());
        }
    }
}
