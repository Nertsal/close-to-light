mod lerp;
mod sod;
mod task;

pub use self::{lerp::*, sod::*, task::*};

use geng::prelude::*;
use geng_utils::bounded::Bounded;

pub fn smoothstep<T: Float>(t: T) -> T {
    T::from_f32(3.0) * t * t - T::from_f32(2.0) * t * t * t
}

/// Returns the given color with the multiplied alpha.
pub fn with_alpha(mut color: Rgba<f32>, alpha: f32) -> Rgba<f32> {
    color.a *= alpha;
    color
}

pub fn world_to_screen(
    camera: &impl geng::AbstractCamera2d,
    framebuffer_size: vec2<f32>,
    pos: vec2<f32>,
) -> vec2<f32> {
    let pos = (camera.projection_matrix(framebuffer_size) * camera.view_matrix()) * pos.extend(1.0);
    let pos = pos.xy() / pos.z;
    vec2(
        (pos.x + 1.0) / 2.0 * framebuffer_size.x,
        (pos.y + 1.0) / 2.0 * framebuffer_size.y,
    )
}

pub fn overflow_scroll(
    delta_time: f32,
    current: f32,
    target: &mut f32,
    content_size: f32,
    visible_size: f32,
) {
    let overflow_up = *target;
    let height = content_size - current;
    let max_scroll = (height - visible_size).max(0.0);
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
