use super::*;

use ctl_client::core::types::Name;

#[derive(Debug, Default)]
pub struct CheckboxWidget {
    pub state: WidgetState,
    pub text: TextWidget,
    pub check: WidgetState,
    pub checked: bool,
}

impl CheckboxWidget {
    pub fn new(text: impl Into<Name>) -> Self {
        Self {
            text: TextWidget::new(text),
            ..default()
        }
    }
}

impl Widget for CheckboxWidget {
    fn state_mut(&mut self) -> &mut WidgetState {
        &mut self.state
    }

    fn update(&mut self, position: Aabb2<f32>, context: &mut UiContext) {
        self.state.update(position, context);

        let check_size = position.height().min(context.font_size) * 0.5;
        let check_pos = Aabb2::point(vec2(position.min.x + check_size / 2.0, position.center().y))
            .extend_uniform(check_size / 2.0);
        self.check.update(check_pos, context);

        let text_pos = position.extend_left(-check_size * 1.1 - context.font_size * 0.2);
        self.text.align(vec2(0.0, 0.5));
        self.text.update(text_pos, context);
    }
}
