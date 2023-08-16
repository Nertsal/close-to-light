use geng::prelude::*;

pub fn smoothstep<T: Float>(t: T) -> T {
    T::from_f32(3.0) * t * t - T::from_f32(2.0) * t * t * t
}

/// Returns the given color with the multiplied alpha.
pub fn with_alpha(mut color: Rgba<f32>, alpha: f32) -> Rgba<f32> {
    color.a *= alpha;
    color
}
