use super::*;

#[derive(Debug, Default)]
pub struct CheckboxWidget {
    pub text: TextWidget,
    pub check: WidgetState,
    pub checked: bool,
}

impl CheckboxWidget {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: TextWidget::new(text),
            ..default()
        }
    }
}

impl Widget for CheckboxWidget {
    fn update(&mut self, position: Aabb2<f32>, context: &UiContext) {
        let check_size = position.height().min(context.font_size) * 0.5;
        let check_pos = Aabb2::point(vec2(position.min.x + check_size / 2.0, position.center().y))
            .extend_uniform(check_size / 2.0);
        self.check.update(check_pos, context);

        let text_pos = position.extend_left(-check_size * 1.1);
        self.text.update(text_pos, context);
    }

    fn walk_states_mut(&mut self, f: &dyn Fn(&mut WidgetState)) {
        self.check.walk_states_mut(f);
        self.text.walk_states_mut(f);
    }
}
