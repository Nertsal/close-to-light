use super::*;

use crate::{prelude::Assets, ui::layout::AreaOps};

use ctl_client::core::types::Name;

#[derive(Debug)]
pub struct ConfirmPopup<T> {
    pub action: T,
    pub title: Name,
    pub message: Name,
}

pub struct ConfirmWidget {
    pub window: UiWindow<()>,
    pub offset: vec2<f32>,
    pub state: WidgetState,
    pub title: TextWidget,
    pub message: TextWidget,
    pub confirm: IconButtonWidget,
    pub discard: IconButtonWidget,
}

impl ConfirmWidget {
    pub fn new(assets: &Rc<Assets>, title: impl Into<Name>, message: impl Into<Name>) -> Self {
        Self {
            window: UiWindow::new((), 0.25),
            offset: vec2::ZERO,
            state: WidgetState::new(),
            title: TextWidget::new(title),
            message: TextWidget::new(message),
            confirm: IconButtonWidget::new_normal(assets.atlas.confirm()),
            discard: IconButtonWidget::new_normal(assets.atlas.discard()),
        }
    }
}

impl WidgetOld for ConfirmWidget {
    fn state_mut(&mut self) -> &mut WidgetState {
        &mut self.state
    }

    fn update(&mut self, position: Aabb2<f32>, context: &mut UiContext) {
        self.window.update(context.delta_time);

        let position = position.translate(self.offset);
        self.state.update(position, context);

        let mut main = position;

        let title = main.cut_top(1.5 * context.font_size);
        self.title.update(title, context);
        if self.title.state.pressed {
            // Drag window
            self.offset += context.cursor.delta();
        }

        let buttons = main.cut_bottom(1.2 * context.font_size);
        let buttons = buttons.split_columns(2);
        self.confirm.update(buttons[0], context);
        self.discard.update(buttons[1], context);

        self.message.update(main, context);
    }
}

impl Widget for ConfirmWidget {
    fn state_mut(&mut self) -> &mut WidgetState {
        &mut self.state
    }

    fn draw(&self, context: &UiContext) -> Geometry {
        let outline_width = context.font_size * 0.1;
        let theme = context.theme();

        let window = self.state.position;
        let min_height = outline_width * 10.0;
        let t = crate::util::smoothstep(self.window.show.time.get_ratio());
        let height = (t * window.height()).max(min_height);

        let mut in_window = Geometry::new();
        let title = self.title.state.position;
        in_window.merge(
            context
                .geometry
                .quad_fill(title, outline_width, theme.light),
        );
        in_window.merge(self.title.draw_colored(context, theme.dark));
        in_window.merge(self.message.draw(context));
        in_window.merge(self.confirm.icon.draw(context));
        in_window.merge(self.discard.icon.draw(context));

        let window = window.with_height(height, 1.0);
        let mut geometry = Geometry::new();
        geometry.merge(
            context
                .geometry
                .quad_outline(window, outline_width, theme.light),
        );
        geometry.merge(
            context
                .geometry
                .quad_fill(window, outline_width, theme.dark),
        );
        geometry.merge(context.geometry.masked(window, in_window));
        geometry
    }
}
