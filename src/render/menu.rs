use super::{mask::MaskedRender, util::UtilRender, *};

use crate::menu::{MenuState, MenuUI};

pub struct MenuRender {
    geng: Geng,
    assets: Rc<Assets>,
    util: UtilRender,
    masked: MaskedRender,
}

impl MenuRender {
    pub fn new(geng: &Geng, assets: &Rc<Assets>) -> Self {
        Self {
            geng: geng.clone(),
            assets: assets.clone(),
            util: UtilRender::new(geng, assets),
            masked: MaskedRender::new(geng, assets, vec2(1, 1)),
        }
    }

    pub fn draw_ui(&mut self, ui: &MenuUI, state: &MenuState, framebuffer: &mut ugli::Framebuffer) {
        self.masked.update_size(framebuffer.size());

        let font_size = framebuffer.size().y as f32 * 0.04;
        let camera = &geng::PixelPerfectCamera;
        let theme = &state.options.theme;

        geng_utils::texture::DrawTexture::new(&self.assets.sprites.title)
            .fit_height(ui.ctl_logo.position, 0.5)
            .colored(theme.light)
            .draw(camera, &self.geng, framebuffer);

        // Clip groups and levels
        let mut mask = self.masked.start();

        mask.mask_quad(ui.groups_state.position);
        mask.mask_quad(ui.levels_state.position);

        for (group, entry) in ui.groups.iter().zip(&state.groups) {
            if let Some(logo) = &entry.logo {
                self.geng.draw2d().textured_quad(
                    framebuffer,
                    camera,
                    group.logo.position,
                    logo,
                    theme.light,
                );
            }
            self.util.draw_text_widget(&group.name, &mut mask.color);
            self.util.draw_text_widget(&group.author, &mut mask.color);

            let group = &group.state;
            let color = if group.pressed {
                theme.light.map_rgb(|x| x * 0.5)
            } else if group.hovered {
                theme.light.map_rgb(|x| x * 0.7)
            } else {
                theme.light
            };
            self.util.draw_outline(
                &Collider::aabb(group.position.map(r32)),
                font_size * 0.2,
                color,
                camera,
                &mut mask.color,
            );
        }

        for level in &ui.levels {
            self.util.draw_text_widget(&level.name, &mut mask.color);
            self.util.draw_text_widget(&level.author, &mut mask.color);

            let level = &level.state;
            let color = if level.pressed {
                theme.light.map_rgb(|x| x * 0.5)
            } else if level.hovered {
                theme.light.map_rgb(|x| x * 0.7)
            } else {
                theme.light
            };
            self.util.draw_outline(
                &Collider::aabb(level.position.map(r32)),
                font_size * 0.2,
                color,
                camera,
                &mut mask.color,
            );
        }

        self.masked.draw(
            ugli::DrawParameters {
                blend_mode: Some(ugli::BlendMode::straight_alpha()),
                ..default()
            },
            framebuffer,
        );

        {
            // Options
            let mut buffer = self.masked.start();

            let head = ui.options_head.state.position;
            let options = ui.options.state.position;

            buffer.mask_quad(head);
            buffer.mask_quad(options);

            self.geng.draw2d().draw2d(
                &mut buffer.color,
                camera,
                &draw2d::Quad::new(head, theme.dark),
            );
            self.geng.draw2d().draw2d(
                &mut buffer.color,
                camera,
                &draw2d::Quad::new(options, theme.dark),
            );

            {
                // Volume
                let volume = &ui.options.volume;
                self.util.draw_text_widget(&volume.title, &mut buffer.color);
                self.util
                    .draw_slider_widget(&volume.master, &mut buffer.color);

                self.util
                    .draw_text_widget(&ui.options_head, &mut buffer.color);
            }

            {
                // Palette
                let palette = &ui.options.palette;
                self.util
                    .draw_text_widget(&palette.title, &mut buffer.color);
                for palette in &palette.palettes {
                    self.util.draw_text_widget(&palette.name, &mut buffer.color);

                    let mut quad = |i: f32, color: Color| {
                        let pos = palette.visual.position;
                        let pos = Aabb2::point(pos.bottom_left())
                            .extend_positive(vec2::splat(pos.height()));
                        let pos = pos.translate(vec2(i * pos.width(), 0.0));
                        self.geng.draw2d().draw2d(
                            &mut buffer.color,
                            camera,
                            &draw2d::Quad::new(pos, color),
                        );
                    };
                    quad(0.0, palette.palette.dark);
                    quad(1.0, palette.palette.light);
                    quad(2.0, palette.palette.danger);

                    let outline_width = font_size * 0.1;
                    self.util.draw_outline(
                        &Collider::aabb(
                            palette
                                .visual
                                .position
                                .extend_uniform(outline_width)
                                .map(r32),
                        ),
                        outline_width,
                        theme.light,
                        camera,
                        &mut buffer.color,
                    );
                }
            }

            self.masked.draw(
                ugli::DrawParameters {
                    blend_mode: Some(ugli::BlendMode::straight_alpha()),
                    ..default()
                },
                framebuffer,
            );

            // Outline
            let outline_width = font_size * 0.2;
            let [bl, br, tr, tl] = options
                .extend_uniform(-outline_width / 2.0)
                .extend_up(outline_width)
                .corners();
            let head = head.extend_uniform(-outline_width / 2.0);
            let head_tl = vec2(head.min.x, bl.y);
            let head_tr = vec2(head.max.x, br.y);
            let chain = Chain::new(vec![
                tl,
                bl,
                head_tl,
                head.bottom_left(),
                head.bottom_right(),
                head_tr,
                br,
                tr,
            ]);
            self.geng.draw2d().draw2d(
                framebuffer,
                camera,
                &draw2d::Chain::new(chain, outline_width, theme.light, 1),
            );
        }

        {
            // Leaderboard
            self.geng.draw2d().draw2d(
                framebuffer,
                camera,
                &draw2d::Quad::new(ui.leaderboard.state.position, theme.dark),
            );
            self.util
                .draw_button_widget(&ui.leaderboard.close, framebuffer);
            self.util
                .draw_text_widget(&ui.leaderboard.title, framebuffer);
            self.util
                .draw_text_widget(&ui.leaderboard.subtitle, framebuffer);
            self.util
                .draw_text_widget(&ui.leaderboard.status, framebuffer);
            for row in &ui.leaderboard.rows {
                self.util.draw_text_widget(&row.rank, framebuffer);
                self.util.draw_text_widget(&row.player, framebuffer);
                self.util.draw_text_widget(&row.score, framebuffer);
            }

            self.util.draw_outline(
                &Collider::aabb(ui.leaderboard.state.position.map(r32)),
                font_size * 0.2,
                theme.light,
                camera,
                framebuffer,
            );
        }

        {
            // Level Config
            self.geng.draw2d().draw2d(
                framebuffer,
                camera,
                &draw2d::Quad::new(ui.level_config.state.position, theme.dark),
            );

            let mut buffer = self.masked.start();
            buffer.mask_quad(ui.level_config.state.position);

            self.util
                .draw_button_widget(&ui.level_config.close, &mut buffer.color);

            for (tab, active) in [
                (
                    &ui.level_config.tab_difficulty,
                    ui.level_config.difficulty.state.visible,
                ),
                (
                    &ui.level_config.tab_mods,
                    ui.level_config.mods.state.visible,
                ),
            ] {
                let options = &tab.options;
                let color = if tab.state.pressed {
                    options.press_color
                } else if tab.state.hovered {
                    options.hover_color
                } else {
                    options.color
                };
                self.util.draw_text(
                    &tab.text,
                    geng_utils::layout::aabb_pos(tab.state.position, tab.options.align),
                    tab.options.color(color),
                    &geng::PixelPerfectCamera,
                    &mut buffer.color,
                );

                if active {
                    // Underline
                    let mut line = tab
                        .state
                        .position
                        .extend_symmetric(-vec2(0.5, 0.1) * options.size);
                    line.max.y = line.min.y + 0.05 * options.size;
                    self.geng.draw2d().draw2d(
                        &mut buffer.color,
                        camera,
                        &draw2d::Quad::new(line, color),
                    );
                }
            }

            if ui.level_config.difficulty.state.visible {
                for preset in &ui.level_config.difficulty.presets {
                    let mut button = preset.button.clone();
                    button.text.state.pressed = preset.selected;
                    self.util.draw_button_widget(&button, &mut buffer.color);
                }
            }
            if ui.level_config.mods.state.visible {
                for preset in &ui.level_config.mods.mods {
                    let mut button = preset.button.clone();
                    button.text.state.pressed = preset.selected;
                    self.util.draw_button_widget(&button, &mut buffer.color);
                }
            }

            self.masked.draw(
                ugli::DrawParameters {
                    blend_mode: Some(ugli::BlendMode::straight_alpha()),
                    ..default()
                },
                framebuffer,
            );

            self.util.draw_outline(
                &Collider::aabb(ui.level_config.state.position.map(r32)),
                font_size * 0.2,
                theme.light,
                camera,
                framebuffer,
            );
        }
    }
}
