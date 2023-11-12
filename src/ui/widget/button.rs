use super::*;

#[derive(Debug, Clone, Default)]
pub struct ButtonWidget {
    pub text: TextWidget,
}

impl ButtonWidget {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: TextWidget::new(text),
        }
    }
}

impl Widget for ButtonWidget {
    fn update(&mut self, position: Aabb2<f32>, context: &UiContext) {
        self.text.update(position, context);
    }

    fn walk_states_mut(&mut self, f: &dyn Fn(&mut WidgetState)) {
        self.text.walk_states_mut(f);
    }
}
