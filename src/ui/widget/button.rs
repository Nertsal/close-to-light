use crate::prelude::ThemeColor;

use super::*;

#[derive(Clone, Default)]
pub struct ButtonWidget {
    pub text: TextWidget,
}

impl ButtonWidget {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: TextWidget::new(text),
        }
    }
}

impl Widget for ButtonWidget {
    fn state_mut(&mut self) -> &mut WidgetState {
        &mut self.text.state
    }

    fn update(&mut self, position: Aabb2<f32>, context: &mut UiContext) {
        self.text.update(position, context);
    }
}

#[derive(Clone)]
pub struct IconButtonWidget {
    pub state: WidgetState,
    pub icon: IconWidget,
    pub light_color: ThemeColor,
}

impl IconButtonWidget {
    pub fn new(
        texture: &Rc<ugli::Texture>,
        light_color: ThemeColor,
        bg_kind: IconBackgroundKind,
    ) -> Self {
        let mut icon = IconWidget::new(texture);
        icon.background = Some(IconBackground {
            color: Rgba::BLACK,
            kind: bg_kind,
        });
        Self {
            state: WidgetState::new(),
            icon,
            light_color,
        }
    }

    pub fn new_normal(texture: &Rc<ugli::Texture>) -> Self {
        Self::new(texture, ThemeColor::Light, IconBackgroundKind::NineSlice)
    }

    pub fn new_danger(texture: &Rc<ugli::Texture>) -> Self {
        Self::new(texture, ThemeColor::Danger, IconBackgroundKind::NineSlice)
    }

    pub fn new_close_button(texture: &Rc<ugli::Texture>) -> Self {
        Self::new(texture, ThemeColor::Danger, IconBackgroundKind::Circle)
    }
}

impl Widget for IconButtonWidget {
    fn state_mut(&mut self) -> &mut WidgetState {
        &mut self.state
    }

    fn update(&mut self, position: Aabb2<f32>, context: &mut UiContext) {
        self.state.update(position, context);
        self.icon.update(position, context);

        let mut light = context.theme.get_color(self.light_color);
        let mut dark = context.theme.dark;
        if self.state.hovered {
            std::mem::swap(&mut dark, &mut light);
        }

        self.icon.color = light;
        if let Some(bg) = &mut self.icon.background {
            bg.color = dark;
        }
    }
}

#[derive(Clone, Default)]
pub struct ToggleWidget {
    pub text: TextWidget,
    pub selected: bool,
    pub can_deselect: bool,
}

impl ToggleWidget {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: TextWidget::new(text),
            selected: false,
            can_deselect: false,
        }
    }

    pub fn new_deselectable(text: impl Into<String>) -> Self {
        Self {
            text: TextWidget::new(text),
            selected: false,
            can_deselect: true,
        }
    }
}

impl Widget for ToggleWidget {
    fn state_mut(&mut self) -> &mut WidgetState {
        &mut self.text.state
    }

    fn update(&mut self, position: Aabb2<f32>, context: &mut UiContext) {
        self.text.update(position, context);
        if self.text.state.clicked {
            if self.can_deselect {
                self.selected = !self.selected;
            } else {
                self.selected = true;
            }
        }
    }
}
