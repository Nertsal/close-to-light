use crate::{
    prelude::*,
    render::{
        THEME,
        dither::DitherRender,
        game::GameRender,
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
const THIRD_HIT: f32 = convert(22, 33);
const FOURTH_HIT: f32 = convert(28, 32);
const FIFTH_HIT: f32 = convert(33, 50);
const OUTRO: f32 = convert(39, 35);

pub struct TrailerState {
    context: Context,

    game_render: GameRender,
    util_render: UtilRender,
    ui_render: UiRender,
    dither: DitherRender,
    post: PostRender,

    model: Model,

    theme: Theme,
    time: FloatTime,
    camera: Camera2d,
    load_texts: Vec<&'static str>,
}

impl TrailerState {
    pub fn new(context: Context, level: PlayLevel) -> Self {
        let mut state = Self {
            game_render: GameRender::new(context.clone()),
            util_render: UtilRender::new(context.clone()),
            ui_render: UiRender::new(context.clone()),
            dither: DitherRender::new(&context.geng, &context.assets),
            post: PostRender::new(context.clone()),

            model: Model::new(
                context.clone(),
                level,
                ctl_local::Leaderboard::new(&context.geng, None, &context.local.fs),
            ),

            theme: Theme::linksider(),
            time: FloatTime::ZERO,
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
        };
        state.model.start(0);
        state
    }
}

impl geng::State for TrailerState {
    fn update(&mut self, delta_time: f64) {
        let delta_time = FloatTime::new(delta_time as f32);
        self.time += delta_time;
        self.model.update(vec2::ZERO, delta_time);
    }

    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        let mut theme = self.theme;
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
            theme = lerp_theme(theme, Theme::peach_mint(), t);
        }
        if self.time.as_f32() > FIFTH_HIT - 0.25 {
            let t = ((self.time.as_f32() - FIFTH_HIT + 0.25) / 0.5).clamp(0.0, 1.0);
            theme = lerp_theme(theme, Theme::linksider(), t);
        }

        ugli::clear(framebuffer, Some(theme.dark), None, None);

        let post_buffer = &mut self.post.begin(framebuffer.size(), theme.dark);
        // self.game_render.draw_world(&self.model, false, post_buffer);

        let mut dither_buffer = self.dither.start();

        let options = self.context.get_options();

        if self.time.as_f32() > INTRO_TIME {
            // Level
            let model = &self.model;
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
                        &self.model.camera,
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
                        &self.model.camera,
                        &mut dither_buffer,
                    );
                }
            }
        }

        if self.time.as_f32() < INTRO_TIME {
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

        {
            // Render dithered
            let mut dither_theme = theme;
            if self.time.as_f32() < INTRO_TIME {
                let danger_t =
                    crate::util::smoothstep((self.time.as_f32() / 1.5 - 1.5).clamp(0.0, 1.0));
                dither_theme.light = Color::lerp(dither_theme.light, dither_theme.danger, danger_t);
            }
            self.dither.finish(self.time, &dither_theme.transparent());
        }

        // Draw to post buffer
        geng_utils::texture::DrawTexture::new(self.dither.get_buffer())
            .fit_screen(vec2(0.5, 0.5), post_buffer)
            .draw(&geng::PixelPerfectCamera, &self.context.geng, post_buffer);

        if self.time.as_f32() < INTRO_TIME {
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

        if self.time.as_f32() < INTRO_TIME {
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
            self.util_render.draw_text(
                self.load_texts
                    [((self.time.as_f32() / 1.5).floor() as usize).min(self.load_texts.len() - 1)],
                vec2(0.0, -0.5),
                TextRenderOptions::new(0.8).color(theme.light),
                &self.camera,
                post_buffer,
            );
            self.util_render.draw_text(
                "by Nertsal",
                vec2(-5.5, 1.75),
                TextRenderOptions::new(0.7)
                    .align(vec2(0.0, 0.5))
                    .color(theme.light),
                &self.camera,
                post_buffer,
            );
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

        if self.time.as_f32() < INTRO_TIME + 0.5 {
            // Transition light
            let dither_buffer = &mut self.dither.start();
            let collider = Collider {
                position: vec2::ZERO,
                rotation: Angle::ZERO,
                shape: Shape::Circle { radius: r32(1.0) },
            };
            let scale = ((self.time.as_f32() - INTRO_TIME + 1.5) * 5.0)
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
                crate::util::smoothstep(((self.time.as_f32() - INTRO_TIME) / 0.5).clamp(0.0, 1.0)),
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
