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

#[derive(ugli::Uniforms)]
pub struct ShaderUniformsCommon<'a> {
    pub u_theme_dark: Color,
    pub u_theme_light: Color,
    pub u_theme_danger: Color,
    pub u_theme_highlight: Color,

    pub u_real_time: f32,
    pub u_level_time_ms: Time,
    pub u_level_time: f32,
    pub u_relative_beat_time: f32,
    pub u_bpm: f32,
    pub u_beat_duration: f32,

    pub u_lights_sdf: &'a ugli::Texture,
}

#[derive(ugli::Uniforms)]
pub struct ShaderUniforms {
    pub u_shader_start_time_ms: Time,
    pub u_shader_start_time: f32,
    pub u_shader_duration_ms: Time,
    pub u_shader_duration: f32,
}

#[allow(clippy::type_complexity)]
pub fn prepare_shaders<'a>(
    theme: Theme,
    level: &'a Level,
    shaders: &'a [(Time, ShaderEvent)],
    real_time: FloatTime,
    play_time_ms: Time,
    level_assets: &'a LevelAssets,
    lights_sdf: &'a ugli::Texture,
) -> (
    Vec<(Time, &'a ShaderEvent, Ref<'a, Rc<ugli::Program>>)>,
    ShaderUniformsCommon<'a>,
    impl Fn(Time, &ShaderEvent) -> ShaderUniforms,
) {
    let active_shaders: Vec<(Time, &ShaderEvent, Ref<Rc<ugli::Program>>)> = shaders
        .iter()
        .flat_map(|(time, shader)| {
            level_assets
                .shaders
                .get(&shader.shader)
                .map(|program| (*time, shader, program.get()))
        })
        .collect();
    let timing = level.timing.get_timing(play_time_ms);
    let beat_duration = timing.beat_time;
    let bpm = r32(60.0) / beat_duration;
    let relative_beat_time = level.timing.get_relative_beat_time(play_time_ms).as_beats();
    let shader_uniforms_common = ShaderUniformsCommon {
        u_theme_dark: theme.dark,
        u_theme_light: theme.light,
        u_theme_danger: theme.danger,
        u_theme_highlight: theme.highlight,

        u_real_time: real_time.as_f32(),
        u_level_time_ms: play_time_ms,
        u_level_time: time_to_seconds(play_time_ms).as_f32(),
        u_relative_beat_time: relative_beat_time.as_f32(),
        u_bpm: bpm.as_f32(),
        u_beat_duration: beat_duration.as_f32(),

        u_lights_sdf: lights_sdf,
    };
    let shader_uniforms = |shader_time: Time, shader: &ShaderEvent| ShaderUniforms {
        u_shader_start_time_ms: shader_time,
        u_shader_start_time: time_to_seconds(shader_time).as_f32(),
        u_shader_duration_ms: shader.duration,
        u_shader_duration: time_to_seconds(shader.duration).as_f32(),
    };

    (active_shaders, shader_uniforms_common, shader_uniforms)
}
