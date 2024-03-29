use super::*;

use crate::prelude::Color;

#[derive(Clone)]
pub struct IconWidget {
    pub state: WidgetState,
    pub texture: Rc<ugli::Texture>,
    pub color: Color,
}

impl IconWidget {
    pub fn new(texture: &Rc<ugli::Texture>) -> Self {
        Self {
            state: WidgetState::new(),
            texture: texture.clone(),
            color: Color::WHITE,
        }
    }
}

impl Widget for IconWidget {
    fn update(&mut self, position: Aabb2<f32>, context: &mut UiContext) {
        self.state.update(position, context);
    }

    fn walk_states_mut(&mut self, f: &dyn Fn(&mut WidgetState)) {
        self.state.walk_states_mut(f);
    }
}
