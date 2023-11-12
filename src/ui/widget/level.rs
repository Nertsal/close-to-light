use super::*;

use crate::{menu::GroupEntry, ui::layout};

// use geng_utils::bounded::Bounded;

pub struct PlayLevelWidget {
    pub state: WidgetState,
    pub level_normal: ButtonWidget,
    pub credits_normal: TextWidget,
    pub level_hard: ButtonWidget,
    pub credits_hard: TextWidget,
}

impl PlayLevelWidget {
    pub fn new() -> Self {
        Self {
            state: WidgetState::default(),
            level_normal: ButtonWidget::new("Normal"),
            credits_normal: TextWidget::new("<normal credits>"),
            level_hard: ButtonWidget::new("Hard"),
            credits_hard: TextWidget::new("<hard credits>"),
        }
    }

    pub fn set_group(&mut self, group: &GroupEntry) {
        self.credits_normal.text = format!("by {}", group.meta.normal.author);
        self.credits_hard.text = format!("by {}", group.meta.hard.author);
    }
}

impl Widget for PlayLevelWidget {
    fn update(&mut self, position: Aabb2<f32>, context: &UiContext) {
        self.state.update(position, context);

        let main = position.extend_uniform(-context.font_size * 0.5);

        // Levels
        let (levels_pos, _main) = layout::cut_top_down(main, context.font_size * 2.5);
        let levels = [
            (&mut self.level_normal, &mut self.credits_normal),
            (&mut self.level_hard, &mut self.credits_hard),
        ];
        for (pos, (button, credits)) in layout::split_columns(levels_pos, levels.len())
            .into_iter()
            .zip(levels)
        {
            let pos = layout::fit_aabb_height(vec2(4.0, 3.0), pos, 0.5);
            let (button_pos, credits_pos) = layout::cut_top_down(pos, context.font_size * 1.2);
            button.update(button_pos, context);
            credits.update(credits_pos, &context.scale_font(0.75));
        }
    }

    fn walk_states_mut(&mut self, f: &dyn Fn(&mut WidgetState)) {
        self.state.walk_states_mut(f);
        self.level_normal.walk_states_mut(f);
        self.credits_normal.walk_states_mut(f);
        self.level_hard.walk_states_mut(f);
        self.credits_hard.walk_states_mut(f);
    }
}
