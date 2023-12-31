use super::*;

use crate::prelude::Assets;

pub struct OptionsWidget {
    pub state: WidgetState,
}

impl OptionsWidget {
    pub fn new(assets: &Rc<Assets>) -> Self {
        Self {
            state: WidgetState::new(),
        }
    }
}

impl Widget for OptionsWidget {
    fn update(&mut self, position: Aabb2<f32>, context: &UiContext) {
        self.state.update(position, context);
        let main = position;
    }

    fn walk_states_mut(&mut self, f: &dyn Fn(&mut WidgetState)) {
        self.state.walk_states_mut(f);
    }
}
