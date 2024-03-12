use super::*;

use crate::{
    prelude::{Assets, LevelMeta},
    ui::layout::AreaOps,
};

use geng_utils::bounded::Bounded;

pub struct LevelWidget {
    pub state: WidgetState,

    pub static_state: WidgetState,
    pub sync: IconWidget,
    pub edit: IconWidget,

    pub name: TextWidget,
    pub author: TextWidget,
    pub selected_time: Bounded<f32>,
}

impl LevelWidget {
    pub fn new(assets: &Assets) -> Self {
        Self {
            state: WidgetState::new(),

            static_state: WidgetState::new(),
            sync: IconWidget::new(&assets.sprites.reset),
            edit: IconWidget::new(&assets.sprites.edit),

            name: TextWidget::new("<level name>"),
            author: TextWidget::new("by <author>"),
            selected_time: Bounded::new_zero(0.2),
        }
    }

    pub fn set_level(&mut self, meta: &LevelMeta) {
        self.name.text = meta.name.to_string();
        self.author.text = format!("by {}", meta.author);
    }
}

impl Widget for LevelWidget {
    fn update(&mut self, position: Aabb2<f32>, context: &mut UiContext) {
        self.state.update(position, context);

        // `static_state` assumed to be updated externally
        let mut stat = self
            .static_state
            .position
            .extend_uniform(-context.font_size * 0.15);
        stat.cut_right(stat.width() - stat.height() / 2.0);
        let rows = stat.split_rows(2);

        self.sync.update(rows[0], context);
        self.sync.background = None;
        if self.sync.state.hovered {
            self.sync.color = context.theme.dark;
            self.sync.background = Some(context.theme.light);
        }

        self.edit.update(rows[1], context);
        self.edit.background = None;
        if self.edit.state.hovered {
            self.edit.color = context.theme.dark;
            self.edit.background = Some(context.theme.light);
        }

        let mut author = position;
        let name = author.split_top(0.5);
        let margin = context.font_size * 0.2;
        author.cut_top(margin);

        self.name.update(name, context);
        self.name.align(vec2(0.5, 0.0));

        self.author.update(author, &mut context.scale_font(0.6)); // TODO: better
        self.author.align(vec2(0.5, 1.0));
    }

    fn walk_states_mut(&mut self, f: &dyn Fn(&mut WidgetState)) {
        self.state.walk_states_mut(f);

        self.static_state.walk_states_mut(f);
        self.sync.walk_states_mut(f);
        self.edit.walk_states_mut(f);

        self.name.walk_states_mut(f);
        self.author.walk_states_mut(f);
    }
}
