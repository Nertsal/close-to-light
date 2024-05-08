use super::*;

pub struct ModifiersWidget {
    pub slide: Bounded<f32>,
    pub head: TextWidget,
    pub body: WidgetState,
    pub mods: Vec<(ToggleWidget, Modifier)>,
}

impl ModifiersWidget {
    pub fn new() -> Self {
        Self {
            slide: Bounded::new_zero(0.25),
            head: TextWidget::new("Modifiers"),
            body: WidgetState::new(),
            mods: enum_iterator::all::<Modifier>()
                .map(|modifier| {
                    (
                        ToggleWidget::new_deselectable(format!("{}", modifier)),
                        modifier,
                    )
                })
                .collect(),
        }
    }

    pub fn update(&mut self, main: Aabb2<f32>, state: &mut MenuState, context: &mut UiContext) {
        let head_size = vec2(7.0 * context.layout_size, 1.1 * context.font_size);
        let head = main.align_aabb(head_size, vec2(0.5, 0.0));
        self.head.update(head, context);

        let t = crate::util::smoothstep(self.slide.get_ratio());
        let body_size = vec2(15.0 * context.layout_size, 2.0 * context.font_size);
        let body = main
            .align_aabb(body_size, vec2(0.5, 0.0))
            .translate(vec2(0.0, head.height()));
        if body.height() * t <= 1.0 {
            self.body.hide();
        } else {
            self.body.show();
            let body = body.with_height(body.height() * t, 0.0);
            self.body.update(body, context);
        }

        if self.head.state.hovered || self.body.hovered {
            self.slide.change(context.delta_time);
        } else {
            self.slide.change(-context.delta_time);
        }

        if self.body.visible && body_size.y > 20.0 {
            let main = body.extend_uniform(-1.0 * context.layout_size);
            let columns = self.mods.len();
            let spacing = 1.0 * context.layout_size;
            let button_size = vec2(
                (main.width() - spacing * (columns as f32 - 1.0)) / columns as f32,
                1.0 * context.font_size,
            );
            let button = main.align_aabb(button_size, vec2(0.5, 0.5));
            let stack =
                button.stack_aligned(vec2(button_size.x + spacing, 0.0), columns, vec2(0.5, 0.5));
            for ((button, modifier), pos) in self.mods.iter_mut().zip(stack) {
                let value = state.config.modifiers.get_mut(*modifier);
                button.selected = *value;
                button.update(pos, context);
                button.text.options.size = pos.height() * 0.8;
                *value = button.selected;
            }
        }
    }
}
