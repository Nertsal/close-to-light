use super::*;

use crate::ui::layout::AreaOps;

use ctl_client::core::types::Name;

pub struct DropdownWidget<T> {
    pub state: WidgetState,
    pub name: TextWidget,
    pub value_text: TextWidget,
    pub value: usize,
    pub options: Vec<(Name, T)>,

    pub dropdown_state: WidgetState,
    pub dropdown_window: UiWindow<()>,
    pub dropdown_items: Vec<TextWidget>,
}

impl<T> DropdownWidget<T> {
    pub fn new(
        text: impl Into<Name>,
        value: usize,
        options: impl IntoIterator<Item = (impl Into<Name>, T)>,
    ) -> Self {
        let options: Vec<_> = options
            .into_iter()
            .map(|(name, t)| (name.into(), t))
            .collect();
        Self {
            state: WidgetState::new(),
            name: TextWidget::new(text),
            value_text: TextWidget::new("<value>"),
            value,
            dropdown_state: WidgetState::new(),
            dropdown_window: UiWindow::new((), 0.2),
            dropdown_items: options
                .iter()
                .map(|(name, _)| TextWidget::new(name.clone()))
                .collect(),
            options,
        }
    }
}

impl<T: PartialEq + Clone> StatefulWidget for DropdownWidget<T> {
    type State<'a> = T;

    fn state_mut(&mut self) -> &mut WidgetState {
        &mut self.state
    }

    fn update(
        &mut self,
        position: Aabb2<f32>,
        context: &mut UiContext,
        state: &mut Self::State<'_>,
    ) {
        self.value = self
            .options
            .iter()
            .position(|(_, t)| t == state)
            .unwrap_or(0); // TODO: maybe do smth with the error
        self.state.update(position, context);
        let mut main = position;

        let name = main.split_left(0.5);
        let value = main;

        // TODO: limit height and allow scroll
        let item_height = context.font_size;
        let spacing = context.layout_size * 0.5;
        let dropdown_height = (item_height + spacing) * self.options.len() as f32;
        let floor = (value.max.y - dropdown_height).max(context.screen.min.y);
        let dropdown = Aabb2 {
            min: vec2(value.min.x, floor),
            max: vec2(value.max.x, floor + dropdown_height),
        };
        self.dropdown_state.update(dropdown, context);

        let focus = context.can_focus();
        let can_select =
            focus && self.dropdown_window.show.time.is_max() && self.dropdown_state.hovered;

        let mut position = dropdown.clone().cut_top(item_height);
        for (i, item) in self.dropdown_items.iter_mut().enumerate() {
            item.update(position, context);
            if can_select && item.state.clicked {
                self.value = i;
                if let Some((_, value)) = self.options.get(i) {
                    *state = value.clone();
                }
                self.dropdown_window.request = Some(WidgetRequest::Close);
            }
            position = position.translate(vec2(0.0, -item_height - spacing));
        }

        if can_select {
            context.update_focus(true);
        }

        self.name.update(name, context);
        self.value_text.update(value, context);

        if let Some((name, _)) = self.options.get(self.value) {
            self.value_text.text = name.clone();
        }

        if self.value_text.state.clicked {
            self.dropdown_window.request = Some(WidgetRequest::Open);
        }
        self.dropdown_window.update(context.delta_time);

        if focus {
            context.reset_focus();
        }
    }
}
