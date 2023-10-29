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
    fn update(&mut self, position: Aabb2<f32>, cursor_position: vec2<f32>, cursor_down: bool) {
        let check_size = position.size() * 0.9;
        let check_size = check_size.x.max(check_size.y);
        let check_pos = Aabb2::point(vec2(position.min.x + check_size / 2.0, position.center().y))
            .extend_uniform(check_size / 2.0);
        self.check.update(check_pos, cursor_position, cursor_down);

        let text_pos = position.extend_left(-check_size);
        self.text.update(text_pos, cursor_position, cursor_down);
    }

    fn walk_states_mut(&mut self, f: &dyn Fn(&mut WidgetState)) {
        self.check.walk_states_mut(f);
        self.text.walk_states_mut(f);
    }
}
