use super::*;

use crate::{
    prelude::{Assets, Index},
    ui::layout::AreaOps,
};

use ctl_client::core::types::Name;

#[derive(Debug)]
pub enum ConfirmAction {
    DeleteGroup(Index),
    SyncDiscard,
    ExitUnsaved,
}

#[derive(Debug)]
pub struct ConfirmPopup {
    pub action: ConfirmAction,
    pub message: Name,
}

pub struct ConfirmWidget {
    pub window: UiWindow<()>,
    pub offset: vec2<f32>,
    pub state: WidgetState,
    /// Position that can be dragged to move the widget.
    pub hold: WidgetState,
    pub title: TextWidget,
    pub message: TextWidget,
    pub confirm: IconButtonWidget,
    pub discard: IconButtonWidget,
}

impl ConfirmWidget {
    pub fn new(assets: &Rc<Assets>, message: impl Into<Name>) -> Self {
        Self {
            window: UiWindow::new((), 0.25),
            offset: vec2::ZERO,
            state: WidgetState::new(),
            hold: WidgetState::new(),
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
        self.window.update(context.delta_time);

        let position = position.translate(self.offset);
        self.state.update(position, context);

        let mut main = position;

        let hold = main.cut_top(context.layout_size);
        let hold = hold.extend_symmetric(-vec2(context.layout_size * 2.0, 0.0));
        self.hold.update(hold, context);
        if self.hold.pressed {
            // Drag window
            self.offset += context.cursor.delta();
        }

        let title = main.cut_top(context.font_size);
        self.title.update(title, context);

        let buttons = main.cut_bottom(context.font_size);
        let buttons = buttons.split_columns(2);
        self.confirm.update(buttons[0], context);
        self.discard.update(buttons[1], context);

        self.message.update(main, context);
    }
}
