use super::*;

use crate::{local::CachedGroup, ui::layout::AreaOps};

use generational_arena::Index;
use geng_utils::bounded::Bounded;

pub struct GroupWidget {
    pub group: Index,
    pub state: WidgetState,
    pub logo: WidgetState,
    pub name: TextWidget,
    pub author: TextWidget,
    pub selected_time: Bounded<f32>,
}

impl GroupWidget {
    pub fn new() -> Self {
        Self {
            group: Index::from_raw_parts(0, 0),
            state: WidgetState::new(),
            logo: WidgetState::new(),
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

impl Widget for GroupWidget {
    fn state_mut(&mut self) -> &mut WidgetState {
        &mut self.state
    }

    fn update(&mut self, position: Aabb2<f32>, context: &mut UiContext) {
        self.state.update(position, context);

        // let logo_size = position.height();
        // let (logo, position) = layout::cut_left_right(position, logo_size);
        // self.logo.update(logo, context);

        // let (name, author) = layout::cut_top_down(position, context.font_size);
        let mut author = position;
        let name = author.split_top(0.5);
        let margin = context.font_size * 0.2;
        // let name = name.extend_down(-margin);
        author.cut_top(margin);

        self.name.update(name, context);
        self.name.align(vec2(0.5, 0.0));

        self.author.update(author, &mut context.scale_font(0.6)); // TODO: better
        self.author.align(vec2(0.5, 1.0));
    }
}
