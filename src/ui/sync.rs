use super::*;

use crate::{
    menu::{ConfirmAction, MenuState},
    prelude::Assets,
    ui::layout::AreaOps,
};

use ctl_client::{
    ClientError, Nertboard,
    core::types::{LevelSet, LevelSetInfo},
};
use ctl_local::CachedGroup;
use ctl_util::Task;
use generational_arena::Index;

type TaskRes<T> = Option<Task<ctl_client::Result<T>>>;

pub struct SyncWidget {
    geng: Geng,
    cached_group: Rc<CachedGroup>,
    cached_group_index: Index,
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

    task_group_info: TaskRes<LevelSetInfo>,
    /// Returns group and level index and the new group and level id.
    task_group_upload: TaskRes<(Index, LevelSetInfo)>,
    task_group_download: TaskRes<(LevelSet, LevelSetInfo)>,
}

impl SyncWidget {
    pub fn new(
        geng: &Geng,
        assets: &Rc<Assets>,
        group: Rc<CachedGroup>,
        group_index: Index,
    ) -> Self {
        Self {
            geng: geng.clone(),
            cached_group_index: group_index,
            cached_group: group,
            reload: true,

            state: WidgetState::new(),
            offset: vec2::ZERO,

            window: UiWindow::new((), 0.3),
            hold: WidgetState::new(),
            close: IconButtonWidget::new_close_button(assets.atlas.button_close()),
            title: TextWidget::new("Synchronizing level"),
            status: TextWidget::new("Offline"),
            upload: TextWidget::new("Upload to the server"),
            discard: TextWidget::new("Download new version"),
            response: TextWidget::new(""),

            task_group_info: None,
            task_group_upload: None,
            task_group_download: None,
        }
    }

    pub fn discard_changes(&mut self, client: Arc<Nertboard>) {
        let group_id = self.cached_group.local.meta.id;
        let future = async move {
            let info = client.get_group_info(group_id).await?;
            let bytes = client.download_group(group_id).await?;
            let group: LevelSet = bincode::deserialize(&bytes)?;
            Ok((group, info))
        };
        self.task_group_download = Some(Task::new(&self.geng, future));
    }

    pub fn upload(&mut self, client: Arc<Nertboard>) {
        let group = (*self.cached_group).clone();
        let group_index = self.cached_group_index;
        let future = async move {
            // TODO: it could happen that a level has a local non-zero id
            // but is not present on the server.
            // In that case, upload will fail with "Not found"
            let music_id = if let Some(music) = &group.local.music {
                if music.meta.id == 0 {
                    client
                        .upload_music_bytes(
                            &music.bytes,
                            &ctl_logic::NewMusic {
                                name: music.meta.name.to_string(),
                                romanized_name: music.meta.name.to_string(), // TODO: romanized
                            },
                        )
                        .await?
                } else {
                    // TODO: check that the file is the same
                    music.meta.id
                }
            } else {
                0
            };
            // TODO: update local music
            let mut meta = group.local.meta.clone();
            meta.music.id = music_id;
            let level_set_full = LevelSetFull {
                meta,
                data: LevelSet {
                    levels: group
                        .local
                        .data
                        .levels
                        .iter()
                        .map(|level| (**level).clone())
                        .collect(),
                },
            };
            let group = client.upload_group(&level_set_full, music_id).await?;
            Ok((group_index, group))
        };
        self.task_group_upload = Some(Task::new(&self.geng, future));
    }
}

impl StatefulWidget for SyncWidget {
    type State<'a> = MenuState;

    fn state_mut(&mut self) -> &mut WidgetState {
        &mut self.state
    }

    fn update(
        &mut self,
        position: Aabb2<f32>,
        context: &mut UiContext,
        state: &mut Self::State<'_>,
    ) {
        let local = &state.context.local;

        if std::mem::take(&mut self.reload) && self.task_group_info.is_none() {
            if let Some(client) = local.client() {
                let group_id = self.cached_group.local.meta.id;
                if group_id == 0 {
                    self.status.text = "Level is local".into();
                    self.response.hide();
                    self.upload.show();
                    self.discard.hide();
                } else {
                    let future = async move { client.get_group_info(group_id).await };
                    self.task_group_info = Some(Task::new(&self.geng, future));
                }
            }
        }

        if let Some(task) = self.task_group_info.take() {
            match task.poll() {
                Err(task) => self.task_group_info = Some(task),
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
                        self.response.text = format!("{err}").into();
                        self.upload.hide();
                        self.discard.hide();
                    }
                }
                Ok(Ok(group)) => {
                    if group != self.cached_group.local.meta {
                        // Local level version is probably outdated (or invalid)
                        self.status.text = "Outdated".into();
                        self.response.hide();

                        if group.owner.id == self.cached_group.local.meta.owner.id {
                            // if current user is the author - upload new version ; discard changes
                            self.upload.show();
                        } else {
                            // if current user is not author - download new version
                            self.upload.hide();
                        }

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
        if let Some(task) = self.task_group_upload.take() {
            match task.poll() {
                Err(task) => self.task_group_upload = Some(task),
                Ok(Err(err)) => {
                    self.status.text = "".into();
                    self.response.show();
                    self.response.text = format!("{err}").into();
                }
                Ok(Ok((group_index, group))) => {
                    if let Some(group) = local.synchronize_meta(group_index, group) {
                        let name = group
                            .local
                            .music
                            .as_ref()
                            .map_or(&group.local.meta.owner.name, |music| &music.meta.name);
                        state.notifications.push(format!("Uploaded level {name}"));
                        self.cached_group = group;
                        self.reload = true;
                    }
                }
            }
        }
        if let Some(task) = self.task_group_download.take() {
            match task.poll() {
                Err(task) => self.task_group_download = Some(task),
                Ok(Err(err)) => {
                    if let ClientError::NotFound = err {
                        log::error!("Requested group not found");
                        // TODO: delete local
                    } else {
                        log::error!("Failed to download the group: {err:?}");
                        self.response.show();
                        self.response.text = format!("{err}").into();
                    }
                }
                Ok(Ok((group, info))) => {
                    if let Some(group) =
                        local.update_group(self.cached_group_index, group, Some(info))
                    {
                        let name = group
                            .local
                            .music
                            .as_ref()
                            .map_or(&group.local.meta.owner.name, |music| &music.meta.name);
                        state.notifications.push(format!("Downloaded level {name}"));
                        self.cached_group = group;
                        self.reload = true;
                    }
                }
            }
        }

        let position = position.translate(self.offset);

        self.window
            .layout(true, self.close.state.mouse_left.clicked);
        self.window.update(context.delta_time);
        self.state.update(position, context);

        let mut hold = position.extend_symmetric(-vec2(5.0, 0.0) * context.layout_size / 2.0);
        let hold = hold.cut_top(context.layout_size);
        self.hold.update(hold, context);

        if self.hold.mouse_left.pressed.is_some() {
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
        if self.upload.state.mouse_left.clicked {
            if self.cached_group.local.music.is_some() {
                // TODO: or server responded 404 meaning local state is desynced
                // Create new level or upload new version
                if self.cached_group.local.meta.id == 0 {
                    state.popup_confirm(
                        ConfirmAction::SyncUpload,
                        "You cannot undo this action",
                        "upload",
                        "cancel",
                    );
                } else {
                    state.popup_confirm(
                        ConfirmAction::SyncUpload,
                        "Uploading a new version will reset leaderboards of all difficulties",
                        "upload",
                        "cancel",
                    );
                }
            } else {
                state
                    .notifications
                    .push("Upload failed: the level has no music attached".into());
            }
        }

        let discard = main
            .cut_top(context.font_size * 1.5)
            .align_aabb(button_size, vec2::splat(0.5));
        self.discard.update(discard, context);
        if self.discard.state.mouse_left.clicked {
            if self.cached_group.local.meta.id == 0 {
                // Delete
                state.popup_confirm(
                    ConfirmAction::DeleteGroup(self.cached_group_index),
                    format!(
                        "delete the level by {}",
                        self.cached_group.local.meta.owner.name
                    ),
                    "delete group",
                    "cancel",
                );
                self.window.request = Some(WidgetRequest::Close);
            } else if let Some(_client) = state.context.local.client() {
                state.popup_confirm(
                    ConfirmAction::SyncDiscard,
                    "discard local changes and download the new version",
                    "discard",
                    "cancel",
                );
            }
        }

        main.cut_top(context.layout_size * 1.0);

        let response = main.cut_top(context.font_size);
        self.response.update(response, context);
        self.response.options.color = context.theme().danger;
    }
}
