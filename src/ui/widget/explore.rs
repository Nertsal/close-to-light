use super::*;

pub struct ExploreWidget {
    pub state: WidgetState,
    pub window: UiWindow<()>,
}

impl ExploreWidget {
    pub fn new() -> Self {
        Self {
            state: WidgetState::new(),
            window: UiWindow::new((), 0.3),
        }
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
