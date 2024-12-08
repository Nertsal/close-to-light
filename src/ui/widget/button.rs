use super::*;

use crate::model::ThemeColor;

use ctl_client::core::types::Name;

#[derive(Clone, Default)]
pub struct ButtonWidget {
    pub text: TextWidget,
}

impl ButtonWidget {
    pub fn new(text: impl Into<Name>) -> Self {
        Self {
            text: TextWidget::new(text),
        }
    }

    pub fn update(&mut self, position: Aabb2<f32>, context: &UiContext) {
        self.text.update(position, context);
        self.text.options.color = context.theme().dark;
    }
}

impl WidgetOld for ButtonWidget {
    fn update(&mut self, position: Aabb2<f32>, context: &mut UiContext) {
        self.text.update(position, context);
    }

    fn state_mut(&mut self) -> &mut WidgetState {
        &mut self.text.state
    }
}

impl Widget for ButtonWidget {
    fn draw(&self, context: &UiContext) -> Geometry {
        let theme = context.theme();
        let state = &self.text.state;
        let width = self.text.options.size * 0.2;

        let position = state.position;
        let mut geometry = if state.pressed {
            context
                .geometry
                .quad_fill(position.extend_uniform(-width * 1.5), theme.light)
        } else if state.hovered {
            context
                .geometry
                .quad_fill(position.extend_uniform(-width), theme.light)
        } else {
            context.geometry.quad_fill(position, theme.light)
        };

        geometry.merge(self.text.draw(context));
        geometry
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
            color: ThemeColor::Dark,
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

impl WidgetOld for IconButtonWidget {
    fn state_mut(&mut self) -> &mut WidgetState {
        &mut self.state
    }

    fn update(&mut self, position: Aabb2<f32>, context: &mut UiContext) {
        self.state.update(position, context);
        self.icon.update(position, context);

        let mut light = self.light_color;
        let mut dark = ThemeColor::Dark;
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
    pub fn new(text: impl Into<Name>) -> Self {
        Self {
            text: TextWidget::new(text),
            selected: false,
            can_deselect: false,
        }
    }

    pub fn new_deselectable(text: impl Into<Name>) -> Self {
        Self {
            text: TextWidget::new(text),
            selected: false,
            can_deselect: true,
        }
    }
}

impl WidgetOld for ToggleWidget {
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
