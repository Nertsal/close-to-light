mod beat_time;
mod button;
mod confirm;
mod dropdown;
mod explore;
mod icon;
mod input;
mod leaderboard;
mod notification;
mod options;
mod profile;
mod slider;
mod sync;
mod text;
mod value;

pub use self::{
    beat_time::*, button::*, confirm::*, dropdown::*, explore::*, icon::*, input::*,
    leaderboard::*, notification::*, options::*, profile::*, slider::*, sync::*, text::*, value::*,
};

use super::{WidgetId, context::*, geometry::Geometry, window::*};

use crate::simple_widget_state;

use std::any::Any;

use geng::prelude::*;

#[macro_export]
macro_rules! simple_widget_state {
    () => {
        fn state_mut(&mut self) -> &mut WidgetState {
            &mut self.state
        }
    };
    ($path:tt) => {
        fn state_mut(&mut self) -> &mut WidgetState {
            &mut self.$path.state
        }
    };
}

pub trait Widget: WidgetToAny {
    fn state_mut(&mut self) -> &mut WidgetState;
    #[must_use]
    fn draw_top(&self, context: &UiContext) -> Geometry {
        #![allow(unused_variables)]
        Geometry::new()
    }
    #[must_use]
    fn draw(&self, context: &UiContext) -> Geometry;
}

#[doc(hidden)]
pub trait WidgetToAny {
    fn to_any_mut(&mut self) -> &mut dyn Any;
}

impl<T: Any> WidgetToAny for T {
    fn to_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

pub trait WidgetOld {
    /// Update position and related properties.
    fn update(&mut self, position: Aabb2<f32>, context: &mut UiContext);
    /// Get a mutable reference to the root state.
    fn state_mut(&mut self) -> &mut WidgetState;
    /// Make the widget visible.
    fn show(&mut self) {
        self.state_mut().show()
    }
    /// Hide the widget and disable interactions.
    fn hide(&mut self) {
        self.state_mut().hide()
    }
}

pub trait StatefulWidget {
    /// The external state that the widget operates on and can modify.
    type State<'a>;

    /// Update position and related properties.
    fn update(
        &mut self,
        position: Aabb2<f32>,
        context: &mut UiContext,
        state: &mut Self::State<'_>,
    );
    /// Get a mutable reference to the root state.
    fn state_mut(&mut self) -> &mut WidgetState;
    /// Make the widget visible.
    fn show(&mut self) {
        self.state_mut().show()
    }
    /// Hide the widget and disable interactions.
    fn hide(&mut self) {
        self.state_mut().hide()
    }
}

#[derive(Debug, Clone)]
pub struct WidgetState {
    pub id: WidgetId,
    pub position: Aabb2<f32>,
    /// Whether to show the widget.
    pub visible: bool,
    pub hovered: bool,
    /// Whether user has left clicked on the widget since last frame.
    pub clicked: bool,
    /// Whether user is holding the left mouse button down on the widget.
    pub pressed: bool,
    /// Whether user has released the left mouse button since last frame.
    pub released: bool,
    /// Whether user has right clicked on the widget since last frame.
    pub right_clicked: bool,
    /// Whether user is holding the right mouse button down on the widget.
    pub right_pressed: bool,
}

impl WidgetState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update(&mut self, position: Aabb2<f32>, context: &UiContext) {
        self.position = position;
        if self.visible && context.can_focus() {
            self.hovered = self.position.contains(context.cursor.position);
            let was_pressed = self.pressed;
            // TODO: check for mouse being pressed and then dragged onto the widget
            self.pressed =
                context.cursor.down && (was_pressed || self.hovered && !context.cursor.was_down);
            self.clicked = !was_pressed && self.pressed;
            self.released = was_pressed && !self.pressed;

            let was_pressed = self.right_pressed;
            self.right_pressed = context.cursor.right_down
                && (was_pressed || self.hovered && !context.cursor.was_right_down);
            self.right_clicked = !was_pressed && self.right_pressed;
        } else {
            self.released = self.pressed;

            self.hovered = false;
            self.pressed = false;
            self.clicked = false;
            self.right_pressed = false;
            self.right_clicked = false;
        }
    }

    pub fn show(&mut self) {
        self.visible = true;
    }

    pub fn hide(&mut self) {
        self.visible = false;
        self.hovered = false;
        self.pressed = false;
        self.clicked = false;
        self.released = false;
        self.right_pressed = false;
        self.right_clicked = false;
    }
}

impl Default for WidgetState {
    fn default() -> Self {
        Self {
            id: WidgetId::default(),
            position: Aabb2::ZERO.extend_uniform(1.0),
            visible: true,
            hovered: false,
            clicked: false,
            pressed: false,
            released: false,
            right_clicked: false,
            right_pressed: false,
        }
    }
}
