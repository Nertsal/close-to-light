use super::*;

use crate::layout::AreaOps;

use ctl_assets::Assets;
use ctl_core::types::Name;

#[derive(Debug)]
pub struct ConfirmPopup<T> {
    pub action: T,
    pub title: Name,
    pub message: Name,
    pub confirm_text: Name,
    pub discard_text: Name,
}

pub struct ConfirmWidget {
    pub window: UiWindow<()>,
    pub offset: vec2<f32>,
    pub state: WidgetState,
    pub title: TextWidget,
    pub message: TextWidget,
    pub confirm_icon: IconButtonWidget,
    pub discard_icon: IconButtonWidget,
    pub confirm_text: ButtonWidget,
    pub discard_text: ButtonWidget,
}

impl ConfirmWidget {
    pub fn new(
        assets: &Rc<Assets>,
        title: impl Into<Name>,
        message: impl Into<Name>,
        confirm_text: impl Into<Name>,
        discard_text: impl Into<Name>,
    ) -> Self {
        Self {
            window: UiWindow::new((), 0.25),
            offset: vec2::ZERO,
            state: WidgetState::new(),
            title: TextWidget::new(title),
            message: TextWidget::new(message),
            confirm_icon: IconButtonWidget::new_normal(assets.atlas.confirm()),
            discard_icon: IconButtonWidget::new_normal(assets.atlas.discard()),
            confirm_text: ButtonWidget::new(confirm_text.into()),
            discard_text: ButtonWidget::new(discard_text.into()),
        }
    }
}

impl WidgetOld for ConfirmWidget {
    fn state_mut(&mut self) -> &mut WidgetState {
        &mut self.state
    }

    fn update(&mut self, position: Aabb2<f32>, context: &mut UiContext) {
        self.window.update(context.delta_time);

        if self.confirm_text.text.text.is_empty() {
            self.confirm_text.hide();
            self.confirm_icon.show();
        } else {
            self.confirm_text.show();
            self.confirm_icon.hide();
        }
        if self.discard_text.text.text.is_empty() {
            self.discard_text.hide();
            self.discard_icon.show();
        } else {
            self.discard_text.show();
            self.discard_icon.hide();
        }

        let position = position.translate(self.offset);
        self.state.update(position, context);

        let mut main = position;

        let title = main.cut_top(1.5 * context.font_size);
        self.title.update(title, context);
        if self.title.state.mouse_left.pressed.is_some() {
            // Drag window
            self.offset += context.cursor.delta();
        }

        let buttons = main.cut_bottom(1.2 * context.font_size);
        let buttons = buttons.split_columns(2);
        self.confirm_icon.update(buttons[0], context);
        self.confirm_text.update(buttons[0], context);
        self.discard_icon.update(buttons[1], context);
        self.discard_text.update(buttons[1], context);

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
        let t = ctl_util::smoothstep(self.window.show.time.get_ratio());
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
        in_window.merge(self.confirm_icon.draw(context));
        in_window.merge(self.discard_icon.draw(context));
        in_window.merge(self.confirm_text.draw(context));
        in_window.merge(self.discard_text.draw(context));

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
