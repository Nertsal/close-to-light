use super::*;

use crate::{prelude::Assets, ui::layout::AreaOps};

use ctl_client::core::types::Name;

pub struct ConfirmWidget {
    pub state: WidgetState,
    pub title: TextWidget,
    pub message: TextWidget,
    pub confirm: IconButtonWidget,
    pub discard: IconButtonWidget,
}

impl ConfirmWidget {
    pub fn new(assets: &Rc<Assets>, message: impl Into<Name>) -> Self {
        Self {
            state: WidgetState::new(),
            title: TextWidget::new("Are you sure?"),
            message: TextWidget::new(message),
            confirm: IconButtonWidget::new_normal(&assets.sprites.confirm),
            discard: IconButtonWidget::new_normal(&assets.sprites.discard),
        }
    }
}

impl Widget for ConfirmWidget {
    fn state_mut(&mut self) -> &mut WidgetState {
        &mut self.state
    }

    fn update(&mut self, position: Aabb2<f32>, context: &mut UiContext) {
        self.state.update(position, context);

        let mut main = position;
        let title = main.cut_top(context.font_size);
        self.title.update(title, context);

        let buttons = main.cut_bottom(context.font_size);
        let buttons = buttons.split_columns(2);
        self.confirm.update(buttons[0], context);
        self.discard.update(buttons[1], context);

        self.message.update(main, context);
    }
}
