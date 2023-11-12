use super::*;

use crate::{menu::GroupEntry, ui::layout};

#[derive(Default)]
pub struct GroupWidget {
    pub state: WidgetState,
    pub logo: WidgetState,
    pub name: TextWidget,
    pub author: TextWidget,
}

impl GroupWidget {
    pub fn set_group(&mut self, group: &GroupEntry) {
        self.name.text = group.meta.name.to_string();
        self.author.text = format!("by {}", group.meta.music.author);
    }
}

impl Widget for GroupWidget {
    fn update(&mut self, position: Aabb2<f32>, context: &UiContext) {
        self.state.update(position, context);

        let logo_size = position.height();
        let (logo, position) = layout::cut_left_right(position, logo_size);
        self.logo.update(logo, context);

        // let (name, author) = layout::cut_top_down(position, context.font_size);
        let (name, author) = layout::split_top_down(position, 0.5);
        let margin = context.font_size * 0.1;
        let name = name.extend_down(-margin);
        let author = author.extend_up(-margin);

        self.name.update(name, context);
        self.name.align(vec2(0.0, 0.0));

        self.author.update(author, &context.scale_font(0.75));
        self.author.align(vec2(0.0, 1.0));
    }

    fn walk_states_mut(&mut self, f: &dyn Fn(&mut WidgetState)) {
        self.state.walk_states_mut(f);
        self.logo.walk_states_mut(f);
        self.name.walk_states_mut(f);
        self.author.walk_states_mut(f);
    }
}

pub struct PlayLevelWidget {
    pub state: WidgetState,
    // pub name: TextWidget,
    pub level_normal: ButtonWidget,
    pub credits_normal: TextWidget,
    pub level_hard: ButtonWidget,
    pub credits_hard: TextWidget,
    // pub music_credits: TextWidget,
}

impl PlayLevelWidget {
    pub fn new() -> Self {
        Self {
            state: WidgetState::default(),
            // name: TextWidget::new("<level name>"),
            level_normal: ButtonWidget::new("Normal"),
            credits_normal: TextWidget::new("<normal credits>"),
            level_hard: ButtonWidget::new("Hard"),
            credits_hard: TextWidget::new("<hard credits>"),
            // music_credits: TextWidget::new("<music credits>"),
        }
    }

    pub fn set_group(&mut self, group: &GroupEntry) {
        // self.name.text = group.meta.name.to_string();
        self.credits_normal.text = format!("by {}", group.meta.normal.author);
        self.credits_hard.text = format!("by {}", group.meta.hard.author);
        // self.music_credits.text = format!("Music by {}", group.meta.music.author);
    }
}

impl Widget for PlayLevelWidget {
    fn update(&mut self, position: Aabb2<f32>, context: &UiContext) {
        self.state.update(position, context);

        let position = position.extend_uniform(-context.font_size * 0.5);

        // let (name, main) = layout::cut_top_down(position, context.font_size * 1.5);
        // self.name.update(name, context);
        let main = position;

        // {
        //     let context = context.scale_font(0.75);
        //     let music =
        //         layout::align_aabb(vec2(7.5, 1.0) * context.font_size, name, vec2(1.0, 0.5));
        //     self.music_credits.update(music, &context);
        // }

        let (_main, bottom) = layout::cut_top_down(main, main.height() - context.font_size * 2.5);
        for (pos, (button, credits)) in layout::split_columns(bottom, 2).into_iter().zip([
            (&mut self.level_normal, &mut self.credits_normal),
            (&mut self.level_hard, &mut self.credits_hard),
        ]) {
            let pos = layout::fit_aabb_height(vec2(4.0, 3.0), pos, 0.5);
            let (button_pos, credits_pos) = layout::cut_top_down(pos, context.font_size * 1.2);
            button.update(button_pos, context);
            credits.update(credits_pos, &context.scale_font(0.75));
        }
    }

    fn walk_states_mut(&mut self, f: &dyn Fn(&mut WidgetState)) {
        self.state.walk_states_mut(f);
        // self.name.walk_states_mut(f);
        self.level_normal.walk_states_mut(f);
        self.credits_normal.walk_states_mut(f);
        self.level_hard.walk_states_mut(f);
        self.credits_hard.walk_states_mut(f);
        // self.music_credits.walk_states_mut(f);
    }
}
