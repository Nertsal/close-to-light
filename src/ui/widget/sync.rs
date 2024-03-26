use super::*;

use crate::{
    local::{CachedLevel, LevelCache},
    prelude::Assets,
    task::Task,
    ui::layout::AreaOps,
};

use ctl_client::core::types::LevelInfo;

pub struct SyncWidget {
    geng: Geng,
    cached_level: Rc<CachedLevel>,
    reload: bool,

    pub state: WidgetState,
    pub window: UiWindow<()>,
    /// Position that can be dragged to move the widget.
    pub hold: WidgetState,
    pub close: IconButtonWidget,
    pub title: TextWidget,
    pub status: TextWidget,

    task_level_info: Option<Task<anyhow::Result<LevelInfo>>>,
}

impl SyncWidget {
    pub fn new(geng: &Geng, assets: &Rc<Assets>, level: &Rc<CachedLevel>) -> Self {
        Self {
            geng: geng.clone(),
            cached_level: level.clone(),
            reload: true,

            state: WidgetState::new(),
            window: UiWindow::new((), 0.3),
            hold: WidgetState::new(),
            close: IconButtonWidget::new_close_button(&assets.sprites.button_close),
            title: TextWidget::new("Synchronizing level"),
            status: TextWidget::new("Loading..."),

            task_level_info: None,
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
                        } else {
                            // Everything's fine
                            self.status.text = "Level is up to date".into();
                        }
                    } else {
                        // Level is unknown to the server - probably created by the user
                        self.status.text = "Level unknown to the server".into();
                    }
                }
            }
        }

        self.window.layout(true, self.close.state.clicked);
        self.window.update(context.delta_time);
        self.state.update(position, context);

        let mut hold = position.extend_symmetric(-vec2(5.0, 0.0) * context.layout_size / 2.0);
        let hold = hold.cut_top(context.layout_size);
        self.hold.update(hold, context);

        let mut main = position.extend_uniform(-context.font_size * 0.2);

        let close = main.align_aabb(vec2::splat(2.0) * context.layout_size, vec2(1.0, 1.0));
        self.close.update(close, context);

        main.cut_top(context.layout_size);

        let title = main.cut_top(context.font_size);
        self.title.update(title, context);

        let status = main.cut_top(context.font_size);
        self.status.update(status, context);
    }
}
