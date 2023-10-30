use super::*;

#[derive(Debug, Default)]
pub struct TextWidget {
    pub state: WidgetState,
    pub text: String,
    pub font_size: f32,
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
    fn update(&mut self, position: Aabb2<f32>, context: &UiContext) {
        self.state.update(position, context);
        self.font_size = context.font_size;
    }

    fn walk_states_mut(&mut self, f: &dyn Fn(&mut WidgetState)) {
        self.state.walk_states_mut(f);
    }
}
