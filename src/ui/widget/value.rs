use geng_utils::bounded::Bounded;

use super::*;

pub struct ValueWidget<T> {
    pub state: WidgetState,
    pub text: TextWidget,
    pub value: Bounded<T>,
    pub value_text: TextWidget,
    pub scroll_by: T,
    /// Whether to wrap around the bounds.
    pub wrap: bool,
}

impl<T: Num + Copy> ValueWidget<T> {
    pub fn new(text: impl Into<String>, value: T, range: RangeInclusive<T>, scroll_by: T) -> Self {
        Self {
            state: WidgetState::new(),
            text: TextWidget::new(text),
            value: Bounded::new(value, range),
            value_text: TextWidget::new("<value>"),
            scroll_by,
            wrap: false,
        }
    }

    pub fn wrapping(self) -> Self {
        Self { wrap: true, ..self }
    }
}

impl<T: Num + Display> StatefulWidget for ValueWidget<T> {
    type State = T;

    fn update(&mut self, position: Aabb2<f32>, context: &mut UiContext, state: &mut T) {
        self.value.set(*state);
        self.state.update(position, context);

        if self.state.hovered {
            let sign = if context.cursor.scroll.approx_eq(&0.0) {
                T::ZERO
            } else if context.cursor.scroll > 0.0 {
                T::ONE
            } else {
                -T::ONE
            };

            let mut target = self.value.value() + sign * self.scroll_by;
            if self.wrap {
                // TODO: move to Bounded
                let range = self.value.max() - self.value.min();
                if target > self.value.max() {
                    target -= range;
                } else if target < self.value.min() {
                    target += range;
                }
            }
            self.value.set(target);
        }

        self.text.align(vec2(0.0, 0.5));
        self.text.update(position, context);

        self.value_text.align(vec2(1.0, 0.5));
        self.value_text.text = format!("{}", self.value.value());
        self.value_text.update(position, context);

        *state = self.value.value();
    }

    fn walk_states_mut(&mut self, f: &dyn Fn(&mut WidgetState)) {
        self.state.walk_states_mut(f);
        self.text.walk_states_mut(f);
        self.value_text.walk_states_mut(f);
    }
}
