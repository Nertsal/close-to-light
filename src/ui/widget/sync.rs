use super::*;

use crate::{
    local::{CachedGroup, CachedLevel, LevelCache},
    prelude::Assets,
    task::Task,
    ui::layout::AreaOps,
};

use ctl_client::{
    core::{
        model::Level,
        types::{Id, LevelInfo, NewLevel},
    },
    ClientError,
};
use generational_arena::Index;

type TaskRes<T> = Option<Task<ctl_client::Result<T>>>;

pub struct SyncWidget {
    geng: Geng,
    cached_group: Id,
    cached_group_index: Index,
    cached_music: Option<Id>,
    cached_level: Rc<CachedLevel>,
    cached_level_index: usize,
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
    pub discard: TextWidget,
    pub response: TextWidget,

    task_level_info: TaskRes<LevelInfo>,
    /// Returns group and level index and the new group and level id.
    task_level_upload: TaskRes<(Index, usize, Id, Id)>,
    task_level_download: TaskRes<Level>,
}

impl SyncWidget {
    pub fn new(
        geng: &Geng,
        assets: &Rc<Assets>,
        group: &CachedGroup,
        group_index: Index,
        level: &Rc<CachedLevel>,
        level_index: usize,
    ) -> Self {
        Self {
            geng: geng.clone(),
            cached_group: group.meta.id,
            cached_group_index: group_index,
            cached_music: group.music.as_ref().map(|music| music.meta.id),
            cached_level: level.clone(),
            cached_level_index: level_index,
            reload: true,

            state: WidgetState::new(),
            offset: vec2::ZERO,

            window: UiWindow::new((), 0.3),
            hold: WidgetState::new(),
            close: IconButtonWidget::new_close_button(&assets.sprites.button_close),
            title: TextWidget::new("Synchronizing level"),
            status: TextWidget::new("Offline"),
            upload: TextWidget::new("Upload to the server"),
            discard: TextWidget::new("Discard changes"),
            response: TextWidget::new(""),

            task_level_info: None,
            task_level_upload: None,
            task_level_download: None,
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
            if let Some(client) = state.client() {
                let level_id = self.cached_level.meta.id;
                if level_id == 0 {
                    self.status.text = "Level is local".into();
                    self.response.hide();
                    self.upload.show();
                    self.discard.show();
                } else {
                    let future = async move { client.get_level_info(level_id).await };
                    self.task_level_info = Some(Task::new(&self.geng, future));
                }
            }
        }

        if let Some(task) = self.task_level_info.take() {
            match task.poll() {
                Err(task) => self.task_level_info = Some(task),
                Ok(Err(err)) => {
                    if let ClientError::NotFound = err {
                        // Level is unknown to the server - probably created by the user
                        self.status.text = "Unknown to the server".into();
                        self.response.hide();
                        self.upload.show();
                        self.discard.show();
                    } else {
                        self.status.text = "Failed".into();
                        self.response.show();
                        self.response.text = format!("{}", err);
                        self.upload.hide();
                        self.discard.hide();
                    }
                }
                Ok(Ok(level)) => {
                    if level.hash != self.cached_level.hash {
                        // Local level version is probably outdated (or invalid)
                        self.status.text = "Outdated or changed".into();
                        self.response.hide();

                        // TODO: Check the author
                        // if current user is the author - upload new version ; discard changes
                        // if current user is not author - download new version

                        self.upload.show();
                        self.discard.show();
                    } else {
                        // Everything's fine
                        self.status.text = "Up to date".into();
                        self.response.hide();
                        self.upload.hide();
                        self.discard.hide();
                    }
                }
            }
        }
        if let Some(task) = self.task_level_upload.take() {
            match task.poll() {
                Err(task) => self.task_level_upload = Some(task),
                Ok(Err(err)) => {
                    // TODO
                    log::error!("Failed to upload the level: {:?}", err);
                    self.status.text = "".into();
                    self.response.show();
                    self.response.text = format!("{}", err);
                }
                Ok(Ok((group_index, level_index, group, level))) => {
                    if let Some(level) = state.synchronize(group_index, level_index, group, level) {
                        self.cached_group = group;
                        self.cached_level = level;
                        self.reload = true;
                    }
                }
            }
        }
        if let Some(task) = self.task_level_download.take() {
            match task.poll() {
                Err(task) => self.task_level_download = Some(task),
                Ok(Err(err)) => {
                    if let ClientError::NotFound = err {
                        log::error!("Requested level not found");
                        // TODO: delete local
                    } else {
                        log::error!("Failed to download the level: {:?}", err);
                        self.response.show();
                        self.response.text = format!("{}", err);
                    }
                }
                Ok(Ok(level)) => {
                    if let Some(level) = state.update_level(self.cached_level.meta.id, level) {
                        self.cached_level = level;
                        self.reload = true;
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

        main.cut_top(context.layout_size * 1.0);

        let button_size = vec2(main.width() * 0.75, context.font_size * 1.3);

        let upload = main
            .cut_top(context.font_size * 1.5)
            .align_aabb(button_size, vec2::splat(0.5));
        self.upload.update(upload, context);
        if self.upload.state.clicked {
            if let Some(music) = self.cached_music {
                // TODO: or server responded 404 meaning local state is desynced
                // Create new level or upload new version
                if let Some(client) = state.client() {
                    let mut group = self.cached_group;
                    let group_index = self.cached_group_index;
                    let level_index = self.cached_level_index;
                    let level_id = self.cached_level.meta.id;
                    let level = Rc::clone(&self.cached_level);
                    let future = async move {
                        if group == 0 {
                            // Create group
                            group = client.create_group(music).await?;
                        }
                        // TODO: it could happen that a level has a local non-zero id
                        // but is not present on the server.
                        // In that case, upload will fail with "Not found"
                        let level_id = client
                            .upload_level(
                                NewLevel {
                                    level_id: (level_id != 0).then_some(level_id),
                                    name: level.meta.name.clone(),
                                    group,
                                },
                                &level.data,
                            )
                            .await?;
                        Ok((group_index, level_index, group, level_id))
                    };
                    self.task_level_upload = Some(Task::new(&self.geng, future));
                }
            } else {
                // TODO: notify the user that no music is selected for the level
            }
        }

        let discard = main
            .cut_top(context.font_size * 1.5)
            .align_aabb(button_size, vec2::splat(0.5));
        self.discard.update(discard, context);
        if self.discard.state.clicked {
            if self.cached_level.meta.id == 0 {
                // Delete
                state.delete_level(self.cached_group_index, self.cached_level_index);
                self.window.request = Some(WidgetRequest::Close);
            } else if let Some(client) = state.client() {
                let level_id = self.cached_level.meta.id;
                let future = async move {
                    let bytes = client.download_level(level_id).await?;
                    let level: Level = bincode::deserialize(&bytes)?;
                    Ok(level)
                };
                self.task_level_download = Some(Task::new(&self.geng, future));
            }
        }

        main.cut_top(context.layout_size * 1.0);

        let response = main.cut_top(context.font_size);
        self.response.update(response, context);
        self.response.options.color = context.theme.danger;
    }
}
