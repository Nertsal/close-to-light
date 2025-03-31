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
    // TimelineScroll(Coord),
    // TimelineZoom(Coord),
    EndDrag,
    StartDrag(DragTarget),
    ConfirmPopupAction,
    ContextMenu(vec2<f32>, Vec<(Name, EditorStateAction)>),
    CloseContextMenu,
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
            // EditorStateAction::TimelineScroll(scroll) => {
            //     if let Some(level_editor) = &self.editor.level_edit {
            //         let timeline = &mut self.ui.edit.timeline;
            //         let delta = -scroll.as_f32() * 30.0 / timeline.get_scale();
            //         let delta = (delta * TIME_IN_FLOAT_TIME as f32).round() as Time;
            //         let current = -timeline.get_scroll();
            //         let delta = if delta > 0 {
            //             delta.min(current)
            //         } else {
            //             -delta.abs().min(
            //                 level_editor.level.last_time() - timeline.visible_scroll() - current,
            //             )
            //         };
            //         timeline.scroll(delta);
            //     }
            // }
            // EditorStateAction::TimelineZoom(scroll) => {
            //     let timeline = &mut self.ui.edit.timeline;
            //     let zoom = timeline.get_scale();
            //     let zoom = (zoom + scroll.as_f32() * 0.05).clamp(0.05, 0.75);
            //     timeline.rescale(zoom);
            // }
            EditorStateAction::EndDrag => self.end_drag(),
            EditorStateAction::StartDrag(target) => self.start_drag(target),
            EditorStateAction::ConfirmPopupAction => self.editor.confirm_action(&mut self.ui),
            EditorStateAction::ContextMenu(position, options) => {
                self.ui.context_menu = ContextMenuWidget::new(position, options);
            }
            EditorStateAction::CloseContextMenu => {
                self.ui.context_menu.close();
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
                    original.merge(extra);
                    level_editor.selection = original;
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
        if self.ui_context.is_totally_focused() {
            self.ui_context.cancel_total_focus();
        } else if self.ui.context_menu.is_open() {
            self.ui.context_menu.close();
        } else {
            self.execute(LevelAction::Cancel.into());
        }
    }
}
