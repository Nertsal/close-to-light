use geng_utils::bounded::Bounded;

use super::*;

pub struct ValueWidget<T> {
    pub state: WidgetState,
    pub text: TextWidget,
    pub value: Bounded<T>,
    pub value_text: TextWidget,
    pub scroll_by: T,
}

impl<T: PartialOrd + Copy> ValueWidget<T> {
    pub fn new(text: impl Into<String>, value: T, range: RangeInclusive<T>, scroll_by: T) -> Self {
        Self {
            state: WidgetState::new(),
            text: TextWidget::new(text),
            value: Bounded::new(value, range),
            value_text: TextWidget::new("<value>"),
            scroll_by,
        }
    }
}

impl<T: Display> Widget for ValueWidget<T> {
    fn update(&mut self, position: Aabb2<f32>, context: &UiContext) {
        self.state.update(position, context);
    }

    fn walk_states_mut(&mut self, f: &dyn Fn(&mut WidgetState)) {
        self.state.walk_states_mut(f);
        self.text.walk_states_mut(f);
    }
}
