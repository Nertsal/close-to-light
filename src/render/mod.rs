pub mod dither;
pub mod editor;
pub mod game;
pub mod mask;
pub mod menu;
pub mod post;
pub mod ui;
pub mod util;

use crate::prelude::*;

/// Gameplay preview in options.
pub const PREVIEW_RESOLUTION: vec2<usize> = vec2(640 / 3, 360 / 3);

pub const THEME: Theme = Theme {
    dark: Color::BLACK,
    light: Color::GREEN,
    danger: Color::RED,
    highlight: Color::BLUE,
};

pub fn smooth_button(button: &HoverButton, time: FloatTime) -> HoverButton {
    // Appear at 1.0
    // Fade in until 2.0
    let t = (time - FloatTime::ONE).clamp(FloatTime::ZERO, FloatTime::ONE);
    let t = crate::util::smoothstep(t);

    let mut button = button.clone();
    button.base_collider = button.base_collider.transformed(TransformLight {
        scale: t,
        ..default()
    });
    button
}

fn draw_parameters() -> ugli::DrawParameters {
    ugli::DrawParameters {
        blend_mode: Some(ugli::BlendMode::straight_alpha()),
        ..default()
    }
}
