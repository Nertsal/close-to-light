use geng_utils::bounded::Bounded;

use super::*;

pub struct ValueWidget<T> {
    pub state: WidgetState,
    pub text: TextWidget,
    pub value: Bounded<T>,
    pub value_text: TextWidget,
    pub scroll_by: T,
}

impl<T: Num + Copy> ValueWidget<T> {
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

impl<T: Num + Display> Widget for ValueWidget<T> {
    fn update(&mut self, position: Aabb2<f32>, context: &UiContext) {
        self.state.update(position, context);

        if self.state.hovered {
            let sign = if context.cursor.scroll.approx_eq(&0.0) {
                T::ZERO
            } else if context.cursor.scroll > 0.0 {
                T::ONE
            } else {
                -T::ONE
            };
            self.value.change(sign * self.scroll_by);
        }

        self.text.align(vec2(0.0, 0.5));
        self.text.update(position, context);

        self.value_text.align(vec2(1.0, 0.5));
        self.value_text.text = format!("{}", self.value.value());
        self.value_text.update(position, context);
    }

    fn walk_states_mut(&mut self, f: &dyn Fn(&mut WidgetState)) {
        self.state.walk_states_mut(f);
        self.text.walk_states_mut(f);
        self.value_text.walk_states_mut(f);
    }
}
