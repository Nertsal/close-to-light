use super::*;

use crate::{local::CachedGroup, ui::layout::AreaOps};

use geng_utils::bounded::Bounded;

pub struct GroupWidget {
    pub state: WidgetState,
    pub logo: WidgetState,
    pub name: TextWidget,
    pub author: TextWidget,
    pub selected_time: Bounded<f32>,
}

impl GroupWidget {
    pub fn new() -> Self {
        Self {
            state: WidgetState::new(),
            logo: WidgetState::new(),
            name: TextWidget::new("<level name>"),
            author: TextWidget::new("by <author>"),
            selected_time: Bounded::new_zero(0.2),
        }
    }

    pub fn set_group(&mut self, group: &CachedGroup) {
        self.name.text = group.meta.name.to_string();
        if let Some(music) = &group.music {
            self.author.text = format!("by {}", music.meta.author);
        }
    }
}

impl Widget for GroupWidget {
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

    fn walk_states_mut(&mut self, f: &dyn Fn(&mut WidgetState)) {
        self.state.walk_states_mut(f);
        self.logo.walk_states_mut(f);
        self.name.walk_states_mut(f);
        self.author.walk_states_mut(f);
    }
}
