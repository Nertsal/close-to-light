use super::*;

use crate::ui::layout::AreaOps;

use ctl_client::core::types::Name;

#[derive(Debug, Clone)]
pub struct TooltipWidget {
    pub visible: bool,
    pub state: WidgetState,
    pub title: TextWidget,
    pub text: TextWidget,
}

impl TooltipWidget {
    pub fn new() -> Self {
        Self {
            visible: false,
            state: WidgetState::new(),
            title: TextWidget::new("shortcut"),
            text: TextWidget::new("tip").aligned(vec2(0.5, 0.0)),
        }
    }

    pub fn update(&mut self, anchor: &WidgetState, tip: impl Into<Name>, context: &UiContext) {
        if !anchor.hovered {
            return;
        }
        self.visible = true;

        let mut position = Aabb2::point(anchor.position.top_right())
            .extend_positive(vec2::splat(context.font_size * 1.5));
        if position.max.x >= context.screen.max.x {
            position = position.translate(vec2(-anchor.position.width() - position.width(), 0.0));
        }
        self.state.update(position, context);

        let position = position.extend_uniform(-context.font_size * 0.2);

        let title = position.clone().cut_top(context.font_size * 0.3);
        self.title.update(title, &context.scale_font(0.7));

        self.text.text = tip.into();
        self.text.update(position, &context.scale_font(0.9));
    }
}

impl Widget for TooltipWidget {
    simple_widget_state!();
    fn draw(&self, context: &UiContext) -> Geometry {
        if !self.visible {
            return Geometry::new();
        }

        let position = self.state.position;
        let theme = context.theme();
        let width = context.font_size * 0.1;
        let mut geometry = context.geometry.quad_fill(position, width, theme.dark);
        geometry.merge(self.title.draw(context));
        geometry.merge(self.text.draw(context));
        geometry.merge(context.geometry.quad_outline(position, width, theme.light));
        geometry
    }
}
