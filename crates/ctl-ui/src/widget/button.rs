use super::*;

use crate::layout::AreaOps;

use ctl_assets::ThemeColor;
use ctl_core::types::Name;
use ctl_render_core::SubTexture;

#[derive(Clone)]
pub struct ButtonWidget {
    pub text: TextWidget,
    pub bg_color: ThemeColor,
}

impl ButtonWidget {
    pub fn new(text: impl Into<Name>) -> Self {
        let mut text = TextWidget::new(text);
        text.state.sfx_config = WidgetSfxConfig::hover_left();
        Self {
            text,
            bg_color: ThemeColor::Light,
        }
    }

    pub fn color(mut self, bg_color: ThemeColor) -> Self {
        self.bg_color = bg_color;
        self
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
    simple_widget_state!(text);
    fn draw(&self, context: &UiContext) -> Geometry {
        if !self.text.state.visible {
            return Geometry::new();
        }

        let theme = context.theme();
        let state = &self.text.state;
        let width = self.text.options.size * 0.2;

        let mut geometry = self.text.draw(context);

        let position = state.position;
        let bg_color = theme.get_color(self.bg_color);
        geometry.merge(if state.mouse_left.pressed.is_some() {
            context
                .geometry
                .quad_fill(position.extend_uniform(-width), width, bg_color)
        } else if state.hovered {
            context
                .geometry
                .quad_fill(position.extend_uniform(-width * 0.5), width, bg_color)
        } else {
            context.geometry.quad_fill(position, width, bg_color)
        });

        geometry
    }
}

#[derive(Clone)]
pub struct IconButtonWidget {
    pub icon: IconWidget,
    pub light_color: ThemeColor,
}

impl IconButtonWidget {
    pub fn new(texture: SubTexture, light_color: ThemeColor, bg_kind: IconBackgroundKind) -> Self {
        let mut icon = IconWidget::new(texture);
        icon.state.sfx_config = WidgetSfxConfig::hover_left();
        icon.background = Some(IconBackground {
            color: ThemeColor::Dark,
            kind: bg_kind,
        });
        Self { icon, light_color }
    }

    pub fn new_normal(texture: SubTexture) -> Self {
        Self::new(texture, ThemeColor::Light, IconBackgroundKind::NineSlice)
    }

    pub fn new_danger(texture: SubTexture) -> Self {
        Self::new(texture, ThemeColor::Danger, IconBackgroundKind::NineSlice)
    }

    pub fn new_close_button(texture: SubTexture) -> Self {
        Self::new(texture, ThemeColor::Danger, IconBackgroundKind::Circle)
    }

    pub fn update(&mut self, position: Aabb2<f32>, context: &UiContext) {
        self.icon.update(position, context);

        let mut light = self.light_color;
        let mut dark = ThemeColor::Dark;
        if self.icon.state.hovered {
            std::mem::swap(&mut dark, &mut light);
        }

        self.icon.color = light;
        if let Some(bg) = &mut self.icon.background {
            bg.color = dark;
        }
    }
}

impl WidgetOld for IconButtonWidget {
    fn state_mut(&mut self) -> &mut WidgetState {
        &mut self.icon.state
    }

    fn update(&mut self, position: Aabb2<f32>, context: &mut UiContext) {
        self.update(position, context)
    }
}

impl Widget for IconButtonWidget {
    simple_widget_state!(icon);
    fn draw(&self, context: &UiContext) -> Geometry {
        if !self.icon.state.visible {
            return Geometry::new();
        }
        self.icon.draw(context)
    }
}

#[derive(Clone, Default)]
pub struct ToggleButtonWidget {
    pub state: WidgetState,
    pub text: TextWidget,
    pub icon: Option<IconWidget>,
    pub selected: bool,
    pub can_deselect: bool,
}

impl ToggleButtonWidget {
    pub fn new(text: impl Into<Name>) -> Self {
        Self {
            state: WidgetState::new().with_sfx(WidgetSfxConfig::hover_left()),
            text: TextWidget::new(text),
            icon: None,
            selected: false,
            can_deselect: false,
        }
    }

    pub fn new_deselectable(text: impl Into<Name>) -> Self {
        Self {
            state: WidgetState::new().with_sfx(WidgetSfxConfig::hover_left()),
            text: TextWidget::new(text),
            icon: None,
            selected: false,
            can_deselect: true,
        }
    }

    pub fn with_icon(self, texture: SubTexture) -> Self {
        let mut icon = IconWidget::new(texture);
        icon.background = Some(IconBackground {
            color: ThemeColor::Dark,
            kind: IconBackgroundKind::NineSlice,
        });
        Self {
            icon: Some(icon),
            ..self
        }
    }

    pub fn update(&mut self, position: Aabb2<f32>, context: &UiContext) {
        self.state.update(position, context);
        self.text.update(position, context);
        if self.text.state.mouse_left.clicked {
            if self.can_deselect {
                self.selected = !self.selected;
            } else {
                self.selected = true;
            }
        }
        if let Some(icon) = &mut self.icon {
            icon.update(position, context);

            let mut light = ThemeColor::Light;
            let mut dark = ThemeColor::Dark;
            if self.selected || self.text.state.hovered {
                std::mem::swap(&mut dark, &mut light);
            }

            icon.color = light;
            if let Some(bg) = &mut icon.background {
                bg.color = dark;
            }
        }
    }
}

impl WidgetOld for ToggleButtonWidget {
    fn state_mut(&mut self) -> &mut WidgetState {
        &mut self.state
    }

    fn update(&mut self, position: Aabb2<f32>, context: &mut UiContext) {
        self.update(position, context);
    }
}

impl Widget for ToggleButtonWidget {
    simple_widget_state!();
    fn draw(&self, context: &UiContext) -> Geometry {
        let theme = context.theme();
        let width = self.text.options.size * 0.2;
        let (bg_color, fg_color) = if self.selected {
            (theme.light, theme.dark)
        } else {
            (theme.dark, theme.light)
        };

        let mut geometry = context
            .geometry
            .quad_fill(self.text.state.position, width, bg_color);
        geometry.merge(self.text.draw_colored(context, fg_color));
        geometry.merge(
            context
                .geometry
                .quad_outline(self.text.state.position, width, theme.light),
        );
        geometry
    }
}

pub struct ToggleWidget {
    pub state: WidgetState,
    pub text: TextWidget,
    pub tick: WidgetState,
    pub checked: bool,
    pub checked_color: ThemeColor,
}

impl ToggleWidget {
    pub fn new(text: impl Into<Name>) -> Self {
        Self {
            state: WidgetState::new().with_sfx(WidgetSfxConfig::hover_left()),
            text: TextWidget::new(text).aligned(vec2(0.0, 0.5)),
            tick: WidgetState::new(),
            checked: false,
            checked_color: ThemeColor::Highlight,
        }
    }

    pub fn color(mut self, select_color: ThemeColor) -> Self {
        self.checked_color = select_color;
        self
    }

    pub fn update_state(&mut self, position: Aabb2<f32>, context: &UiContext, state: &mut bool) {
        self.update(position, context);
        if self.state.mouse_left.clicked {
            *state = !*state;
        }
        self.checked = *state;
    }

    pub fn update(&mut self, position: Aabb2<f32>, context: &UiContext) {
        let mut main = position;
        self.state.update(main, context);
        let size = main.height() * 3.0 / 5.0;
        let tick = main.cut_right(size).with_height(size, 0.5);
        self.tick.update(tick, context);
        self.text.update(main, context);
    }
}

impl Widget for ToggleWidget {
    simple_widget_state!();
    fn draw(&self, context: &UiContext) -> Geometry {
        let theme = context.theme();
        let width = self.text.options.size * 0.1;

        let mut fg_color = theme.light;
        if self.state.hovered {
            fg_color = theme.get_color(self.checked_color);
        }

        let mut geometry = self.text.draw_colored(context, fg_color);
        geometry.merge(
            context
                .geometry
                .quad_outline(self.tick.position, width, fg_color),
        );
        if self.checked {
            geometry.merge(context.geometry.quad_fill(
                self.tick.position,
                width,
                theme.get_color(self.checked_color),
            ));
        }
        geometry
    }
}
