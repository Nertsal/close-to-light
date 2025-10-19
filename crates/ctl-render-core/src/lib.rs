mod texture_atlas;

pub use self::texture_atlas::*;

use geng::prelude::*;
use geng_utils::conversions::Vec2RealConversions;

pub fn get_pixel_scale(framebuffer_size: vec2<usize>) -> f32 {
    const TARGET_SIZE: vec2<usize> = vec2(640, 360);
    let size = framebuffer_size.as_f32();
    let ratio = size / TARGET_SIZE.as_f32();
    ratio.x.min(ratio.y)
}

type Color = Rgba<f32>;

#[derive(Debug, Clone, Copy)]
pub struct TextRenderOptions {
    pub size: f32,
    pub align: vec2<f32>,
    pub color: Color,
    pub hover_color: Color,
    pub press_color: Color,
    pub rotation: Angle,
}

#[derive(Debug, Clone, Copy)]
pub struct DashRenderOptions {
    pub width: f32,
    pub dash_length: f32,
    pub space_length: f32,
}

impl TextRenderOptions {
    pub fn new(size: f32) -> Self {
        Self { size, ..default() }
    }

    // pub fn size(self, size: f32) -> Self {
    //     Self { size, ..self }
    // }

    pub fn align(self, align: vec2<f32>) -> Self {
        Self { align, ..self }
    }

    pub fn color(self, color: Color) -> Self {
        Self { color, ..self }
    }
}

impl Default for TextRenderOptions {
    fn default() -> Self {
        Self {
            size: 1.0,
            align: vec2::splat(0.5),
            color: Color::WHITE,
            hover_color: Color {
                r: 0.7,
                g: 0.7,
                b: 0.7,
                a: 1.0,
            },
            press_color: Color {
                r: 0.5,
                g: 0.5,
                b: 0.5,
                a: 1.0,
            },
            rotation: Angle::ZERO,
        }
    }
}
