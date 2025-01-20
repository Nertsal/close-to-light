use super::*;

use crate::{prelude::Assets, ui::layout::AreaOps, util::Lerp};

use ctl_client::core::types::Name;

pub struct NotificationsWidget {
    pub assets: Rc<Assets>,
    pub state: WidgetState,
    pub discard_offset: Lerp<f32>,
    pub discard_all: TextWidget,
    pub items: Vec<NotificationWidget>,
    pub items_done: Vec<NotificationWidget>,
}

impl NotificationsWidget {
    pub fn new(assets: &Rc<Assets>) -> Self {
        Self {
            assets: assets.clone(),
            state: WidgetState::new(),
            discard_offset: Lerp::new_smooth(0.3, 0.0, 0.0),
            discard_all: TextWidget::new("discard all"),
            items: Vec::new(),
            items_done: Vec::new(),
        }
    }

    pub fn notify(&mut self, message: impl Into<Name>) {
        let message = message.into();
        log::debug!("Notification: {}", message);
        self.items
            .push(NotificationWidget::new(&self.assets, message));
    }
}

impl WidgetOld for NotificationsWidget {
    fn state_mut(&mut self) -> &mut WidgetState {
        &mut self.state
    }

    fn update(&mut self, position: Aabb2<f32>, context: &mut UiContext) {
        self.state.update(position, context);

        let discard_size = vec2(5.0, 1.2) * context.font_size;
        {
            let mut position = position.align_aabb(discard_size, vec2(1.0, 1.0));
            let offset = position.height();
            position = position.translate(vec2(0.0, offset + self.discard_offset.current()));
            if self.items.is_empty() {
                self.discard_offset.change_target(0.0);
            } else {
                self.discard_offset.change_target(-offset);
            }
            self.discard_offset.update(context.delta_time);
            self.discard_all.update(position, context);
        }

        let size = vec2(15.0, 7.0) * context.layout_size;
        let mut position = position.align_aabb(size, vec2(1.0, 1.0));
        position = position.translate(vec2(0.0, position.height()));
        let anchor = position;
        position = position.translate(vec2(0.0, -discard_size.y + context.layout_size));

        let mut done = Vec::new();
        for (i, notification) in self.items.iter_mut().enumerate() {
            position = position.translate(vec2(0.0, -position.height() - context.layout_size));
            notification.update(anchor, context);
            notification
                .offset_y
                .change_target(position.bottom_left().y - anchor.bottom_left().y);

            context.update_focus(notification.state.hovered);

            if notification.offset_y.time.is_max()
                && (notification.confirm.state.clicked || self.discard_all.state.clicked)
            {
                done.push(i);
            }
        }

        for i in done.into_iter().rev() {
            let item = self.items.remove(i);
            self.items_done.push(item);
        }

        for notification in &mut self.items_done {
            notification.offset_x.change_target(anchor.width() * 2.0);
            notification.offset_y.stop();
            notification.update(anchor, context);
        }
        self.items_done.retain(|item| !item.offset_x.time.is_max());
    }
}

pub struct NotificationWidget {
    pub offset_x: Lerp<f32>,
    pub offset_y: Lerp<f32>,
    pub state: WidgetState,
    pub text: TextWidget,
    pub confirm: IconButtonWidget,
}

impl NotificationWidget {
    pub fn new(assets: &Rc<Assets>, text: impl Into<Name>) -> Self {
        Self {
            offset_x: Lerp::new_smooth(0.3, 0.0, 0.0),
            offset_y: Lerp::new_smooth(0.3, 0.0, 0.0),
            state: WidgetState::new(),
            text: TextWidget::new(text).aligned(vec2(0.0, 0.5)),
            confirm: IconButtonWidget::new_normal(&assets.sprites.confirm),
        }
    }
}

impl WidgetOld for NotificationWidget {
    fn state_mut(&mut self) -> &mut WidgetState {
        &mut self.state
    }

    fn update(&mut self, position: Aabb2<f32>, context: &mut UiContext) {
        self.offset_x.update(context.delta_time);
        self.offset_y.update(context.delta_time);

        let size = position.size();
        let pos = position.bottom_left() + vec2(self.offset_x.current(), self.offset_y.current());
        let position = Aabb2::point(pos).extend_positive(size);

        self.state.update(position, context);

        let mut main = position;
        let confirm = main.cut_right(context.layout_size * 2.0);
        self.confirm.update(confirm, context);

        let text = main.extend_uniform(-context.layout_size * 0.5);
        self.text.update(text, &context.scale_font(0.7));
    }
}
