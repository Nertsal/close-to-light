use super::*;

use crate::util::Lerp;

pub struct ModifiersWidget {
    pub body_slide: Bounded<f32>,
    pub head: TextWidget,
    pub body: WidgetState,
    pub description: Vec<TextWidget>,
    pub description_lerp: Lerp<f32>,
    pub mods: Vec<(ToggleWidget, Modifier)>,
    pub separator: WidgetState,
}

impl ModifiersWidget {
    pub fn new() -> Self {
        Self {
            body_slide: Bounded::new_zero(0.25),
            head: TextWidget::new("Modifiers"),
            body: WidgetState::new(),
            description: Vec::new(),
            description_lerp: Lerp::new_smooth(0.25, 0.0, 0.0),
            mods: enum_iterator::all::<Modifier>()
                .map(|modifier| {
                    (
                        ToggleWidget::new_deselectable(format!("{}", modifier)),
                        modifier,
                    )
                })
                .collect(),
            separator: WidgetState::new(),
        }
    }

    pub fn update(&mut self, main: Aabb2<f32>, state: &mut MenuState, context: &mut UiContext) {
        let head_size = vec2(7.0 * context.layout_size, 1.1 * context.font_size);
        let head = main.align_aabb(head_size, vec2(0.5, 0.0));

        // Slide in when a level is selected
        let t = state
            .selected_level
            .as_ref()
            .map_or(0.0, |show| show.time.get_ratio());
        let t = crate::util::smoothstep(t);
        let slide = vec2(0.0, context.screen.min.y - head.max.y);
        let main = main.translate(slide * (1.0 - t));
        let head = head.translate(slide * (1.0 - t));

        self.head.update(head, context);

        let t = crate::util::smoothstep(self.body_slide.get_ratio());

        let mut body_height = 2.0 * context.font_size + 0.1 * context.layout_size;
        if self.description_lerp.current() > 0.0 {
            body_height += self.description_lerp.current();
        }
        let body_size = vec2(17.0 * context.layout_size, body_height);
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
            self.body_slide.change(context.delta_time);
        } else {
            self.body_slide.change(-context.delta_time);
        }

        if self.body.visible && body_size.y > 20.0 {
            let mut main = body.extend_uniform(-1.0 * context.layout_size);

            let buttons = main.cut_bottom(1.0 * context.font_size);
            let separator = main.cut_bottom(1.0 * context.layout_size);
            let separator = separator.align_aabb(
                vec2(separator.width(), 0.1 * context.layout_size),
                vec2(0.5, 1.0),
            );
            main.cut_bottom(0.5 * context.layout_size);
            self.separator.update(separator, context);
            self.update_description(main, context);
            self.update_buttons(buttons, state, context);
        }
    }

    pub fn update_description(&mut self, main: Aabb2<f32>, context: &mut UiContext) {
        if let Some((_, modifier)) = self
            .mods
            .iter()
            .find(|(widget, _)| widget.text.state.hovered)
        {
            let lines = crate::util::wrap_text(
                &context.font,
                modifier.description(),
                main.width() / context.font_size,
            );
            let row = main.align_aabb(vec2(main.width(), 0.8 * context.font_size), vec2(0.5, 0.0));
            let rows = row.stack(vec2(0.0, row.height()), lines.len());

            self.description_lerp
                .change_target(context.layout_size + row.height() * lines.len() as f32);
            self.description = lines
                .into_iter()
                .rev()
                .zip(rows)
                .map(|(line, pos)| {
                    let mut text = TextWidget::new(line);
                    text.update(pos, context);
                    text.options.size = pos.height();
                    text
                })
                .collect();
        } else {
            self.description_lerp.change_target(0.0);
        }
        self.description_lerp.update(context.delta_time);
    }

    pub fn update_buttons(
        &mut self,
        main: Aabb2<f32>,
        state: &mut MenuState,
        context: &mut UiContext,
    ) {
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
