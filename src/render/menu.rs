use super::{mask::MaskedRender, ui::UiRender, *};

use crate::{
    menu::{MenuState, MenuUI},
    ui::layout::AreaOps,
};

pub struct MenuRender {
    geng: Geng,
    // assets: Rc<Assets>,
    // util: UtilRender,
    masked: MaskedRender,
    masked2: MaskedRender, // TODO: have just one somehow maybe
    ui: UiRender,
    font_size: f32,
}

impl MenuRender {
    pub fn new(geng: &Geng, assets: &Rc<Assets>) -> Self {
        Self {
            geng: geng.clone(),
            // assets: assets.clone(),
            // util: UtilRender::new(geng, assets),
            masked: MaskedRender::new(geng, assets, vec2(1, 1)),
            masked2: MaskedRender::new(geng, assets, vec2(1, 1)),
            ui: UiRender::new(geng, assets),
            font_size: 1.0,
        }
    }

    pub fn draw_ui(&mut self, ui: &MenuUI, state: &MenuState, framebuffer: &mut ugli::Framebuffer) {
        self.masked.update_size(framebuffer.size());
        self.masked2.update_size(framebuffer.size());
        self.font_size = framebuffer.size().y as f32 * 0.04;

        let theme = state.options.theme;

        self.ui.draw_icon(&ui.ctl_logo, framebuffer);

        self.ui
            .draw_quad(ui.separator.position, theme.light, framebuffer);

        self.draw_levels(ui, state, framebuffer);
        self.draw_play_level(ui, state, framebuffer);
        self.draw_modifiers(ui, state, framebuffer);
        self.draw_options(ui, state, framebuffer);
        self.draw_explore(ui, state, framebuffer);

        // // TODO: better ordering solution
        // if ui.panels.profile.window.show.time.is_above_min() {
        //     self.draw_options(ui, state, framebuffer);
        //     self.draw_explore(ui, state, framebuffer);
        //     self.draw_profile(ui, state, framebuffer);
        // } else if ui.panels.explore.window.show.time.is_above_min() {
        //     self.draw_profile(ui, state, framebuffer);
        //     self.draw_options(ui, state, framebuffer);
        //     self.draw_explore(ui, state, framebuffer);
        // } else {
        //     self.draw_profile(ui, state, framebuffer);
        //     self.draw_explore(ui, state, framebuffer);
        //     self.draw_options(ui, state, framebuffer);
        // }

        // self.ui
        //     .draw_leaderboard(&ui.leaderboard, theme, &mut self.masked, framebuffer);
        // self.draw_level_config(ui, state, framebuffer);

        self.draw_sync(ui, state, framebuffer);
    }

    fn draw_sync(&mut self, ui: &MenuUI, state: &MenuState, framebuffer: &mut ugli::Framebuffer) {
        let Some(sync) = &ui.sync else { return };
        let t = sync.window.show.time.get_ratio();

        let window = sync.state.position;
        let min_height = self.font_size * 2.0;
        let height = (t * window.height()).max(min_height);

        let window = window.with_height(height, 1.0);
        self.ui.draw_window(
            &mut self.masked,
            window,
            None,
            self.font_size * 0.2,
            state.options.theme,
            framebuffer,
            |framebuffer| {
                let hold = sync.hold.position;
                let hold = hold.extend_up(self.font_size * 0.2 - hold.height());
                self.ui
                    .draw_quad(hold, state.options.theme.light, framebuffer);

                self.ui.draw_icon(&sync.close.icon, framebuffer);
                self.ui.draw_text(&sync.title, framebuffer);
                self.ui.draw_text(&sync.status, framebuffer);

                self.ui.draw_toggle(
                    &sync.upload,
                    self.font_size * 0.2,
                    state.options.theme,
                    framebuffer,
                );
                self.ui.draw_toggle(
                    &sync.discard,
                    self.font_size * 0.2,
                    state.options.theme,
                    framebuffer,
                );

                self.ui.draw_text(&sync.response, framebuffer);
            },
        );
    }

    fn draw_levels(&mut self, ui: &MenuUI, state: &MenuState, framebuffer: &mut ugli::Framebuffer) {
        let ui = &ui.level_select;
        let theme = state.options.theme;

        self.ui
            .draw_toggle_widget(&ui.tab_music, theme, framebuffer);
        self.ui
            .draw_toggle_widget(&ui.tab_groups, theme, framebuffer);
        self.ui
            .draw_toggle_widget(&ui.tab_levels, theme, framebuffer);

        if ui.tab_music.selected {
            self.draw_item_widget(&ui.add_music, false, 0.5, theme, framebuffer);
            for music in &ui.grid_music {
                let selected =
                    state.selected_music.as_ref().map(|m| m.data) == Some(music.music.meta.id);
                self.draw_item_widget(&music.text, selected, 1.0, theme, framebuffer);
            }
        } else if ui.tab_groups.selected {
            self.draw_item_widget(&ui.add_group, false, 0.5, theme, framebuffer);
            for group in &ui.grid_groups {
                let selected = state.selected_group.as_ref().map(|m| m.data) == Some(group.index);
                self.draw_item_widget(&group.text, selected, 1.0, theme, framebuffer);
            }
        } else if ui.tab_levels.selected {
            for level in &ui.grid_levels {
                let selected = state.selected_level.as_ref().map(|m| m.data) == Some(level.index);
                self.draw_item_widget(&level.text, selected, 1.0, theme, framebuffer);
                self.draw_item_menu(&level.menu, theme, framebuffer);
            }
        }

        self.ui
            .draw_quad(ui.separator.position, theme.light, framebuffer);

        // // Clip groups and levels
        // let mut mask = self.masked.start();

        // mask.mask_quad(Aabb2 {
        //     min: ui.groups_state.position.min,
        //     max: ui.levels_state.position.max,
        // });

        // for (group, _entry) in ui.sets.iter().zip(&state.local.borrow().groups) {
        //     // TODO: logo?
        //     // if let Some(logo) = &entry.logo {
        //     //     self.ui
        //     //         .draw_texture(group.logo.position, logo, theme.light, framebuffer);
        //     // }

        //     self.ui.draw_icon(&group.delete.icon, &mut mask.color);
        //     self.ui.draw_outline(
        //         group.static_state.position,
        //         self.font_size * 0.2,
        //         theme.light,
        //         &mut mask.color,
        //     );

        //     self.ui.draw_toggle_slide(
        //         &group.state,
        //         &[&group.name, &group.author],
        //         self.font_size * 0.2,
        //         group.selected_time.get_ratio() > 0.5,
        //         theme,
        //         &mut mask.color,
        //     );
        // }

        // for level in &ui.difficulties {
        //     if !level.state.visible {
        //         continue;
        //     }

        //     self.ui.draw_icon(&level.sync.icon, &mut mask.color);
        //     self.ui.draw_icon(&level.edit.icon, &mut mask.color);
        //     self.ui.draw_outline(
        //         level.static_state.position,
        //         self.font_size * 0.2,
        //         theme.light,
        //         &mut mask.color,
        //     );

        //     self.ui.draw_toggle_slide(
        //         &level.state,
        //         &[&level.name, &level.author],
        //         self.font_size * 0.2,
        //         level.selected_time.get_ratio() > 0.5,
        //         theme,
        //         &mut mask.color,
        //     );
        // }

        // self.ui
        //     .draw_toggle(&ui.new_group, self.font_size * 0.2, theme, &mut mask.color);
        // self.ui
        //     .draw_toggle(&ui.new_level, self.font_size * 0.2, theme, &mut mask.color);

        // self.masked.draw(draw_parameters(), framebuffer);
    }

    fn draw_play_level(
        &mut self,
        ui: &MenuUI,
        state: &MenuState,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        let ui = &ui.play_level;
        let theme = state.options.theme;

        self.ui.draw_text(&ui.music, framebuffer);
        self.ui.draw_text(&ui.music_author, framebuffer);
        self.ui
            .draw_text_colored(&ui.difficulty, theme.highlight, framebuffer);
        self.ui
            .draw_text_colored(&ui.mappers, theme.highlight, framebuffer);
    }

    fn draw_modifiers(
        &mut self,
        ui: &MenuUI,
        state: &MenuState,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        let ui = &ui.modifiers;
        let theme = state.options.theme;
        let width = self.font_size * 0.2;

        if !ui.body.visible {
            self.ui
                .draw_outline(ui.head.state.position, width, theme.danger, framebuffer);
            self.ui
                .draw_text_colored(&ui.head, theme.danger, framebuffer);
        } else {
            let theme = Theme {
                light: theme.danger,
                ..theme
            };
            self.ui.draw_window(
                &mut self.masked,
                ui.body.position,
                Some(ui.head.state.position),
                width,
                theme,
                framebuffer,
                |framebuffer| {
                    for (modifier, _) in &ui.mods {
                        self.ui.draw_toggle_widget(modifier, theme, framebuffer);
                    }
                    self.ui
                        .draw_quad(ui.separator.position, theme.light, framebuffer);
                    for line in &ui.description {
                        self.ui.draw_text_colored(line, theme.light, framebuffer);
                    }
                },
            );
            self.ui
                .draw_text_colored(&ui.head, theme.danger, framebuffer);
        }
    }

    fn draw_options(
        &mut self,
        ui: &MenuUI,
        state: &MenuState,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        let ui = &ui.options;
        let camera = &geng::PixelPerfectCamera;
        let theme = state.options.theme;

        let width = 12.0;
        let options = ui.options.state.position.extend_up(width);

        if ui.open_time.is_above_min() {
            self.ui.draw_window(
                &mut self.masked,
                options,
                None,
                width,
                theme,
                framebuffer,
                |framebuffer| {
                    {
                        // Profile
                        let ui = &ui.options.profile;
                        self.ui.draw_text(&ui.offline, framebuffer);

                        let register = &ui.register;
                        if register.state.visible {
                            // self.ui.draw_input(&register.username, framebuffer);
                            // self.ui.draw_input(&register.password, framebuffer);
                            // self.ui.draw_button(&register.login, framebuffer);
                            // self.ui.draw_button(&register.register, framebuffer);
                            self.ui.draw_text(&register.login_with, framebuffer);
                            self.ui.draw_icon(&register.discord.icon, framebuffer);
                        }

                        let logged = &ui.logged;
                        if logged.state.visible {
                            self.ui.draw_text(&logged.username, framebuffer);
                            self.ui.draw_toggle_button(
                                &logged.logout,
                                false,
                                false,
                                theme,
                                framebuffer,
                            );
                        }
                    }

                    self.ui
                        .draw_quad(ui.options.separator.position, theme.light, framebuffer);

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
        }

        let transparency = crate::util::smoothstep(1.0 - ui.open_time.get_ratio());
        let mut color = theme.light;
        color.a = transparency;
        self.ui.draw_icon_colored(&ui.button, color, framebuffer);
        self.ui.draw_outline(
            ui.button.state.position,
            self.font_size * 0.1,
            color,
            framebuffer,
        );
    }

    fn draw_explore(
        &mut self,
        ui: &MenuUI,
        state: &MenuState,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        let ui = &ui.explore;
        if !ui.state.visible {
            return;
        };

        let theme = state.options.theme;

        let width = 12.0;
        // let head = ui.explore_head.state.position;
        let explore = ui.state.position; //.extend_up(width);

        self.ui.draw_window(
            &mut self.masked,
            explore,
            // Some(head),
            None,
            width,
            theme,
            framebuffer,
            |framebuffer| {
                for (tab, active) in [
                    (&ui.tab_music, ui.music.state.visible),
                    (&ui.tab_levels, ui.levels.state.visible),
                ] {
                    self.ui
                        .draw_toggle_button(tab, active, false, theme, framebuffer);
                }
                self.ui.draw_icon(&ui.close.icon, framebuffer);
                self.ui
                    .draw_quad(ui.separator.position, theme.light, framebuffer);

                let mut mask = self.masked2.start();

                if ui.music.state.visible {
                    mask.mask_quad(ui.music.items_state.position);
                    self.ui.draw_text(&ui.music.status, &mut mask.color);
                    for item in &ui.music.items {
                        self.ui.draw_icon(&item.download.icon, &mut mask.color);
                        self.ui.draw_icon(&item.play.icon, &mut mask.color);
                        self.ui.draw_icon(&item.goto.icon, &mut mask.color);
                        self.ui.draw_text(&item.name, &mut mask.color);
                        self.ui.draw_text(&item.author, &mut mask.color);
                        self.ui.draw_outline(
                            item.state.position,
                            self.font_size * 0.2,
                            theme.light,
                            &mut mask.color,
                        );
                    }
                } else if ui.levels.state.visible {
                    mask.mask_quad(ui.levels.items_state.position);
                    self.ui.draw_text(&ui.levels.status, &mut mask.color);
                    for item in &ui.levels.items {
                        self.ui.draw_icon(&item.download.icon, &mut mask.color);
                        self.ui.draw_icon(&item.goto.icon, &mut mask.color);
                        self.ui.draw_text(&item.name, &mut mask.color);
                        self.ui.draw_text(&item.author, &mut mask.color);
                        self.ui.draw_outline(
                            item.state.position,
                            self.font_size * 0.2,
                            theme.light,
                            &mut mask.color,
                        );
                    }
                }

                self.masked2.draw(draw_parameters(), framebuffer);
            },
        );

        // self.ui.draw_text(&ui.explore_head, framebuffer);
    }

    fn draw_level_config(
        &mut self,
        ui: &MenuUI,
        state: &MenuState,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        let theme = state.options.theme;

        self.ui.draw_window(
            &mut self.masked,
            ui.level_config.state.position,
            None,
            self.font_size * 0.2,
            theme,
            framebuffer,
            |framebuffer| {
                self.ui.draw_icon(&ui.level_config.close.icon, framebuffer);

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

    fn draw_item_widget(
        &mut self,
        text: &crate::ui::widget::TextWidget,
        selected: bool,
        width: f32,
        theme: Theme,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        let (bg_color, fg_color, out_color) = if selected {
            (theme.light, theme.dark, theme.light)
        } else if text.state.hovered {
            (theme.light, theme.dark, theme.dark)
        } else {
            (theme.dark, theme.light, theme.light)
        };
        let outline_width = self.font_size * 0.2 * width;
        self.ui.draw_quad(
            text.state.position.extend_uniform(-outline_width),
            bg_color,
            framebuffer,
        );
        self.ui.draw_text_colored(text, fg_color, framebuffer);
        self.ui
            .draw_outline(text.state.position, outline_width, out_color, framebuffer);
    }

    fn draw_item_menu(
        &mut self,
        menu: &crate::menu::ItemMenuWidget,
        theme: Theme,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        if !menu.state.visible {
            return;
        }

        let position = menu.state.position;
        let t = menu.window.show.time.get_ratio();
        let t = crate::util::smoothstep(t);
        let position = position.extend_down(-position.height() * (1.0 - t));
        if position.height() < 1.0 {
            return;
        }

        let outline_width = self.font_size * 0.2;
        self.ui.draw_window(
            &mut self.masked,
            position,
            None,
            outline_width,
            theme,
            framebuffer,
            |framebuffer| {
                self.ui.draw_icon(&menu.sync.icon, framebuffer);
                self.ui.draw_icon(&menu.edit.icon, framebuffer);
                self.ui.draw_icon(&menu.delete.icon, framebuffer);
            },
        );
    }
}
