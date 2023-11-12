mod button;
mod checkbox;
mod group;
mod level;
mod light;
mod text;
mod timeline;

pub use self::{
    button::ButtonWidget,
    checkbox::CheckboxWidget,
    group::GroupWidget,
    level::PlayLevelWidget,
    light::{LightStateWidget, LightWidget},
    text::TextWidget,
    timeline::TimelineWidget,
};

use geng::prelude::*;

#[derive(Debug, Clone, Copy)]
pub struct UiContext {
    pub font_size: f32,
    pub cursor_position: vec2<f32>,
    pub cursor_down: bool,
}

impl UiContext {
    pub fn scale_font(self, scale: f32) -> Self {
        Self {
            font_size: self.font_size * scale,
            ..self
        }
    }
}

pub trait Widget {
    /// Update position and related properties.
    fn update(&mut self, position: Aabb2<f32>, context: &UiContext);
    /// Apply a function to all states contained in the widget.
    fn walk_states_mut(&mut self, f: &dyn Fn(&mut WidgetState));
    /// Make the widget visible.
    fn show(&mut self) {
        self.walk_states_mut(&WidgetState::show)
    }
    /// Hide the widget and disable interactions.
    fn hide(&mut self) {
        self.walk_states_mut(&WidgetState::hide)
    }
}

#[derive(Debug, Clone)]
pub struct WidgetState {
    pub position: Aabb2<f32>,
    /// Whether to show the widget.
    pub visible: bool,
    pub hovered: bool,
    /// Whether user has clicked on the widget since last frame.
    pub clicked: bool,
    /// Whether user is holding the mouse button down on the widget.
    pub pressed: bool,
}

impl WidgetState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update(&mut self, position: Aabb2<f32>, context: &UiContext) {
        self.position = position;
        self.hovered = self.position.contains(context.cursor_position);
        let was_pressed = self.pressed;
        // TODO: check for mouse being pressed and then dragged onto the widget
        self.pressed = context.cursor_down && (was_pressed || self.hovered);
        self.clicked = !was_pressed && self.pressed;
    }

    /// For compatibility with [Widget::walk_states_mut].
    pub fn walk_states_mut(&mut self, f: &dyn Fn(&mut WidgetState)) {
        f(self)
    }

    pub fn show(&mut self) {
        self.visible = true;
    }

    pub fn hide(&mut self) {
        self.visible = false;
        self.pressed = false;
        self.clicked = false;
    }
}

impl Default for WidgetState {
    fn default() -> Self {
        Self {
            position: Aabb2::ZERO.extend_uniform(1.0),
            visible: true,
            hovered: false,
            clicked: false,
            pressed: false,
        }
    }
}
