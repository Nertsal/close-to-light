use crate::{
    prelude::*,
    render::{
        THEME,
        dither::DitherRender,
        post::{PostRender, PostVfx},
        ui::UiRender,
        util::UtilRender,
    },
};

use ctl_render_core::TextRenderOptions;
use geng_utils::conversions::AngleRealConversions;

const fn convert(seconds: i32, fraction: i32) -> f32 {
    seconds as f32 + fraction as f32 / 60.0
}

const INTRO_TIME: f32 = 5.0;
const FIRST_HIT: f32 = convert(11, 15);
const SECOND_HIT: f32 = convert(16, 55);
// const THIRD_HIT: f32 = convert(22, 33);
const FOURTH_HIT: f32 = convert(28, 32);
const FIFTH_HIT: f32 = convert(33, 50);
const OUTRO: f32 = convert(39, 35);

pub struct TrailerState {
    context: Context,
    /// When set to `true`, disables all hard-coded trailer-specific effects.
    custom: bool,
    duration: FloatTime,

    util_render: UtilRender,
    ui_render: UiRender,
    dither: DitherRender,
    post: PostRender,
    framebuffer_size: vec2<usize>,

    model: Model,

    theme: Theme,
    time: FloatTime,
    camera: Camera2d,
    load_texts: Vec<&'static str>,
}

impl TrailerState {
    pub fn new(
        context: Context,
        level: PlayLevel,
        custom: bool,
        duration: Option<FloatTime>,
    ) -> Self {
        context
            .geng
            .window()
            .set_cursor_type(geng::CursorType::None);

        let start_time = level.start_time;
        let mut state = Self {
            util_render: UtilRender::new(context.clone()),
            ui_render: UiRender::new(context.clone()),
            dither: DitherRender::new(&context.geng, &context.assets),
            post: PostRender::new(context.clone()),
            framebuffer_size: vec2(1, 1),

            model: Model::new(
                context.clone(),
                level,
                ctl_local::Leaderboard::new(&context.geng, None, &context.local.fs),
            ),

            theme: Theme::linksider(),
            time: if custom {
                FloatTime::ZERO
            } else {
                time_to_seconds(start_time)
            },
            camera: Camera2d {
                center: vec2::ZERO,
                rotation: Angle::ZERO,
                fov: Camera2dFov::Vertical(10.0),
            },
            load_texts: vec![
                "Turning the lights on...",
                "Initializing evil... >:3",
                "Are you ready?",
            ],
            context,
            custom,
            duration: duration.unwrap_or(r32(OUTRO)),
        };
        state.model.start(start_time);
        state
    }
}

impl geng::State for TrailerState {
    fn fixed_update(&mut self, delta_time: f64) {
        let delta_time = FloatTime::new(delta_time as f32);
        self.time += delta_time;

        let pos = self
            .context
            .geng
            .window()
            .cursor_position()
            .unwrap_or(vec2::ZERO)
            .as_f32();
        let game_pos = geng_utils::layout::fit_aabb(
            self.framebuffer_size.as_f32(),
            Aabb2::ZERO.extend_positive(self.framebuffer_size.as_f32()),
            vec2(0.5, 0.5),
        );
        let pos = pos - game_pos.bottom_left();
        let target_pos = self
            .model
            .camera
            .screen_to_world(game_pos.size(), pos)
            .as_r32();
        self.model.update(target_pos, delta_time);
    }

    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        self.framebuffer_size = framebuffer.size();

        let mut theme = self.theme;
        if !self.custom {
            if self.time.as_f32() > FIRST_HIT - 0.25 {
                let t = ((self.time.as_f32() - FIRST_HIT + 0.25) / 0.5).clamp(0.0, 1.0);
                theme = lerp_theme(theme, Theme::corruption(), t);
            }
            if self.time.as_f32() > SECOND_HIT - 0.25 {
                let t = ((self.time.as_f32() - SECOND_HIT + 0.25) / 0.5).clamp(0.0, 1.0);
                theme = lerp_theme(theme, Theme::classic(), t);
            }
            // if self.time.as_f32() > THIRD_HIT - 0.25 {
            //     let t = ((self.time.as_f32() - THIRD_HIT + 0.25) / 0.5).clamp(0.0, 1.0);
            //     theme = lerp_theme(theme, Theme::peach_mint(), t);
            // }
            if self.time.as_f32() > FOURTH_HIT - 0.25 {
                let t = ((self.time.as_f32() - FOURTH_HIT + 0.25) / 0.5).clamp(0.0, 1.0);
                theme = lerp_theme(theme, Theme::stargazer(), t);
            }
            if self.time.as_f32() > FIFTH_HIT - 0.25 {
                let t = ((self.time.as_f32() - FIFTH_HIT + 0.25) / 0.5).clamp(0.0, 1.0);
                theme = lerp_theme(theme, Theme::linksider(), t);
            }
        }

        ugli::clear(framebuffer, Some(theme.dark), None, None);

        let mut dither_buffer = self.dither.start();

        let options = self.context.get_options();
        let intro_time = if self.custom { 2.0 } else { INTRO_TIME };

        if self.time.as_f32() > intro_time {
            // Level
            let model = &mut self.model;
            let beat_time = model
                .level
                .level
                .data
                .timing
                .get_timing(model.play_time_ms)
                .beat_time;

            if !model.level.config.modifiers.sudden {
                // Telegraphs
                for tele in &model.level_state.telegraphs {
                    let color = if tele.light.danger {
                        THEME.danger
                    } else {
                        THEME.get_color(options.graphics.lights.telegraph_color)
                    };
                    self.util_render.draw_outline(
                        &tele.light.collider,
                        0.05,
                        color,
                        &model.camera,
                        &mut dither_buffer,
                    );
                }
            }

            if !model.level.config.modifiers.hidden {
                // Lights
                for light in &model.level_state.lights {
                    let color = if light.danger {
                        THEME.danger
                    } else {
                        THEME.light
                    };
                    self.util_render.draw_light(
                        light,
                        color,
                        THEME.dark,
                        beat_time,
                        &model.camera,
                        &mut dither_buffer,
                    );
                }
            }

            // Rhythm feedback
            for rhythm in &model.rhythms {
                let color = if rhythm.perfect {
                    THEME.highlight
                } else {
                    THEME.danger
                };
                let t = rhythm.time.clone().map(|t| t as f32).get_ratio();

                let scale = r32(crate::util::smoothstep(1.0 - t));
                let mut visual = model
                    .player
                    .collider
                    .transformed(TransformLight { scale, ..default() });
                visual.position = rhythm.position;
                self.util_render.draw_outline(
                    &visual,
                    0.05,
                    color,
                    &model.camera,
                    &mut dither_buffer,
                );
            }

            if !model.level.config.modifiers.clean_auto {
                let t = crate::util::smoothstep(
                    ((self.duration - self.time).as_f32() / 0.5).clamp(0.0, 1.0),
                );
                let mut options = self.context.get_options();
                options.cursor.inner_radius = 0.15 * t;
                self.context.set_options(options);
                model.player.collider.shape = Shape::circle(r32(0.5 * t));
                self.util_render
                    .draw_player(&model.player, &model.camera, &mut dither_buffer);
            }
        }

        if !self.custom && self.time.as_f32() < INTRO_TIME {
            // Loading screen lights
            let loading_lights = [
                Collider {
                    position: vec2(-8.75, 4.5).as_r32(),
                    rotation: Angle::from_degrees(15.0).as_r32(),
                    shape: Shape::Line { width: r32(1.7) },
                },
                Collider {
                    position: vec2(8.75, 0.5).as_r32(),
                    rotation: Angle::from_degrees(75.0).as_r32(),
                    shape: Shape::Line { width: r32(0.95) },
                },
                Collider {
                    position: vec2(-8.75, -5.0).as_r32(),
                    rotation: Angle::ZERO,
                    shape: Shape::Circle { radius: r32(1.2) },
                },
                Collider {
                    position: vec2(2.5, 5.0).as_r32(),
                    rotation: Angle::ZERO,
                    shape: Shape::Circle { radius: r32(0.9) },
                },
                Collider {
                    position: vec2(3.5, -4.5).as_r32(),
                    rotation: Angle::ZERO,
                    shape: Shape::Circle { radius: r32(0.5) },
                },
                Collider {
                    position: vec2(-7.5, -0.5).as_r32(),
                    rotation: Angle::ZERO,
                    shape: Shape::Circle { radius: r32(0.25) },
                },
            ];
            for (i, collider) in loading_lights.into_iter().enumerate() {
                let offset = (i as f32 - 2.0) / 5.0;
                let scale = crate::util::smoothstep(
                    ((self.time.as_f32() - 0.75 + offset) / 1.5).clamp(0.0, 1.0),
                );
                self.util_render.draw_light(
                    &Light {
                        base_collider: collider.clone(),
                        collider: Collider {
                            shape: collider.shape.scaled(r32(scale)),
                            ..collider
                        },
                        lifetime: 0,
                        danger: false,
                        hollow: R32::ZERO,
                        event_id: None,
                        closest_waypoint: (100, WaypointId::Initial),
                    },
                    THEME.light,
                    THEME.dark,
                    r32(0.01),
                    &self.camera,
                    &mut dither_buffer,
                );
            }
        }

        if self.time > self.duration {
            // Outro screen
            let t = ((self.time - self.duration).as_f32() / 2.5).clamp(0.0, 1.0);
            let light = crate::util::with_alpha(THEME.light, t);

            if let Ok(pos) = self
                .camera
                .world_to_screen(dither_buffer.size().as_f32(), vec2(0.0, 2.0))
            {
                self.ui_render.draw_texture(
                    Aabb2::point(pos),
                    &self.context.assets.sprites.title,
                    light,
                    2.0,
                    &mut dither_buffer,
                );
            }
            self.util_render.draw_text(
                "Wishlist now on Steam!",
                vec2(0.0, 0.0),
                TextRenderOptions::new(1.0).color(light),
                &self.camera,
                &mut dither_buffer,
            );
        }

        {
            // Render dithered
            let mut dither_theme = theme;
            if !self.custom && self.time.as_f32() < INTRO_TIME {
                let danger_t =
                    crate::util::smoothstep((self.time.as_f32() / 1.5 - 1.5).clamp(0.0, 1.0));
                dither_theme.light = Color::lerp(dither_theme.light, dither_theme.danger, danger_t);
            }
            self.dither.finish(self.time, &dither_theme.transparent());
        }

        // Draw to post buffer
        let post_buffer = &mut self.post.begin(framebuffer.size(), theme.dark);
        geng_utils::texture::DrawTexture::new(self.dither.get_buffer())
            .fit_screen(vec2(0.5, 0.5), post_buffer)
            .draw(&geng::PixelPerfectCamera, &self.context.geng, post_buffer);

        if !self.custom && self.time.as_f32() < INTRO_TIME {
            // Fake loading bar
            let font_size = 1.0;
            let size = vec2(10.0, 0.8) * font_size;
            let load_bar = Aabb2::point(vec2(0.0, -font_size * 1.5)).extend_symmetric(size / 2.0);
            let fill_bar = load_bar.extend_uniform(-font_size * 0.1);
            let t = (self.time.as_f32() / 1.5 / 3.0).min(1.0);
            let t = crate::util::smoothstep(t);
            let fill_bar = fill_bar.extend_right((t - 1.0) * fill_bar.width());
            self.context
                .geng
                .draw2d()
                .quad(post_buffer, &self.camera, load_bar, theme.light);
            self.context
                .geng
                .draw2d()
                .quad(post_buffer, &self.camera, fill_bar, theme.highlight);
        }

        if self.time.as_f32() < intro_time {
            // Title screen
            if let Ok(pos) = self
                .camera
                .world_to_screen(framebuffer.size().as_f32(), vec2(0.0, 3.0))
            {
                self.ui_render.draw_texture(
                    Aabb2::point(pos),
                    &self.context.assets.sprites.title,
                    theme.light,
                    2.0,
                    post_buffer,
                );
            }
            if !self.custom {
                self.util_render.draw_text(
                    self.load_texts[((self.time.as_f32() / 1.5).floor() as usize)
                        .min(self.load_texts.len() - 1)],
                    vec2(0.0, -0.5),
                    TextRenderOptions::new(0.8).color(theme.light),
                    &self.camera,
                    post_buffer,
                );
            }
            self.util_render.draw_text(
                "by Nertsal",
                vec2(-5.5, 1.75),
                TextRenderOptions::new(0.7)
                    .align(vec2(0.0, 0.5))
                    .color(theme.light),
                &self.camera,
                post_buffer,
            );
            if !self.custom {
                self.util_render.draw_text(
                    "music by IcyLava",
                    vec2(5.5, 1.75),
                    TextRenderOptions::new(0.7)
                        .align(vec2(1.0, 0.5))
                        .color(theme.light),
                    &self.camera,
                    post_buffer,
                );
            }
        }

        if self.time.as_f32() < intro_time + 0.5 {
            // Transition light
            let dither_buffer = &mut self.dither.start();
            let collider = Collider {
                position: vec2::ZERO,
                rotation: Angle::ZERO,
                shape: Shape::Circle { radius: r32(1.0) },
            };
            let scale = ((self.time.as_f32() - intro_time + 1.5) * 5.0)
                .clamp(0.0, 50.0)
                .powi(3);
            self.util_render.draw_light(
                &Light {
                    base_collider: collider.clone(),
                    collider: Collider {
                        shape: collider.shape.scaled(r32(scale)),
                        ..collider
                    },
                    lifetime: 0,
                    danger: false,
                    hollow: R32::ZERO,
                    event_id: None,
                    closest_waypoint: (100, WaypointId::Initial),
                },
                THEME.light,
                THEME.dark,
                r32(0.01),
                &self.camera,
                dither_buffer,
            );

            let mut theme = theme.transparent();
            theme.light = Color::lerp(
                theme.light,
                theme.dark,
                crate::util::smoothstep(((self.time.as_f32() - intro_time) / 0.5).clamp(0.0, 1.0)),
            );

            self.dither.finish(self.time, &theme);
            geng_utils::texture::DrawTexture::new(self.dither.get_buffer())
                .fit_screen(vec2(0.5, 0.5), post_buffer)
                .draw(&geng::PixelPerfectCamera, &self.context.geng, post_buffer);
        }

        // Post processing effects
        self.post.post_process(
            PostVfx {
                time: self.time,
                crt: true,
                rgb_split: self.model.vfx.rgb_split.value.current.as_f32(),
            },
            framebuffer,
        );
    }
}

fn lerp_theme(from: Theme, to: Theme, t: f32) -> Theme {
    Theme {
        dark: Color::lerp(from.dark, to.dark, t),
        light: Color::lerp(from.light, to.light, t),
        danger: Color::lerp(from.danger, to.danger, t),
        highlight: Color::lerp(from.highlight, to.highlight, t),
    }
}
