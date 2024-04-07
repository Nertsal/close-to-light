use super::*;

use crate::{
    local::{CachedGroup, LevelCache},
    prelude::Assets,
    ui::layout::AreaOps,
};

use generational_arena::Index;
use geng_utils::bounded::Bounded;

pub struct GroupWidget {
    pub group: Index,
    pub state: WidgetState,

    pub static_state: WidgetState,
    pub delete: IconButtonWidget,
    pub name: TextWidget,
    pub author: TextWidget,

    pub selected_time: Bounded<f32>,
}

impl GroupWidget {
    pub fn new(assets: &Rc<Assets>) -> Self {
        Self {
            group: Index::from_raw_parts(0, 0),
            state: WidgetState::new(),

            static_state: WidgetState::new(),
            delete: IconButtonWidget::new_danger(&assets.sprites.trash),
            name: TextWidget::new("<level name>"),
            author: TextWidget::new("by <author>"),

            selected_time: Bounded::new_zero(0.2),
        }
    }

    pub fn set_group(&mut self, group: &CachedGroup, index: Index) {
        self.group = index;
        let name = group
            .music
            .as_ref()
            .map_or("<name>".into(), |music| music.meta.name.clone());
        self.name.text = name;
        if let Some(music) = &group.music {
            self.author.text = format!("by {}", music.meta.authors());
        }
    }
}

impl StatefulWidget for GroupWidget {
    type State = LevelCache;

    fn state_mut(&mut self) -> &mut WidgetState {
        &mut self.state
    }

    fn update(&mut self, position: Aabb2<f32>, context: &mut UiContext, state: &mut Self::State) {
        self.state.update(position, context);

        let mut stat = self
            .static_state
            .position
            .extend_uniform(-context.font_size * 0.15);
        stat.cut_right(stat.width() - stat.height() / 2.0);
        let rows = stat.split_rows(2);

        self.delete.update(rows[1], context);
        if self.delete.state.clicked {
            // TODO: confirmation window
            state.delete_group(self.group);
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
}
