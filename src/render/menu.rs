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

        geng_utils::texture::DrawTexture::new(&self.assets.sprites.title)
            .fit_height(ui.ctl_logo.position, 0.5)
            .draw(camera, &self.geng, framebuffer);

        // Clip groups and levels
        let mut mask = self.masked.start();

        self.geng.draw2d().draw2d(
            &mut mask.mask,
            camera,
            &draw2d::Quad::new(ui.groups_state.position, Color::WHITE),
        );
        self.geng.draw2d().draw2d(
            &mut mask.mask,
            camera,
            &draw2d::Quad::new(ui.levels_state.position, Color::WHITE),
        );

        for (group, entry) in ui.groups.iter().zip(&state.groups) {
            if let Some(logo) = &entry.logo {
                self.geng.draw2d().textured_quad(
                    framebuffer,
                    camera,
                    group.logo.position,
                    logo,
                    Color::WHITE,
                );
            }
            self.util.draw_text_widget(&group.name, &mut mask.color);
            self.util.draw_text_widget(&group.author, &mut mask.color);

            let group = &group.state;
            let color = if group.pressed {
                state.config.theme.light.map_rgb(|x| x * 0.5)
            } else if group.hovered {
                state.config.theme.light.map_rgb(|x| x * 0.7)
            } else {
                state.config.theme.light
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
                state.config.theme.light.map_rgb(|x| x * 0.5)
            } else if level.hovered {
                state.config.theme.light.map_rgb(|x| x * 0.7)
            } else {
                state.config.theme.light
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
            // Leaderboard
            self.geng.draw2d().draw2d(
                framebuffer,
                camera,
                &draw2d::Quad::new(ui.leaderboard.state.position, state.config.theme.dark),
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
                state.config.theme.light,
                camera,
                framebuffer,
            );
        }

        {
            // Level Config
            self.geng.draw2d().draw2d(
                framebuffer,
                camera,
                &draw2d::Quad::new(ui.level_config.state.position, state.config.theme.dark),
            );

            let mut buffer = self.masked.start();
            self.geng.draw2d().draw2d(
                &mut buffer.mask,
                camera,
                &draw2d::Quad::new(ui.level_config.state.position, Color::WHITE),
            );

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
                state.config.theme.light,
                camera,
                framebuffer,
            );
        }

        // if ui.play_group.state.visible {
        //     self.util.draw_outline(
        //         &Collider::aabb(ui.play_group.state.position.map(r32)),
        //         font_size * 0.2,
        //         state.config.theme.light,
        //         camera,
        //         framebuffer,
        //     );

        //     for level in &ui.play_group.levels {
        //         self.util.draw_button_widget(&level.play, framebuffer);
        //         self.util.draw_text_widget(&level.credits, framebuffer);
        //     }

        //     // Title clip
        //     let mut buffer = self.masked.start();
        //     buffer.mask_quad(ui.play_group.title);
        //     for title in &ui.play_group.config_titles {
        //         self.util.draw_text_widget(title, &mut buffer.color);
        //     }
        //     self.masked.draw(
        //         ugli::DrawParameters {
        //             blend_mode: Some(ugli::BlendMode::straight_alpha()),
        //             ..default()
        //         },
        //         framebuffer,
        //     );

        //     self.util
        //         .draw_button_widget(&ui.play_group.prev_config, framebuffer);
        //     self.util
        //         .draw_button_widget(&ui.play_group.next_config, framebuffer);

        //     // Main clip
        //     let mut buffer = self.masked.start();
        //     buffer.mask_quad(ui.play_group.main);

        //     for config in &ui.play_group.configs {
        //         if !config.state.visible {
        //             continue;
        //         }
        //         match &config.configuring {
        //             Configuring::Palette { presets } => {
        //                 for preset in presets {
        //                     let mut button = preset.button.clone();
        //                     button.text.state.pressed = preset.selected;
        //                     self.util.draw_button_widget(&button, &mut buffer.color);

        //                     // Palette
        //                     let (_, theme_preview) =
        //                         crate::ui::layout::split_top_down(button.text.state.position, 0.5);
        //                     let theme_preview = theme_preview
        //                         .extend_symmetric(vec2(0.0, -theme_preview.height() / 4.0));
        //                     let theme_preview = crate::ui::layout::fit_aabb_height(
        //                         vec2(5.0, 2.0),
        //                         theme_preview,
        //                         0.5,
        //                     );

        //                     let theme = &preset.preset;
        //                     let theme = [theme.dark, theme.light, theme.danger];
        //                     let layout = crate::ui::layout::split_columns(theme_preview, 3);
        //                     for (color, pos) in theme.into_iter().zip(layout) {
        //                         self.geng.draw2d().draw2d(
        //                             &mut buffer.color,
        //                             &geng::PixelPerfectCamera,
        //                             &draw2d::Quad::new(pos, color),
        //                         );
        //                     }

        //                     self.util.draw_outline(
        //                         &Collider::aabb(theme_preview.map(r32)),
        //                         5.0,
        //                         Color::WHITE,
        //                         &geng::PixelPerfectCamera,
        //                         &mut buffer.color,
        //                     );
        //                 }
        //             }
        //             Configuring::Health { presets } => {
        //                 for preset in presets {
        //                     let mut button = preset.button.clone();
        //                     button.text.state.pressed = preset.selected;
        //                     self.util.draw_button_widget(&button, &mut buffer.color);
        //                 }
        //             }
        //             Configuring::Modifiers { presets } => {
        //                 for preset in presets {
        //                     let mut button = preset.button.clone();
        //                     button.text.state.pressed = preset.selected;
        //                     self.util.draw_button_widget(&button, &mut buffer.color);
        //                 }
        //             }
        //         }
        //     }

        //     self.masked.draw(
        //         ugli::DrawParameters {
        //             blend_mode: Some(ugli::BlendMode::straight_alpha()),
        //             ..default()
        //         },
        //         framebuffer,
        //     );

        //     // geng_utils::texture::DrawTexture::new(self.masked.color_texture())
        //     //     .fit_screen(vec2(0.5, 0.5), framebuffer)
        //     //     .draw(&geng::PixelPerfectCamera, &self.geng, framebuffer)
        // }
    }
}
