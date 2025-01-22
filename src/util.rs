mod lerp;
mod sod;
mod texture_atlas;

pub use self::{lerp::*, sod::*, texture_atlas::*};

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

pub fn wrap_text(font: &geng::Font, text: &str, target_width: f32) -> Vec<String> {
    let mut lines = Vec::new();
    for source_line in text.lines() {
        let mut line = String::new();
        for word in source_line.split_whitespace() {
            if line.is_empty() {
                line += word;
                continue;
            }
            if font
                .measure(
                    &(line.clone() + " " + word),
                    vec2::splat(geng::TextAlign::CENTER),
                )
                .unwrap_or(Aabb2::ZERO)
                .width()
                > target_width
            {
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
