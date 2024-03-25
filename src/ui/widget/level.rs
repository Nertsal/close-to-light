use super::*;

use crate::{prelude::Assets, ui::layout::AreaOps};

use ctl_client::core::types::LevelInfo;
use geng_utils::bounded::Bounded;

pub struct LevelWidget {
    pub info: LevelInfo,

    pub state: WidgetState,

    pub static_state: WidgetState,
    pub sync: IconButtonWidget,
    pub edit: IconButtonWidget,

    pub name: TextWidget,
    pub author: TextWidget,
    pub selected_time: Bounded<f32>,
}

impl LevelWidget {
    pub fn new(assets: &Assets) -> Self {
        Self {
            info: LevelInfo::default(),

            state: WidgetState::new(),

            static_state: WidgetState::new(),
            sync: IconButtonWidget::new_normal(&assets.sprites.reset),
            edit: IconButtonWidget::new_normal(&assets.sprites.edit),

            name: TextWidget::new("<level name>"),
            author: TextWidget::new("by <author>"),
            selected_time: Bounded::new_zero(0.2),
        }
    }

    pub fn set_level(&mut self, meta: &LevelInfo) {
        self.info = meta.clone();
        self.name.text = meta.name.to_string();
        self.author.text = format!("by {}", meta.authors());
    }
}

impl Widget for LevelWidget {
    fn state_mut(&mut self) -> &mut WidgetState {
        &mut self.state
    }

    fn update(&mut self, position: Aabb2<f32>, context: &mut UiContext) {
        self.state.update(position, context);
        context.update_focus(self.state.hovered);

        // `static_state` assumed to be updated externally
        let mut stat = self
            .static_state
            .position
            .extend_uniform(-context.font_size * 0.15);
        stat.cut_right(stat.width() - stat.height() / 2.0);
        let rows = stat.split_rows(2);

        self.sync.update(rows[0], context);
        self.edit.update(rows[1], context);

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
