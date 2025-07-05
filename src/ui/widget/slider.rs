use super::*;

use crate::{
    render::util::{TextRenderOptions, update_text_options},
    ui::layout::AreaOps,
};

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
    pub display_precision: usize,
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
            display_precision: 2,
        }
    }

    pub fn with_display_precision(self, precision: usize) -> Self {
        Self {
            display_precision: precision,
            ..self
        }
    }

    pub fn update(&mut self, position: Aabb2<f32>, context: &UiContext, state: &mut Bounded<f32>) {
        self.state.update(position, context);

        update_text_options(&mut self.options, context);
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
        self.value.text = format!(
            "{:.precision$}",
            state.value(),
            precision = self.display_precision
        )
        .into();
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

        if self.bar_box.pressed && self.bar.position.width() > 0.0 {
            let t =
                (context.cursor.position.x - self.bar.position.min.x) / self.bar.position.width();
            let t = t.clamp(0.0, 1.0);
            state.set_ratio(t);
        }
    }
}

impl Widget for SliderWidget {
    simple_widget_state!();
    fn draw(&self, context: &UiContext) -> Geometry {
        let theme = context.theme();
        let width = context.font_size * 0.1;
        let mut geometry = self.text.draw(context);
        geometry.merge(self.value.draw(context));
        geometry.merge(
            context
                .geometry
                .quad_fill(self.bar.position, width, theme.light),
        );
        geometry.merge(
            context
                .geometry
                .quad_fill(self.head.position, width, theme.highlight),
        );
        geometry
    }
}
