use super::{
    dither::DitherRender,
    mask::MaskedRender,
    ui::UiRender,
    util::{TextRenderOptions, UtilRender},
    *,
};

use crate::game::GameUI;

pub struct GameRender {
    geng: Geng,
    // assets: Rc<Assets>,
    dither: DitherRender,
    masked: MaskedRender,
    util: UtilRender,
    ui: UiRender,
}

impl GameRender {
    pub fn new(geng: &Geng, assets: &Rc<Assets>) -> Self {
        Self {
            geng: geng.clone(),
            // assets: assets.clone(),
            dither: DitherRender::new(geng, assets),
            masked: MaskedRender::new(geng, assets, vec2(1, 1)),
            util: UtilRender::new(geng, assets),
            ui: UiRender::new(geng, assets),
        }
    }

    pub fn get_render_size(&self) -> vec2<usize> {
        self.dither.get_render_size()
    }

    pub fn draw_world(&mut self, model: &Model, old_framebuffer: &mut ugli::Framebuffer) {
        self.dither.set_noise(1.0);
        let mut framebuffer = self.dither.start();

        let camera = &model.camera;
        let theme = &model.options.theme;

        if !model.level.config.modifiers.sudden {
            // Telegraphs
            for tele in &model.level_state.telegraphs {
                let color = if tele.light.danger {
                    THEME.danger
                } else {
                    THEME.light
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
                self.util
                    .draw_light(light, color, THEME.dark, camera, &mut framebuffer);
            }
        }

        let fading = model.restart_button.is_fading() || model.exit_button.is_fading();
        if let State::Lost { .. } | State::Finished = model.state {
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

            self.util.draw_text(
                "made in rust btw",
                vec2(0.0, -3.0).as_r32(),
                TextRenderOptions::new(0.7).color(THEME.dark),
                camera,
                &mut framebuffer,
            );
        }

        if !model.level.config.modifiers.clean_auto {
            self.util
                .draw_player(&model.player, camera, &mut framebuffer);
        }

        if !fading {
            match model.state {
                State::Starting { .. } | State::Playing => {}
                State::Lost { .. } => {
                    self.util.draw_text(
                        "YOU FAILED TO CHASE THE LIGHT",
                        vec2(0.0, 3.5).as_r32(),
                        TextRenderOptions::new(1.0).color(THEME.light),
                        camera,
                        &mut framebuffer,
                    );
                }
                State::Finished => {
                    self.util.draw_text(
                        "YOU CAUGHT THE LIGHT",
                        vec2(0.0, 3.5).as_r32(),
                        TextRenderOptions::new(1.0).color(THEME.light),
                        camera,
                        &mut framebuffer,
                    );
                }
            }
        }

        if let State::Playing = model.state {
            if !model.level.config.modifiers.clean_auto {
                self.util.draw_health(
                    &model.player.health,
                    model.player.get_lit_state(),
                    // &model.config.theme,
                    &mut framebuffer,
                );
            }
        }

        self.dither.finish(model.real_time, theme);

        let aabb = Aabb2::ZERO.extend_positive(old_framebuffer.size().as_f32());
        geng_utils::texture::DrawTexture::new(self.dither.get_buffer())
            .fit(aabb, vec2(0.5, 0.5))
            .draw(&geng::PixelPerfectCamera, &self.geng, old_framebuffer);
    }

    pub fn draw_ui(&mut self, ui: &GameUI, model: &Model, framebuffer: &mut ugli::Framebuffer) {
        self.masked.update_size(framebuffer.size());

        // let camera = &geng::PixelPerfectCamera;
        let theme = &model.options.theme;
        // let font_size = framebuffer.size().y as f32 * 0.05;

        let fading = model.restart_button.is_fading() || model.exit_button.is_fading();

        if let State::Lost { .. } | State::Finished = model.state {
            if !fading {
                self.util.draw_text(
                    &format!("SCORE: {}", model.score.calculated.combined),
                    vec2(-3.0, -3.0),
                    TextRenderOptions::new(0.7).color(theme.light),
                    &model.camera,
                    framebuffer,
                );
            }
        } else if !model.level.config.modifiers.clean_auto {
            // self.util.draw_text(
            //     format!("SCORE: {}", model.score.calculated.combined),
            //     vec2(-0.8, 4.5).as_r32(),
            //     TextRenderOptions::new(0.7)
            //         .color(theme.light)
            //         .align(vec2(0.0, 0.5)),
            //     &model.camera,
            //     framebuffer,
            // );
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

        if ui.leaderboard.state.visible {
            self.ui
                .draw_leaderboard(&ui.leaderboard, theme, &mut self.masked, framebuffer);
        }
    }
}
