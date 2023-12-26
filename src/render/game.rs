use super::{
    dither::DitherRender,
    util::{TextRenderOptions, UtilRender},
    *,
};

pub struct GameRender {
    geng: Geng,
    // assets: Rc<Assets>,
    dither: DitherRender,
    util: UtilRender,
}

impl GameRender {
    pub fn new(geng: &Geng, assets: &Rc<Assets>) -> Self {
        Self {
            geng: geng.clone(),
            // assets: assets.clone(),
            dither: DitherRender::new(geng, assets),
            util: UtilRender::new(geng, assets),
        }
    }

    pub fn get_render_size(&self) -> vec2<usize> {
        self.dither.get_render_size()
    }

    pub fn draw_world(&mut self, model: &Model, old_framebuffer: &mut ugli::Framebuffer) {
        let mut framebuffer = self.dither.start();

        let camera = &model.camera;
        let theme = &model.config.theme;

        if !model.config.modifiers.sudden {
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

        if !model.config.modifiers.hidden {
            // Lights
            for light in &model.level_state.lights {
                let color = if light.danger {
                    THEME.danger
                } else {
                    THEME.light
                };
                self.util
                    .draw_light(&light.collider, color, camera, &mut framebuffer);
            }
        }

        let fading = model.restart_button.hover_time.get_ratio().as_f32() > 0.5;
        if let State::Lost { .. } | State::Finished = model.state {
            let button = smooth_button(&model.restart_button, model.switch_time);
            self.util
                .draw_button(&button, "RESTART", &THEME, camera, &mut framebuffer);

            self.util.draw_text(
                "made in rust btw",
                vec2(0.0, -3.0).as_r32(),
                TextRenderOptions::new(0.7).color(THEME.dark),
                camera,
                &mut framebuffer,
            );

            let mut draw_text = |text: &str, position: vec2<f32>, size: f32, align: vec2<f32>| {
                self.util.draw_text(
                    text,
                    position.as_r32(),
                    TextRenderOptions::new(size).align(align).color(THEME.light),
                    camera,
                    &mut framebuffer,
                );
            };

            if !fading {
                draw_text(
                    &format!("SCORE: {:.0}", model.score),
                    vec2(-3.0, -3.0),
                    0.7,
                    vec2(0.5, 1.0),
                );
                draw_text(
                    &format!("HIGHSCORE: {:.0}", model.high_score),
                    vec2(-3.0, -4.0),
                    0.7,
                    vec2(0.5, 1.0),
                );

                // Leaderboard
                match &model.leaderboard {
                    LeaderboardState::None => {
                        draw_text("LEADERBOARD", vec2(4.0, 0.5), 0.8, vec2(0.5, 0.5));
                        draw_text("NOT AVAILABLE", vec2(4.0, -0.5), 0.8, vec2(0.5, 0.5));
                    }
                    LeaderboardState::Pending => {
                        let mut pos = vec2(4.0, 2.5);
                        draw_text("LEADERBOARD", pos, 0.8, vec2(0.5, 1.0));
                        pos.y -= 0.8;
                        draw_text("LOADING...", pos, 0.7, vec2(0.5, 1.0));
                    }
                    LeaderboardState::Failed => {
                        let mut pos = vec2(4.0, 2.5);
                        draw_text("LEADERBOARD", pos, 0.8, vec2(0.5, 1.0));
                        pos.y -= 0.8;
                        draw_text("FAILED TO LOAD", pos, 0.7, vec2(0.5, 1.0));
                    }
                    LeaderboardState::Ready(leaderboard) => {
                        let mut pos = vec2(4.0, 2.5);
                        draw_text("LEADERBOARD", pos, 0.8, vec2(0.5, 1.0));
                        pos.y -= 0.8;
                        {
                            let text = if let Some(place) = leaderboard.my_position {
                                format!("{} PLACE", place + 1)
                            } else if let State::Lost { .. } = model.state {
                                "FINISH TO COMPETE".to_string()
                            } else if model.player.name.trim().is_empty() {
                                "CANNOT SUBMIT WITHOUT A NAME".to_string()
                            } else {
                                "".to_string()
                            };
                            draw_text(&text, pos, 0.7, vec2(0.5, 1.0));
                            pos.y -= 0.7;
                        }
                        for score in &leaderboard.top10 {
                            let font_size = 0.6;
                            draw_text(&score.player, pos, font_size, vec2(1.0, 1.0));
                            draw_text(
                                &format!(" - {:.0}", score.score),
                                pos,
                                font_size,
                                vec2(0.0, 1.0),
                            );
                            pos.y -= font_size;
                        }
                    }
                }
            }
        } else {
            self.util.draw_text(
                format!("SCORE: {:.0}", model.score),
                vec2(0.0, 4.5).as_r32(),
                TextRenderOptions::new(0.7).color(THEME.light),
                camera,
                &mut framebuffer,
            );
        }

        self.util
            .draw_player(&model.player, camera, &mut framebuffer);

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
            self.util.draw_health(
                &model.player.health,
                model.player.get_lit_state(),
                // &model.config.theme,
                &mut framebuffer,
            );
        }

        self.dither.finish(model.real_time, theme);

        let aabb = Aabb2::ZERO.extend_positive(old_framebuffer.size().as_f32());
        geng_utils::texture::DrawTexture::new(self.dither.get_buffer())
            .fit(aabb, vec2(0.5, 0.5))
            .draw(&geng::PixelPerfectCamera, &self.geng, old_framebuffer);
    }

    pub fn draw_ui(&mut self, _model: &Model, _framebuffer: &mut ugli::Framebuffer) {
        // let camera = &geng::PixelPerfectCamera;
        // let screen = Aabb2::ZERO.extend_positive(framebuffer.size().as_f32());

        // let font_size = screen.height() * 0.05;

        // if let State::Playing = model.state {
        //     self.util.draw_health(&model.player.health, &model.config.theme, framebuffer);
        // }
    }
}
