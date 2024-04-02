use super::*;

use crate::{
    local::{CachedGroup, CachedLevel, LevelCache},
    prelude::Assets,
    task::Task,
    ui::layout::AreaOps,
};

use ctl_client::core::types::{Id, LevelInfo, NewLevel};

pub struct SyncWidget {
    geng: Geng,
    cached_group: Id,
    cached_level: Rc<CachedLevel>,
    reload: bool,

    pub state: WidgetState,
    pub offset: vec2<f32>,

    pub window: UiWindow<()>,
    /// Position that can be dragged to move the widget.
    pub hold: WidgetState,
    pub close: IconButtonWidget,
    pub title: TextWidget,
    pub status: TextWidget,
    pub upload: TextWidget,

    task_level_info: Option<Task<anyhow::Result<LevelInfo>>>,
    task_level_upload: Option<Task<anyhow::Result<()>>>,
}

impl SyncWidget {
    pub fn new(geng: &Geng, assets: &Rc<Assets>, group: Id, level: &Rc<CachedLevel>) -> Self {
        Self {
            geng: geng.clone(),
            cached_group: group,
            cached_level: level.clone(),
            reload: true,

            state: WidgetState::new(),
            offset: vec2::ZERO,

            window: UiWindow::new((), 0.3),
            hold: WidgetState::new(),
            close: IconButtonWidget::new_close_button(&assets.sprites.button_close),
            title: TextWidget::new("Synchronizing level"),
            status: TextWidget::new("Offline"),
            upload: TextWidget::new("Upload to the server"),

            task_level_info: None,
            task_level_upload: None,
        }
    }
}

impl StatefulWidget for SyncWidget {
    type State = LevelCache;

    fn state_mut(&mut self) -> &mut WidgetState {
        &mut self.state
    }

    fn update(&mut self, position: Aabb2<f32>, context: &mut UiContext, state: &mut Self::State) {
        if std::mem::take(&mut self.reload) && self.task_level_info.is_none() {
            if let Some(client) = state.client().cloned() {
                let level_id = self.cached_level.meta.id;
                let future = async move {
                    if level_id == 0 {
                        return Err(anyhow!("Level is local"));
                    }
                    let level = client.get_level_info(level_id).await?;
                    Ok(level)
                };
                self.task_level_info = Some(Task::new(&self.geng, future));
            }
        }

        if let Some(task) = self.task_level_info.take() {
            match task.poll() {
                Err(task) => self.task_level_info = Some(task),
                Ok(level) => {
                    if let Ok(level) = level {
                        if level.hash != self.cached_level.hash {
                            // Local level version is probably outdated (or invalid)
                            self.status.text = "Local level version is outdated or changed".into();

                            // TODO: Check the author
                            // if current user is the author - upload new version ; discard changes
                            // if current user is not author - download new version
                        } else {
                            // Everything's fine
                            self.status.text = "Level is up to date".into();
                        }
                    } else {
                        // TODO: match error type, e.g. server does not respond, level id is 0
                        // Level is unknown to the server - probably created by the user
                        self.status.text = "Level unknown to the server".into();
                        self.upload.show();
                    }
                }
            }
        }

        let position = position.translate(self.offset);

        self.window.layout(true, self.close.state.clicked);
        self.window.update(context.delta_time);
        self.state.update(position, context);

        let mut hold = position.extend_symmetric(-vec2(5.0, 0.0) * context.layout_size / 2.0);
        let hold = hold.cut_top(context.layout_size);
        self.hold.update(hold, context);

        if self.hold.pressed {
            // Drag window
            self.offset += context.cursor.delta();
        }

        let mut main = position.extend_uniform(-context.font_size * 0.2);

        let close = main.align_aabb(vec2::splat(2.0) * context.layout_size, vec2(1.0, 1.0));
        self.close.update(close, context);

        main.cut_top(context.layout_size);

        let title = main.cut_top(context.font_size);
        self.title.update(title, context);

        let status = main.cut_top(context.font_size);
        self.status.update(status, context);

        main.cut_top(context.layout_size * 5.0);

        let upload = main.cut_top(context.font_size * 1.5);
        self.upload.update(upload, context);
        if self.upload.state.clicked {
            if self.cached_group == 0 {
                // Create a group
                // TODO
            }
            if self.cached_level.meta.id == 0 {
                // Create new level
                if let Some(client) = state.client().cloned() {
                    let group = self.cached_group;
                    let level = Rc::clone(&self.cached_level);
                    let future = async move {
                        client
                            .upload_level(
                                NewLevel {
                                    name: level.meta.name.clone(),
                                    group,
                                },
                                &level.data,
                            )
                            .await?;
                        Ok(())
                    };
                    self.task_level_upload = Some(Task::new(&self.geng, future));
                }
            } else {
                // TODO: upload new version
            }
        }
    }
}
