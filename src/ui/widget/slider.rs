use super::*;

use crate::{render::util::TextRenderOptions, ui::layout};

use geng_utils::bounded::Bounded;

pub struct SliderWidget {
    pub text: TextWidget,
    pub bar: WidgetState,
    /// Hitbox
    pub bar_box: WidgetState,
    pub head: WidgetState,
    pub value: TextWidget,
    pub options: TextRenderOptions,
}

impl SliderWidget {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: TextWidget::new(text),
            bar: WidgetState::new(),
            bar_box: WidgetState::new(),
            head: WidgetState::new(),
            value: TextWidget::new(""),
            options: TextRenderOptions::default(),
        }
    }
}

impl StatefulWidget for SliderWidget {
    type State = Bounded<f32>;

    fn update(&mut self, position: Aabb2<f32>, context: &mut UiContext, state: &mut Self::State) {
        self.options.update(context);
        let mut main = position;

        if !self.text.text.is_empty() {
            let (text, m) = layout::cut_left_right(main, context.font_size * 5.0);
            self.text.show();
            self.text.align(vec2(1.0, 0.5));
            self.text.update(text, context);
            main = m;
        } else {
            self.text.hide();
        }

        let (main, value) = layout::cut_left_right(main, main.width() - context.font_size * 3.0);
        self.value.text = format!("{:.precision$}", state.value(), precision = 2);
        self.value.update(value, context);

        let bar = Aabb2::point(layout::aabb_pos(main, vec2(0.0, 0.5)))
            .extend_right(main.width())
            .extend_symmetric(vec2(0.0, context.font_size * 0.1) / 2.0);
        self.bar.update(bar, context);

        let bar_box = bar.extend_symmetric(vec2(0.0, context.font_size * 0.6) / 2.0);
        self.bar_box.update(bar_box, context);

        let head = Aabb2::point(layout::aabb_pos(main, vec2(state.get_ratio(), 0.5)))
            .extend_symmetric(vec2(0.1, 0.6) * context.font_size / 2.0);
        self.head.update(head, context);

        if self.bar_box.pressed {
            let t =
                (context.cursor.position.x - self.bar.position.min.x) / self.bar.position.width();
            let t = t.clamp(0.0, 1.0);
            state.set_ratio(t);
        }
    }

    fn walk_states_mut(&mut self, f: &dyn Fn(&mut WidgetState)) {
        self.text.walk_states_mut(f);
        self.bar.walk_states_mut(f);
        self.head.walk_states_mut(f);
        self.value.walk_states_mut(f);
    }
}
