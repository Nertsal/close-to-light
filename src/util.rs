mod lerp;
mod sod;
mod texture_atlas;

pub use self::{lerp::*, sod::*, texture_atlas::*};

use crate::assets::Font;

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

pub fn wrap_text(font: &Font, text: &str, target_width: f32) -> Vec<String> {
    let mut lines = Vec::new();
    for source_line in text.lines() {
        let mut line = String::new();
        for word in source_line.split_whitespace() {
            if line.is_empty() {
                line += word;
                continue;
            }
            if font.measure(&(line.clone() + " " + word), 1.0).width() > target_width {
                lines.push(line);
                line = word.to_string();
            } else {
                line += " ";
                line += word;
            }
        }
        if !line.is_empty() {
            lines.push(line);
        }
    }
    lines
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
