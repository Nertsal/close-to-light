use super::*;

use crate::util::Lerp;

pub struct ModifiersWidget {
    t: f32,
    pub active_mods: Vec<IconWidget>,
    pub body_slide: Bounded<f32>,
    pub head: TextWidget,
    pub body: WidgetState,
    pub description: Vec<TextWidget>,
    pub description_lerp: Lerp<f32>,
    pub mods: Vec<(ToggleButtonWidget, IconWidget, Modifier)>,
    pub score_multiplier: TextWidget,
    pub separator: WidgetState,
}

impl ModifiersWidget {
    pub fn new(assets: &Rc<Assets>) -> Self {
        Self {
            t: 0.0,
            active_mods: Vec::new(),
            body_slide: Bounded::new_zero(0.25),
            head: TextWidget::new("Modifiers"),
            body: WidgetState::new(),
            description: Vec::new(),
            description_lerp: Lerp::new_smooth(0.25, 0.0, 0.0),
            mods: enum_iterator::all::<Modifier>()
                .map(|modifier| {
                    (
                        ToggleButtonWidget::new_deselectable(format!("{}", modifier)),
                        IconWidget::new(assets.get_modifier(modifier)),
                        modifier,
                    )
                })
                .collect(),
            score_multiplier: TextWidget::new(""),
            separator: WidgetState::new(),
        }
    }

    pub fn update(&mut self, main: Aabb2<f32>, state: &mut MenuState, context: &mut UiContext) {
        let head_size = vec2(7.0 * context.layout_size, 1.1 * context.font_size);
        let head = main.align_aabb(head_size, vec2(0.5, 0.0));

        // Active mods
        self.active_mods = state
            .config
            .modifiers
            .iter()
            .map(|modifier| IconWidget::new(context.context.assets.get_modifier(modifier)))
            .collect();
        let mods = head.translate(vec2(0.0, head.height()));
        let mod_pos = mods.align_aabb(vec2(mods.height(), mods.height()), vec2(0.5, 0.5));
        let mods = mod_pos.stack_aligned(
            vec2(mod_pos.width(), 0.0),
            self.active_mods.len(),
            vec2(0.5, 0.5),
        );
        for (modifier, pos) in self.active_mods.iter_mut().zip(mods) {
            modifier.update(pos, context);
        }

        // Slide in when a level is selected
        let t = state.selected_level.as_ref().map_or(0.0, |show| {
            let mut t = show.time.get_ratio();
            if state.switch_level.is_some() {
                t = t.max(self.t);
            }
            t
        });
        self.t = t;
        let t = crate::util::smoothstep(t);
        let slide = vec2(0.0, context.screen.min.y - head.max.y);
        let main = main.translate(slide * (1.0 - t));
        let head = head.translate(slide * (1.0 - t));

        self.head.update(head, context);

        let t = crate::util::smoothstep(self.body_slide.get_ratio());

        let mut body_height = 4.0 * context.font_size + 0.1 * context.layout_size;
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

        if self.t > 0.0 && (self.head.state.hovered || self.body.hovered) {
            self.body_slide.change(context.delta_time);
        } else {
            self.body_slide.change(-context.delta_time);
        }

        if self.body.visible && body_size.y > 20.0 {
            let mut main = body.extend_uniform(-1.0 * context.layout_size);

            let buttons = main.cut_bottom(1.0 * context.font_size);
            let _icons = main.cut_bottom(0.7 * context.font_size);

            let mut multipler = main.cut_bottom(1.0 * context.font_size);
            multipler.cut_bottom(context.layout_size);
            self.score_multiplier.text =
                format!("Score x{:.2}", state.config.modifiers.multiplier()).into();
            self.score_multiplier
                .update(multipler, &context.scale_font(0.7));

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
        if let Some((_, _, modifier)) = self
            .mods
            .iter()
            .find(|(widget, _, _)| widget.text.state.hovered)
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

        let icon_size = vec2::splat(0.7) * context.font_size;

        let button_size = vec2(
            (main.width() - spacing * (columns as f32 - 1.0)) / columns as f32,
            1.0 * context.font_size,
        );
        let button = main.align_aabb(button_size, vec2(0.5, 0.5));
        let stack =
            button.stack_aligned(vec2(button_size.x + spacing, 0.0), columns, vec2(0.5, 0.5));
        for ((button, icon, modifier), pos) in self.mods.iter_mut().zip(stack) {
            let value = state.config.modifiers.get_mut(*modifier);
            button.selected = *value;
            button.update(pos, context);
            button.text.options.size = pos.height() * 0.8;
            *value = button.selected;

            let icon_pos = pos.align_aabb(icon_size, vec2(0.5, 1.0));
            let icon_pos =
                icon_pos.translate(vec2(0.0, icon_pos.height() + context.layout_size * 0.1));
            icon.update(icon_pos, context);
        }
    }
}
