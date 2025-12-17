use ctl_util::SecondOrderState;

use super::*;

use crate::widget::WidgetState;

#[derive(Debug)]
pub struct ScrollState {
    pub state: SecondOrderState<f32>,
    pub drag_from: Option<f32>,
    pub release_velocity: Option<f32>,
    pub release_slowdown: f32,
}

impl Default for ScrollState {
    fn default() -> Self {
        Self::new()
    }
}

impl ScrollState {
    pub fn new() -> Self {
        Self {
            state: SecondOrderState::new(5.0, 2.0, 0.0, 0.0),
            drag_from: None,
            release_velocity: None,
            release_slowdown: 0.85,
        }
    }

    /// Scrolling with pen-controls or mouse wheel.
    pub fn drag(&mut self, context: &UiContext, state: &WidgetState) {
        if state.mouse_left.just_pressed {
            // Start drag
            self.drag_from = Some(self.state.current);
        }

        if state.hovered {
            if let Some(press) = &state.mouse_left.pressed
                && let Some(drag_from) = self.drag_from
            {
                // Drag
                let previous = self.state.current;
                self.state.snap_to(
                    drag_from - context.cursor.position.y + press.press_position.y,
                    context.delta_time,
                );

                let velocity = (self.state.current - previous) / context.delta_time;
                self.release_velocity =
                    Some(self.release_velocity.unwrap_or_default() * 0.3 + velocity * 0.7);
                return; // NOTE
            } else if self.drag_from.take().is_some() {
                // Release
            } else if context.cursor.scroll != 0.0 {
                // Wheel scroll
                let scroll_speed = 2.0;
                self.state.target += context.cursor.scroll * scroll_speed;
                self.release_velocity = None;
            }
        }

        if let Some(velocity) = &mut self.release_velocity {
            // Slide with velocity
            self.state.snap_to(
                self.state.current + *velocity * context.delta_time,
                context.delta_time,
            );
            *velocity *= self.release_slowdown;
            if velocity.abs() < 5.0 {
                self.release_velocity = None;
            }
        } else {
            // Interpolate to target
            self.state.update(context.delta_time);
        }
    }

    /// Control scroll overflow.
    pub fn overflow(&mut self, delta_time: f32, content_size: f32, visible_size: f32) {
        let overflow_up = self.state.target;
        let max_scroll = (content_size - visible_size).max(0.0);
        let overflow_down = -max_scroll - self.state.target;
        let overflow = if overflow_up > 0.0 {
            overflow_up
        } else if overflow_down > 0.0 {
            -overflow_down
        } else {
            0.0
        };
        self.state.target -= overflow * (delta_time / 0.1).min(1.0);

        if overflow != 0.0
            && let Some(velocity) = &mut self.release_velocity
        {
            // Extra slowdown at overflow
            *velocity *= 0.5;
        }
    }
}
