use super::*;

use crate::layout::AreaOps;

use ctl_core::types::Name;

pub struct DropdownValueWidget<T> {
    pub state: WidgetState,
    pub name: TextWidget,
    pub value: usize,
    pub dropdown: DropdownWidget<T>,
}

pub struct DropdownWidget<T> {
    pub options: Vec<(Name, T)>,
    pub text: TextWidget,
    pub dropdown_state: WidgetState,
    pub dropdown_window: UiWindow<()>,
    pub dropdown_items: Vec<TextWidget>,
}

impl<T> DropdownWidget<T> {
    pub fn new(
        text: impl Into<Name>,
        options: impl IntoIterator<Item = (impl Into<Name>, T)>,
    ) -> Self {
        let options: Vec<_> = options
            .into_iter()
            .map(|(name, t)| (name.into(), t))
            .collect();
        Self {
            text: TextWidget::new(text),
            dropdown_state: WidgetState::new(),
            dropdown_window: UiWindow::new((), 0.2),
            dropdown_items: options
                .iter()
                .map(|(name, _)| {
                    let mut text = TextWidget::new(name.clone());
                    text.state.sfx_config = WidgetSfxConfig::hover_left();
                    text
                })
                .collect(),
            options,
        }
    }

    pub fn update(&mut self, position: Aabb2<f32>, context: &UiContext) -> Option<usize> {
        self.text.update(position, context);

        let _l = context.state.start_layer();

        let could_focus_before = context.can_focus();
        let focus =
            self.dropdown_window.show.time.is_max() && context.total_focus(self.dropdown_state.id);
        if focus {
            context.focus_begin();
        } else {
            // TODO: better more general solution
            // Block focus for dropdown options
            context.update_focus(true);
        }

        // TODO: limit height and allow scroll
        let item_height = context.font_size;
        let spacing = context.layout_size * 0.5;
        let dropdown_height = (item_height + spacing) * self.dropdown_items.len() as f32 - spacing;
        let floor = (position.max.y - dropdown_height).max(context.screen.min.y);
        let dropdown = Aabb2 {
            min: vec2(position.min.x, floor),
            max: vec2(position.max.x, floor + dropdown_height),
        };
        self.dropdown_state.update(dropdown, context);
        self.dropdown_state.z_index = context.state.current_layer();

        let mut just_selected = None;
        let can_select = focus && self.dropdown_state.hovered;
        let mut position = dropdown.clone().cut_top(item_height);
        for (i, item) in self.dropdown_items.iter_mut().enumerate() {
            item.update(position, context);
            if can_select && item.state.mouse_left.clicked {
                just_selected = Some(i);
                self.dropdown_window.request = Some(WidgetRequest::Close);
            }
            position = position.translate(vec2(0.0, -item_height - spacing));
        }
        if focus {
            context.focus_end();
        } else if could_focus_before {
            *context.can_focus.borrow_mut() = could_focus_before;
        }

        if self.dropdown_window.show.time.is_max() && !focus {
            self.dropdown_window.request = Some(WidgetRequest::Close);
        }

        if !focus && self.text.state.mouse_left.clicked {
            self.dropdown_window.request = Some(WidgetRequest::Open);
        }
        self.dropdown_window.update(context.delta_time);

        just_selected
    }
}

impl<T: 'static> Widget for DropdownWidget<T> {
    simple_widget_state!(text);
    fn draw(&self, context: &UiContext) -> Geometry {
        let outline_width = context.font_size * 0.1;
        let theme = context.theme();

        let mut fg_color = theme.light;
        if self.text.state.hovered {
            fg_color = theme.highlight;
        }

        let mut geometry = Geometry::new();

        let mut bounds = self.dropdown_state.position;
        let height = bounds.height() * self.dropdown_window.show.time.get_ratio();
        if height > outline_width * 2.0 {
            let bounds = bounds.cut_top(height);
            let mut window = Geometry::new();
            for text in &self.dropdown_items {
                let mut fg_color = theme.light;
                let mut bg_color = theme.dark;

                if text.state.hovered {
                    std::mem::swap(&mut fg_color, &mut bg_color);
                }

                window.merge(text.draw_colored(context, fg_color));

                let position = text.state.position;
                window.merge(if text.state.mouse_left.pressed.is_some() {
                    context.geometry.quad_fill(
                        position.extend_uniform(-outline_width * 0.5),
                        outline_width,
                        bg_color,
                    )
                } else {
                    context
                        .geometry
                        .quad_fill(position, outline_width, bg_color)
                });
                // window.merge(text.draw(context));
            }
            window.merge(
                context
                    .geometry
                    .quad_fill(bounds, outline_width, theme.dark),
            );
            geometry.merge(context.geometry.masked(bounds, window));
            geometry.merge(
                context
                    .geometry
                    .quad_outline(bounds, outline_width, theme.light),
            );
            geometry.change_z_index(10);
        } else {
            geometry.merge(self.text.draw_colored(context, fg_color));
        }

        geometry
    }
}

impl<T: PartialEq + Clone> DropdownValueWidget<T> {
    pub fn new(
        text: impl Into<Name>,
        value: usize,
        options: impl IntoIterator<Item = (impl Into<Name>, T)>,
    ) -> Self {
        Self {
            state: WidgetState::new().with_sfx(WidgetSfxConfig::hover_left()),
            name: TextWidget::new(text).aligned(vec2(0.0, 0.5)),
            value,
            dropdown: DropdownWidget::new("<value>", options),
        }
    }

    pub fn update(&mut self, position: Aabb2<f32>, context: &UiContext, state: &mut T) {
        self.value = self
            .dropdown
            .options
            .iter()
            .position(|(_, t)| t == state)
            .unwrap_or(0); // TODO: maybe do smth with the error
        let state_position = position;
        let mut main = position;

        let name = main.split_left(if self.name.text.is_empty() { 0.0 } else { 0.5 });
        let value = main;

        self.state.update(state_position, context);
        self.name.update(name, context);
        if let Some(i) = self.dropdown.update(value, context) {
            self.value = i;
            if let Some((_, value)) = self.dropdown.options.get(i) {
                *state = value.clone();
            }
        }

        if let Some((name, _)) = self.dropdown.options.get(self.value) {
            self.dropdown.text.text = name.clone();
        }
    }
}

impl<T: 'static> Widget for DropdownValueWidget<T> {
    simple_widget_state!();
    fn draw(&self, context: &UiContext) -> Geometry {
        let theme = context.theme();

        let mut fg_color = theme.light;
        if self.state.hovered {
            fg_color = theme.highlight;
        }

        let mut geometry = self.name.draw_colored(context, fg_color);
        geometry.merge(self.dropdown.draw(context));

        geometry
    }
}

impl<T: PartialEq + Clone> StatefulWidget for DropdownValueWidget<T> {
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
        self.update(position, context, state)
    }
}
