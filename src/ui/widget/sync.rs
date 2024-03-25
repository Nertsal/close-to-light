use super::*;

use crate::{
    local::{CachedLevel, LevelCache},
    prelude::Assets,
    task::Task,
};

use ctl_client::core::types::LevelInfo;

pub struct SyncWidget {
    geng: Geng,
    cached_level: Rc<CachedLevel>,
    reload: bool,

    pub state: WidgetState,

    task_level_info: Option<Task<anyhow::Result<LevelInfo>>>,
}

impl SyncWidget {
    pub fn new(geng: &Geng, assets: &Rc<Assets>, level: &Rc<CachedLevel>) -> Self {
        Self {
            geng: geng.clone(),
            cached_level: level.clone(),
            reload: true,

            state: WidgetState::new(),

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
                            log::debug!("Local level version is probably outdated (or invalid)");
                        } else {
                            // Everything's fine
                            log::debug!("Level is up to date");
                        }
                    } else {
                        // Level is unknown to the server - probably created by the user
                        log::debug!("Level unknown");
                    }
                }
            }
        }

        self.state.update(position, context);

        let mut main = position;
    }
}
