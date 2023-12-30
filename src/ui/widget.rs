mod button;
mod checkbox;
mod group;
mod leaderboard;
mod level;
mod level_config;
mod light;
mod text;
mod timeline;

pub use self::{
    button::ButtonWidget,
    checkbox::CheckboxWidget,
    group::GroupWidget,
    leaderboard::{LeaderboardEntryWidget, LeaderboardWidget},
    level::*,
    level_config::{LevelConfigWidget, LevelDifficultyWidget, LevelModsWidget},
    light::{LightStateWidget, LightWidget},
    text::TextWidget,
    timeline::TimelineWidget,
};

use geng::prelude::*;

#[derive(Debug, Clone, Copy)]
pub struct CursorContext {
    pub position: vec2<f32>,
    pub down: bool,
    /// Was the cursor down last frame.
    pub was_down: bool,
}

impl CursorContext {
    pub fn new() -> Self {
        Self {
            position: vec2::ZERO,
            down: false,
            was_down: false,
        }
    }

    pub fn update(&mut self, is_down: bool) {
        self.was_down = self.down;
        self.down = is_down;
    }
}

#[derive(Debug, Clone, Copy)]
pub struct UiContext {
    pub font_size: f32,
    /// Whether the widget can use the cursor position to get focus.
    pub can_focus: bool,
    pub cursor: CursorContext,
}

impl UiContext {
    pub fn scale_font(self, scale: f32) -> Self {
        Self {
            font_size: self.font_size * scale,
            ..self
        }
    }

    /// Update `can_focus` property given another widget's focus.
    pub fn update_focus(&mut self, focus: bool) {
        self.can_focus = self.can_focus && !focus;
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
        if self.visible && context.can_focus {
            self.hovered = self.position.contains(context.cursor.position);
            let was_pressed = self.pressed;
            // TODO: check for mouse being pressed and then dragged onto the widget
            self.pressed =
                context.cursor.down && (was_pressed || self.hovered && !context.cursor.was_down);
            self.clicked = !was_pressed && self.pressed;
        } else {
            self.hovered = false;
            self.pressed = false;
            self.clicked = false;
        }
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
