use super::*;

use crate::layout::AreaOps;

use ctl_core::types::Name;
use ctl_render_core::TextRenderOptions;
use geng_utils::bounded::Bounded;

pub struct SliderWidget {
    pub state: WidgetState,
    pub text: TextWidget,
    pub bar: WidgetState,
    /// Hitbox
    pub bar_box: WidgetState,
    pub is_dragging: bool,
    pub lock_drag: bool,
    pub head: WidgetState,
    pub value: TextWidget,
    pub options: TextRenderOptions,
    pub display_precision: usize,
}

impl SliderWidget {
    pub fn new(text: impl Into<Name>) -> Self {
        Self {
            state: WidgetState::new(),
            text: TextWidget::new(text).aligned(vec2(0.0, 0.5)),
            bar: WidgetState::new(),
            bar_box: WidgetState::new(),
            is_dragging: false,
            lock_drag: false,
            head: WidgetState::new(),
            value: TextWidget::new("").aligned(vec2(1.0, 0.5)),
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

    pub fn update_value(
        &mut self,
        position: Aabb2<f32>,
        context: &UiContext,
        state: &mut f32,
        range: RangeInclusive<f32>,
    ) {
        let mut bounded = Bounded::new(*state, range);
        self.update(position, context, &mut bounded);
        *state = bounded.value();
    }

    pub fn update(&mut self, position: Aabb2<f32>, context: &UiContext, state: &mut Bounded<f32>) {
        self.state.update(
            position.with_width(position.width() + context.font_size * 0.3, 0.5),
            context,
        );

        crate::update_text_options(&mut self.options, context);
        let mut main = position;

        if !self.text.text.is_empty() {
            let text_width = (context.font_size * 5.0).min(main.width() * 0.4);
            let text = main.cut_left(text_width);
            self.text.show();
            self.text.update(text, context);
        } else {
            self.text.hide();
        }

        let value = main.cut_right(context.font_size * 1.0);
        self.value.text = format!(
            "{:.precision$}",
            state.value(),
            precision = self.display_precision
        )
        .into();
        self.value.update(value, context);

        main.cut_left(context.layout_size * 0.1);
        let bar = Aabb2::point(main.align_pos(vec2(0.0, 0.5)))
            .extend_right(main.width())
            .extend_symmetric(vec2(0.0, context.font_size * 0.1) / 2.0);
        self.bar.update(bar, context);

        let bar_box = bar.extend_symmetric(vec2(0.0, context.font_size * 0.6) / 2.0);
        self.bar_box.update(bar_box, context);

        let head = Aabb2::point(main.align_pos(vec2(state.get_ratio(), 0.5)))
            .extend_symmetric(vec2(0.1, 0.6) * context.font_size / 2.0);
        self.head.update(head, context);

        if self.bar.position.width() > 0.0 {
            let cursor_t =
                (context.cursor.position.x - self.bar.position.min.x) / self.bar.position.width();
            let cursor_t = cursor_t.clamp(0.0, 1.0);
            if self.bar_box.mouse_left.clicked {
                state.set_ratio(cursor_t);
            } else if self.bar_box.mouse_left.pressed.is_some() {
                if self.is_dragging {
                    context.total_focus();
                    state.set_ratio(cursor_t);
                } else if !self.lock_drag {
                    let delta = context.cursor.position - context.cursor.last_position;
                    if delta != vec2::ZERO {
                        if (delta.arg().as_degrees() / 90.0 + 0.5).floor() as i32 % 2 == 0 {
                            // Only horizontal drag counts
                            self.is_dragging = true;
                        } else {
                            self.lock_drag = true;
                        }
                    }
                }
            } else {
                self.is_dragging = false;
                self.lock_drag = false;
            }
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
