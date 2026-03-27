mod beat_time;
mod button;
mod color;
mod confirm;
mod dropdown;
mod explore;
mod icon;
mod input;
mod notification;
mod slider;
mod text;
mod value;

pub use self::{
    beat_time::*, button::*, color::*, confirm::*, dropdown::*, explore::*, icon::*, input::*,
    notification::*, slider::*, text::*, value::*,
};

use super::{WidgetId, context::*, geometry::Geometry, window::*};

use crate::simple_widget_state;

use std::any::Any;

use geng::prelude::*;

/// Max distance that the cursor can travel for a click to register as a stationary one.
const MAX_CLICK_DISTANCE: f32 = 10.0;

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
    pub mouse_left: WidgetMouseState,
    pub mouse_right: WidgetMouseState,
    pub sfx_config: WidgetSfxConfig,
}

#[derive(Default, Debug, Clone)]
pub struct WidgetMouseState {
    /// Whether user has pressed on the widget since last frame.
    pub just_pressed: bool,
    /// Whether user is holding the mouse button down on the widget.
    pub pressed: Option<WidgetPressState>,
    /// Whether user has released the mouse button since last frame.
    pub just_released: bool,
    /// Set to `true` on frames when a press+release input was registered as a click.
    pub clicked: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct WidgetPressState {
    /// Cursor position where the press started.
    pub press_position: vec2<f32>,
    /// The duration of the press.
    pub duration: f32,
}

#[derive(Default, Debug, Clone)]
pub struct WidgetSfxConfig {
    pub hover: bool,
    pub left_click: bool,
    pub right_click: bool,
}

impl WidgetState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_sfx(self, sfx_config: WidgetSfxConfig) -> Self {
        Self { sfx_config, ..self }
    }

    pub fn update(&mut self, position: Aabb2<f32>, context: &UiContext) {
        self.position = position;
        if self.visible && context.can_focus() {
            let was_hovered = self.hovered;
            self.hovered = self.position.contains(context.cursor.position);

            self.mouse_left
                .update(context, self.hovered, &context.cursor.left);
            self.mouse_right
                .update(context, self.hovered, &context.cursor.right);

            let context = &context.context;
            if self.mouse_left.clicked && self.sfx_config.left_click {
                context.sfx.play(&context.assets.sounds.ui_click);
            }
            if self.mouse_right.clicked && self.sfx_config.right_click {
                context.sfx.play(&context.assets.sounds.ui_click);
            }
            if !was_hovered && self.hovered && self.sfx_config.hover {
                context.sfx.play(&context.assets.sounds.ui_hover);
            }
        } else {
            self.mouse_left.just_released = self.mouse_left.pressed.is_some();
            self.mouse_right.just_released = self.mouse_right.pressed.is_some();

            self.hovered = false;
            self.mouse_left = WidgetMouseState::default();
            self.mouse_right = WidgetMouseState::default();
        }
    }

    pub fn show(&mut self) {
        self.visible = true;
    }

    pub fn hide(&mut self) {
        self.visible = false;
        self.hovered = false;
        self.mouse_left = WidgetMouseState::default();
        self.mouse_right = WidgetMouseState::default();
    }
}

impl Default for WidgetState {
    fn default() -> Self {
        Self {
            id: WidgetId::default(),
            position: Aabb2::ZERO.extend_uniform(1.0),
            visible: true,
            hovered: false,
            mouse_left: WidgetMouseState::default(),
            mouse_right: WidgetMouseState::default(),
            sfx_config: WidgetSfxConfig::default(),
        }
    }
}

impl WidgetMouseState {
    pub fn update(&mut self, context: &UiContext, hovered: bool, mouse: &MouseButtonContext) {
        let was_pressed = self.pressed.is_some();
        // TODO: check for mouse being pressed and then dragged onto the widget
        let pressed = mouse.down && (was_pressed || hovered && !mouse.was_down);
        self.just_pressed = !was_pressed && pressed;
        self.just_released = was_pressed && !pressed;
        self.clicked = self.just_released
            && self.pressed.is_some_and(|state| {
                (state.press_position - context.cursor.position).len() < MAX_CLICK_DISTANCE
            });
        self.pressed = if pressed {
            match self.pressed {
                Some(mut state) => {
                    state.duration += context.delta_time;
                    Some(state)
                }
                None => Some(WidgetPressState {
                    press_position: context.cursor.position,
                    duration: 0.0,
                }),
            }
        } else {
            None
        };
    }
}

impl WidgetSfxConfig {
    pub fn none() -> Self {
        Self::default()
    }

    pub fn all() -> Self {
        Self {
            hover: true,
            left_click: true,
            right_click: true,
        }
    }

    pub fn hover() -> Self {
        Self {
            hover: true,
            ..default()
        }
    }

    pub fn hover_left() -> Self {
        Self {
            hover: true,
            left_click: true,
            ..default()
        }
    }

    pub fn hover_right() -> Self {
        Self {
            hover: true,
            right_click: true,
            ..default()
        }
    }

    pub fn hover_left_right() -> Self {
        Self {
            hover: true,
            left_click: true,
            right_click: true,
            // ..default()
        }
    }
}
