use super::*;

#[derive(Debug, Default)]
pub struct TextWidget {
    pub state: WidgetState,
    pub text: String,
}

impl TextWidget {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            ..default()
        }
    }
}

impl Widget for TextWidget {
    fn update(&mut self, position: Aabb2<f32>, cursor_position: vec2<f32>, cursor_down: bool) {
        self.state.update(position, cursor_position, cursor_down);
    }

    fn walk_states_mut(&mut self, f: &dyn Fn(&mut WidgetState)) {
        self.state.walk_states_mut(f);
    }
}
