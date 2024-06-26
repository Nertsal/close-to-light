use super::*;

use crate::{render::util::TextRenderOptions, ui::layout::AreaOps};

use ctl_client::core::types::Name;
use geng_utils::bounded::Bounded;

pub struct SliderWidget {
    pub state: WidgetState,
    pub text: TextWidget,
    pub bar: WidgetState,
    /// Hitbox
    pub bar_box: WidgetState,
    pub head: WidgetState,
    pub value: TextWidget,
    pub options: TextRenderOptions,
}

impl SliderWidget {
    pub fn new(text: impl Into<Name>) -> Self {
        Self {
            state: WidgetState::new(),
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

    fn state_mut(&mut self) -> &mut WidgetState {
        &mut self.state
    }

    fn update(&mut self, position: Aabb2<f32>, context: &mut UiContext, state: &mut Self::State) {
        self.state.update(position, context);

        self.options.update(context);
        let mut main = position;

        if !self.text.text.is_empty() {
            let text = main.cut_left(context.font_size * 5.0);
            self.text.show();
            self.text.align(vec2(1.0, 0.5));
            self.text.update(text, context);
        } else {
            self.text.hide();
        }

        let value = main.cut_right(context.font_size * 2.0);
        self.value.text = format!("{:.precision$}", state.value(), precision = 2).into();
        self.value.update(value, context);

        main.cut_left(context.layout_size * 0.5);
        let bar = Aabb2::point(main.align_pos(vec2(0.0, 0.5)))
            .extend_right(main.width())
            .extend_symmetric(vec2(0.0, context.font_size * 0.1) / 2.0);
        self.bar.update(bar, context);

        let bar_box = bar.extend_symmetric(vec2(0.0, context.font_size * 0.6) / 2.0);
        self.bar_box.update(bar_box, context);

        let head = Aabb2::point(main.align_pos(vec2(state.get_ratio(), 0.5)))
            .extend_symmetric(vec2(0.1, 0.6) * context.font_size / 2.0);
        self.head.update(head, context);

        if self.bar_box.pressed {
            let t =
                (context.cursor.position.x - self.bar.position.min.x) / self.bar.position.width();
            let t = t.clamp(0.0, 1.0);
            state.set_ratio(t);
        }
    }
}
