use super::{mask::MaskedRender, ui::UiRender, *};

use crate::menu::{MenuState, MenuUI};

pub struct MenuRender {
    geng: Geng,
    assets: Rc<Assets>,
    // util: UtilRender,
    masked: MaskedRender,
    ui: UiRender,
}

impl MenuRender {
    pub fn new(geng: &Geng, assets: &Rc<Assets>) -> Self {
        Self {
            geng: geng.clone(),
            assets: assets.clone(),
            // util: UtilRender::new(geng, assets),
            masked: MaskedRender::new(geng, assets, vec2(1, 1)),
            ui: UiRender::new(geng, assets),
        }
    }

    pub fn draw_ui(&mut self, ui: &MenuUI, state: &MenuState, framebuffer: &mut ugli::Framebuffer) {
        self.masked.update_size(framebuffer.size());

        let font_size = framebuffer.size().y as f32 * 0.04;
        let camera = &geng::PixelPerfectCamera;
        let theme = &state.options.theme;

        self.ui.draw_texture(
            ui.ctl_logo.position,
            &self.assets.sprites.title,
            theme.light,
            framebuffer,
        );

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

            let (bg_color, fg_color) = if group.selected_time.get_ratio() > 0.5 {
                (theme.light, theme.dark)
            } else {
                (theme.dark, theme.light)
            };

            let width = font_size * 0.2;
            self.ui.draw_quad(
                group.state.position.extend_uniform(-width),
                bg_color,
                &mut mask.color,
            );

            self.ui
                .draw_text_widget_colored(&group.name, fg_color, &mut mask.color);
            self.ui
                .draw_text_widget_colored(&group.author, fg_color, &mut mask.color);

            self.ui
                .draw_outline(group.state.position, width, theme.light, &mut mask.color);
        }

        for level in &ui.levels {
            if !level.state.visible {
                continue;
            }

            let (bg_color, fg_color) = if level.selected_time.get_ratio() > 0.5 {
                (theme.light, theme.dark)
            } else {
                (theme.dark, theme.light)
            };

            let width = font_size * 0.2;
            self.ui.draw_quad(
                level.state.position.extend_uniform(-width),
                bg_color,
                &mut mask.color,
            );

            self.ui
                .draw_text_widget_colored(&level.name, fg_color, &mut mask.color);
            self.ui
                .draw_text_widget_colored(&level.author, fg_color, &mut mask.color);

            self.ui
                .draw_outline(level.state.position, width, theme.light, &mut mask.color);
        }

        self.masked.draw(draw_parameters(), framebuffer);

        {
            // Options
            let mut buffer = self.masked.start();

            let head = ui.options_head.state.position;
            let options = ui.options.state.position;

            buffer.mask_quad(head);
            buffer.mask_quad(options);

            self.ui.draw_quad(head, theme.dark, &mut buffer.color);
            self.ui.draw_quad(options, theme.dark, &mut buffer.color);

            {
                // Volume
                let volume = &ui.options.volume;
                self.ui.draw_text_widget(&volume.title, &mut buffer.color);
                self.ui
                    .draw_slider_widget(&volume.master, &mut buffer.color);
            }

            {
                // Palette
                let palette = &ui.options.palette;
                self.ui.draw_text_widget(&palette.title, &mut buffer.color);
                for palette in &palette.palettes {
                    self.ui.draw_text_widget(&palette.name, &mut buffer.color);

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
                    self.ui.draw_outline(
                        palette.visual.position.extend_uniform(outline_width),
                        outline_width,
                        theme.light,
                        &mut buffer.color,
                    );
                }
            }

            // Outline
            let width = 15.0;
            self.ui.draw_outline(
                head.extend_up(2.0 * width),
                width,
                theme.light,
                &mut buffer.color,
            );
            self.ui.draw_outline(
                options.extend_up(width),
                width,
                theme.light,
                &mut buffer.color,
            );
            self.ui.draw_quad(
                head.extend_uniform(-width).extend_up(width * 3.0),
                theme.dark,
                &mut buffer.color,
            );

            self.ui
                .draw_text_widget(&ui.options_head, &mut buffer.color);

            self.masked.draw(draw_parameters(), framebuffer);

            // let outline_width = font_size * 0.2;
            // let [bl, br, tr, tl] = options
            //     .extend_uniform(-outline_width / 2.0)
            //     .extend_up(outline_width)
            //     .corners();
            // let head = head.extend_uniform(-outline_width / 2.0);
            // let head_tl = vec2(head.min.x, bl.y);
            // let head_tr = vec2(head.max.x, br.y);
            // let chain = Chain::new(vec![
            //     tl,
            //     bl,
            //     head_tl,
            //     head.bottom_left(),
            //     head.bottom_right(),
            //     head_tr,
            //     br,
            //     tr,
            // ]);
            // self.geng.draw2d().draw2d(
            //     framebuffer,
            //     camera,
            //     &draw2d::Chain::new(chain, outline_width, theme.light, 1),
            // );
        }

        self.ui
            .draw_leaderboard(&ui.leaderboard, theme, &mut self.masked, framebuffer);

        {
            // Level Config
            self.geng.draw2d().draw2d(
                framebuffer,
                camera,
                &draw2d::Quad::new(ui.level_config.state.position, theme.dark),
            );

            let mut buffer = self.masked.start();
            buffer.mask_quad(ui.level_config.state.position);

            self.ui
                .draw_close_button(&ui.level_config.close, theme, &mut buffer.color);

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
                self.ui
                    .draw_toggle_button(tab, active, false, theme, &mut buffer.color);
            }
            self.ui.draw_quad(
                ui.level_config.separator.position,
                theme.light,
                &mut buffer.color,
            );

            if ui.level_config.difficulty.state.visible {
                for preset in &ui.level_config.difficulty.presets {
                    self.ui.draw_toggle_button(
                        &preset.button.text,
                        preset.selected,
                        false,
                        theme,
                        &mut buffer.color,
                    );
                }
            }
            if ui.level_config.mods.state.visible {
                for preset in &ui.level_config.mods.mods {
                    self.ui.draw_toggle_button(
                        &preset.button.text,
                        preset.selected,
                        true,
                        theme,
                        &mut buffer.color,
                    );
                }
            }

            self.masked.draw(draw_parameters(), framebuffer);

            self.ui.draw_outline(
                ui.level_config.state.position,
                font_size * 0.2,
                theme.light,
                framebuffer,
            );
        }
    }
}
