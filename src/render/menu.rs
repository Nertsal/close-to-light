use super::{mask::MaskedRender, ui::UiRender, *};

use crate::menu::{MenuState, MenuUI};

pub struct MenuRender {
    geng: Geng,
    assets: Rc<Assets>,
    // util: UtilRender,
    masked: MaskedRender,
    ui: UiRender,
    font_size: f32,
}

impl MenuRender {
    pub fn new(geng: &Geng, assets: &Rc<Assets>) -> Self {
        Self {
            geng: geng.clone(),
            assets: assets.clone(),
            // util: UtilRender::new(geng, assets),
            masked: MaskedRender::new(geng, assets, vec2(1, 1)),
            ui: UiRender::new(geng, assets),
            font_size: 1.0,
        }
    }

    pub fn draw_ui(&mut self, ui: &MenuUI, state: &MenuState, framebuffer: &mut ugli::Framebuffer) {
        self.masked.update_size(framebuffer.size());
        self.font_size = framebuffer.size().y as f32 * 0.04;

        let theme = state.options.theme;

        self.ui.draw_texture(
            ui.ctl_logo.position,
            &self.assets.sprites.title,
            theme.light,
            framebuffer,
        );

        self.draw_levels(ui, state, framebuffer);

        // TODO: better ordering solution
        if state.show_profile.time.is_above_min() {
            self.draw_options(ui, state, framebuffer);
            self.draw_profile(ui, state, framebuffer);
        } else {
            self.draw_profile(ui, state, framebuffer);
            self.draw_options(ui, state, framebuffer);
        }

        self.ui
            .draw_leaderboard(&ui.leaderboard, theme, &mut self.masked, framebuffer);
        self.draw_level_config(ui, state, framebuffer);
    }

    fn draw_levels(&mut self, ui: &MenuUI, state: &MenuState, framebuffer: &mut ugli::Framebuffer) {
        let theme = state.options.theme;

        // Clip groups and levels
        let mut mask = self.masked.start();

        mask.mask_quad(ui.groups_state.position);
        mask.mask_quad(ui.levels_state.position);

        for (group, entry) in ui.groups.iter().zip(&state.groups) {
            if let Some(logo) = &entry.logo {
                self.ui
                    .draw_texture(group.logo.position, logo, theme.light, framebuffer);
            }

            self.ui.draw_toggle_slide(
                &group.state,
                &[&group.name, &group.author],
                self.font_size * 0.2,
                group.selected_time.get_ratio() > 0.5,
                theme,
                &mut mask.color,
            );
        }

        for level in &ui.levels {
            if !level.state.visible {
                continue;
            }

            self.ui.draw_toggle_slide(
                &level.state,
                &[&level.name, &level.author],
                self.font_size * 0.2,
                level.selected_time.get_ratio() > 0.5,
                theme,
                &mut mask.color,
            );
        }

        self.masked.draw(draw_parameters(), framebuffer);
    }

    fn draw_profile(
        &mut self,
        ui: &MenuUI,
        state: &MenuState,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        let theme = state.options.theme;
        let width = 12.0;
        let head = ui.profile_head.state.position;
        let profile = ui.profile.position.extend_up(width);

        self.ui.draw_window(
            profile,
            Some(head),
            width,
            theme,
            framebuffer,
            |_framebuffer| {},
        );

        self.ui.draw_icon(&ui.profile_head, framebuffer);
    }

    fn draw_options(
        &mut self,
        ui: &MenuUI,
        state: &MenuState,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        let camera = &geng::PixelPerfectCamera;
        let theme = state.options.theme;

        let width = 12.0;
        let head = ui.options_head.state.position;
        let options = ui.options.state.position.extend_up(width);

        self.ui.draw_window(
            options,
            Some(head),
            width,
            theme,
            framebuffer,
            |framebuffer| {
                {
                    // Volume
                    let volume = &ui.options.volume;
                    self.ui.draw_text(&volume.title, framebuffer);
                    self.ui.draw_slider(&volume.master, framebuffer);
                }

                {
                    // Palette
                    let palette = &ui.options.palette;
                    self.ui.draw_text(&palette.title, framebuffer);
                    for palette in &palette.palettes {
                        self.ui.draw_text(&palette.name, framebuffer);

                        let mut quad = |i: f32, color: Color| {
                            let pos = palette.visual.position;
                            let pos = Aabb2::point(pos.bottom_left())
                                .extend_positive(vec2::splat(pos.height()));
                            let pos = pos.translate(vec2(i * pos.width(), 0.0));
                            self.geng.draw2d().draw2d(
                                framebuffer,
                                camera,
                                &draw2d::Quad::new(pos, color),
                            );
                        };
                        quad(0.0, palette.palette.dark);
                        quad(1.0, palette.palette.light);
                        quad(2.0, palette.palette.danger);
                        quad(3.0, palette.palette.highlight);

                        let outline_width = self.font_size * 0.1;
                        self.ui.draw_outline(
                            palette.visual.position.extend_uniform(outline_width),
                            outline_width,
                            theme.light,
                            framebuffer,
                        );
                    }
                }
            },
        );

        self.ui.draw_text(&ui.options_head, framebuffer);
    }

    fn draw_level_config(
        &mut self,
        ui: &MenuUI,
        state: &MenuState,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        let theme = state.options.theme;

        self.ui.draw_window(
            ui.level_config.state.position,
            None,
            self.font_size * 0.2,
            theme,
            framebuffer,
            |framebuffer| {
                self.ui
                    .draw_close_button(&ui.level_config.close, theme, framebuffer);

                {
                    let (tab, active) = (
                        &ui.level_config.tab_mods,
                        ui.level_config.mods.state.visible,
                    );
                    self.ui
                        .draw_toggle_button(tab, active, false, theme, framebuffer);
                }
                self.ui
                    .draw_quad(ui.level_config.separator.position, theme.light, framebuffer);

                if ui.level_config.mods.state.visible {
                    for preset in &ui.level_config.mods.mods {
                        self.ui.draw_toggle_button(
                            &preset.button.text,
                            preset.selected,
                            true,
                            theme,
                            framebuffer,
                        );
                    }
                }
            },
        );
    }
}
