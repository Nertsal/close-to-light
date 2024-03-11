use super::*;

use crate::prelude::LevelMeta;

use std::path::PathBuf;

pub struct ExploreWidget {
    pub state: WidgetState,
    pub window: UiWindow<()>,

    pub editing_levels: Vec<ExploreLevelWidget>,
}

pub struct ExploreLevelWidget {
    pub state: WidgetState,
    pub path: PathBuf,
    // pub meta: LevelMeta,
}

impl ExploreWidget {
    pub fn new() -> Self {
        Self {
            state: WidgetState::new(),
            window: UiWindow::new((), 0.3),

            editing_levels: Vec::new(),
        }
    }

    pub fn load_editing(&mut self, levels: Vec<ExploreLevelWidget>) {
        self.editing_levels = levels;
    }
}

impl Widget for ExploreWidget {
    fn update(&mut self, position: Aabb2<f32>, context: &mut UiContext) {
        self.state.update(position, context);
        self.window.update(context.delta_time);
    }

    fn walk_states_mut(&mut self, f: &dyn Fn(&mut WidgetState)) {
        self.state.walk_states_mut(f);
    }
}
