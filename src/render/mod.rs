pub mod dither;
pub mod editor;
pub mod game;
pub mod menu;
pub mod util;

use crate::prelude::*;

pub const THEME: Theme = Theme {
    dark: Color::BLACK,
    light: Color::GREEN,
    danger: Color::RED,
};

pub fn smooth_button(button: &HoverButton, time: Time) -> HoverButton {
    // Appear at 1.0
    // Fade in until 2.0
    let t = (time - Time::ONE).clamp(Time::ZERO, Time::ONE);
    let t = crate::util::smoothstep(t);

    let mut button = button.clone();
    button.collider = button.collider.transformed(Transform {
        scale: t,
        ..default()
    });
    button
}
