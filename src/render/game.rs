use super::{
    dither::DitherRender,
    mask::MaskedRender,
    ui::UiRender,
    util::{TextRenderOptions, UtilRender},
    *,
};

use crate::game::GameUI;

pub struct GameRender {
    context: Context,
    pub dither: DitherRender,
    masked: MaskedRender,
    masked2: MaskedRender,
    pub util: UtilRender,
    ui: UiRender,

    font_size: f32,
}

impl GameRender {
    pub fn new(context: Context) -> Self {
        Self {
            dither: DitherRender::new(&context.geng, &context.assets),
            masked: MaskedRender::new(&context.geng, &context.assets, vec2(1, 1)),
            masked2: MaskedRender::new(&context.geng, &context.assets, vec2(1, 1)),
            util: UtilRender::new(context.clone()),
            ui: UiRender::new(context.clone()),
            context,

            font_size: 1.0,
        }
    }

    pub fn get_render_size(&self) -> vec2<usize> {
        self.dither.get_render_size()
    }

    pub fn draw_world(
        &mut self,
        model: &Model,
        _debug_mode: bool,
        old_framebuffer: &mut ugli::Framebuffer,
    ) {
        self.dither.set_noise(1.0);
        let mut framebuffer = self.dither.start();
        let options = self.context.get_options();

        let camera = &model.camera;
        let theme = &model.options.theme;
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
                self.util
                    .draw_outline(&tele.light.collider, 0.05, color, camera, &mut framebuffer);
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
                self.util.draw_light(
                    light,
                    color,
                    THEME.dark,
                    beat_time,
                    camera,
                    &mut framebuffer,
                );
            }
        }

        if let Some(button) = &model.transition_button {
            self.util
                .draw_button(button, "", &THEME, camera, &mut framebuffer);
        }

        let fading = model.restart_button.is_fading() || model.exit_button.is_fading();
        let end_screen = matches!(model.state, State::Lost { .. } | State::Finished);
        if end_screen {
            for (button, text) in [
                (&model.restart_button, "RESTART"),
                (&model.exit_button, "EXIT"),
            ] {
                if fading && !button.is_fading() {
                    continue;
                }
                let button = smooth_button(button, model.switch_time);
                self.util
                    .draw_button(&button, text, &THEME, camera, &mut framebuffer);
            }
        }

        if end_screen || model.transition_button.is_some() {
            self.util.draw_text(
                "made in rust btw",
                vec2(0.0, -3.0).as_r32(),
                TextRenderOptions::new(0.5).color(THEME.dark),
                camera,
                &mut framebuffer,
            );
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
                .transformed(Transform { scale, ..default() });
            visual.position = rhythm.position;
            self.util
                .draw_outline(&visual, 0.05, color, camera, &mut framebuffer);
        }

        if !model.level.config.modifiers.clean_auto {
            self.util
                .draw_player(&model.player, camera, &mut framebuffer);
        }

        if !fading {
            let t = (model.switch_time.as_f32() / 2.0).min(1.0);
            // let t = crate::util::smoothstep(t);
            let color = THEME.light.map_rgb(|x| x * t);
            match model.state {
                State::Starting { .. } | State::Playing => {}
                State::Lost { .. } => {
                    self.util.draw_text(
                        "DARKNESS ABSORBS YOU",
                        vec2(0.0, 3.5).as_r32(),
                        TextRenderOptions::new(0.8).color(color),
                        camera,
                        &mut framebuffer,
                    );
                }
                State::Finished => {
                    self.util.draw_text(
                        "YOU CAUGHT THE LIGHT",
                        vec2(0.0, 3.5).as_r32(),
                        TextRenderOptions::new(1.0).color(color),
                        camera,
                        &mut framebuffer,
                    );
                }
            }
        }

        if let State::Playing = model.state
            && !model.level.config.modifiers.clean_auto
        {
            self.util.draw_health(
                &model.player.health,
                model.player.get_lit_state(),
                // &model.config.theme,
                &mut framebuffer,
            );
        }

        // TODO: option
        // {
        //     // Rhythm
        //     let radius = 0.2;
        //     for rhythm in &model.rhythms {
        //         let t = rhythm.time.get_ratio().as_f32();

        //         let t = t * f32::PI;
        //         let (sin, cos) = t.sin_cos();
        //         let pos = vec2(0.0, 5.0 - radius) + vec2(cos, sin) * vec2(1.0, -0.2);

        //         let color = if rhythm.perfect {
        //             THEME.light
        //         } else {
        //             THEME.danger
        //         };

        //         self.geng.draw2d().draw2d(
        //             &mut framebuffer,
        //             camera,
        //             &draw2d::Ellipse::circle(pos, radius, color),
        //         );
        //     }
        // }

        self.dither.finish(model.real_time, theme);

        let aabb = Aabb2::ZERO.extend_positive(old_framebuffer.size().as_f32());
        geng_utils::texture::DrawTexture::new(self.dither.get_buffer())
            .fit(aabb, vec2(0.5, 0.5))
            .draw(
                &geng::PixelPerfectCamera,
                &self.context.geng,
                old_framebuffer,
            );
    }

    pub fn draw_ui(
        &mut self,
        ui: &GameUI,
        model: &Model,
        debug_mode: bool,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        self.font_size = framebuffer.size().y as f32 * 0.04;
        self.masked.update_size(framebuffer.size());
        self.masked2.update_size(framebuffer.size());

        // let camera = &geng::PixelPerfectCamera;
        let theme = model.options.theme;
        // let font_size = framebuffer.size().y as f32 * 0.05;

        // let fading = model.restart_button.is_fading() || model.exit_button.is_fading();

        let accuracy = model.score.calculated.accuracy.as_f32() * 100.0;
        // let precision = model.score.calculated.precision.as_f32() * 100.0;

        if let State::Lost { .. } | State::Finished = model.state {
        } else if !model.level.config.modifiers.clean_auto {
            self.util.draw_text(
                format!("SCORE: {}", model.score.calculated.combined),
                vec2(-8.5, 4.5).as_r32(),
                TextRenderOptions::new(0.7)
                    .color(theme.light)
                    .align(vec2(0.0, 0.5)),
                &model.camera,
                framebuffer,
            );

            self.util.draw_text(
                format!("{accuracy:3.2}%"),
                vec2(-8.5, 3.9).as_r32(),
                TextRenderOptions::new(0.7)
                    .color(theme.light)
                    .align(vec2(0.0, 0.5)),
                &model.camera,
                framebuffer,
            );

            // self.util.draw_text(
            //     format!("{:3.2}%", precision),
            //     vec2(-8.0, 3.5).as_r32(),
            //     TextRenderOptions::new(0.7)
            //         .color(theme.light)
            //         .align(vec2(0.0, 0.5)),
            //     &model.camera,
            //     framebuffer,
            // );

            let position = Aabb2::point(vec2(-8.3, 3.2)).extend_uniform(0.3);
            for (i, modifier) in model.level.config.modifiers.iter().enumerate() {
                let position = position.translate(vec2(i as f32, 0.0) * position.size());
                if let Ok(position) = model
                    .camera
                    .world_to_screen(framebuffer.size().as_f32(), position.center())
                {
                    let texture = self.context.assets.get_modifier(modifier);
                    self.ui.draw_subtexture(
                        Aabb2::point(position),
                        &texture,
                        theme.light,
                        1.0,
                        framebuffer,
                    );
                }
            }

            if debug_mode {
                self.util.draw_text(
                    format!("{:#?}", model.score),
                    vec2(-7.0, 0.0).as_r32(),
                    TextRenderOptions::new(0.7)
                        .color(theme.light)
                        .align(vec2(0.0, 0.5)),
                    &model.camera,
                    framebuffer,
                );
            }
        }

        if ui.score.state.visible {
            self.ui
                .draw_score(&ui.score, theme, self.font_size * 0.1, framebuffer);
        }
        if ui.leaderboard.state.visible {
            let theme = self.context.get_options().theme;
            let width = self.font_size * 0.2;

            self.ui.draw_window(
                &mut self.masked,
                ui.leaderboard.state.position,
                Some(ui.leaderboard_head.state.position),
                width,
                theme,
                framebuffer,
                |framebuffer| {
                    self.ui.draw_leaderboard(
                        &ui.leaderboard,
                        theme,
                        self.font_size * 0.1,
                        &mut self.masked2,
                        framebuffer,
                    );
                },
            );
            self.ui.draw_text(&ui.leaderboard_head, framebuffer);
        }
    }
}
