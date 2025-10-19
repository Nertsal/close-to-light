use super::*;

use crate::widget::WidgetState;

use ctl_util::SecondOrderState;

pub fn overflow_scroll(delta_time: f32, target: &mut f32, content_size: f32, visible_size: f32) {
    let overflow_up = *target;
    let max_scroll = (content_size - visible_size).max(0.0);
    let overflow_down = -max_scroll - *target;
    let overflow = if overflow_up > 0.0 {
        overflow_up
    } else if overflow_down > 0.0 {
        -overflow_down
    } else {
        0.0
    };
    *target -= overflow * (delta_time / 0.1).min(1.0);
}

/// Scrolling with pen-controls.
pub fn scroll_drag(
    context: &UiContext,
    state: &WidgetState,
    scroll: &mut SecondOrderState<f32>,
    scroll_drag_from: &mut f32,
) {
    // Scroll
    if state.mouse_left.just_pressed {
        *scroll_drag_from = scroll.current;
    }
    if state.hovered {
        if let Some(press) = &state.mouse_left.pressed {
            scroll.snap_to(
                *scroll_drag_from - context.cursor.position.y + press.press_position.y,
                context.delta_time,
            );
        } else {
            let scroll_speed = 2.0;
            scroll.target += context.cursor.scroll * scroll_speed;
        }
    }
    scroll.update(context.delta_time);
}
