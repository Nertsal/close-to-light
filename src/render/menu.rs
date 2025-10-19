use ctl_ui::widget::WidgetState;

use super::{mask::MaskedRender, ui::UiRender, *};

use crate::{
    menu::{MenuState, MenuUI},
    ui::layout::AreaOps,
};

pub struct MenuRender {
    context: Context,
    // util: UtilRender,
    masked: MaskedRender,
    masked2: MaskedRender, // TODO: have just one somehow maybe
    ui: UiRender,
    font_size: f32,
}

impl MenuRender {
    pub fn new(context: Context) -> Self {
        Self {
            // util: UtilRender::new(geng, assets),
            masked: MaskedRender::new(&context.geng, &context.assets, vec2(1, 1)),
            masked2: MaskedRender::new(&context.geng, &context.assets, vec2(1, 1)),
            ui: UiRender::new(context.clone()),
            context,
            font_size: 1.0,
        }
    }

    pub fn draw_ui(&mut self, ui: &MenuUI, state: &MenuState, framebuffer: &mut ugli::Framebuffer) {
        self.masked.update_size(framebuffer.size());
        self.masked2.update_size(framebuffer.size());
        self.font_size = framebuffer.size().y as f32 * 0.04;

        let theme = state.context.get_options().theme;

        // self.ui.draw_icon(&ui.ctl_logo, theme, framebuffer);

        // self.ui
        //     .draw_quad(ui.separator.position, theme.light, framebuffer);

        self.draw_levels(ui, state, framebuffer);
        self.draw_play_level(ui, state, framebuffer);

        // Options button
        self.ui.draw_icon(&ui.options.button, theme, framebuffer);
        self.ui.draw_outline(
            ui.options.button.state.position,
            self.font_size * 0.1,
            theme.light,
            framebuffer,
        );

        self.draw_leaderboard(ui, state, framebuffer);
        self.draw_modifiers(ui, state, framebuffer);

        if ui.options.open_time.is_above_min() {
            self.draw_options(ui, state, framebuffer);
        }

        self.draw_explore(ui, state, framebuffer);
        self.draw_sync(ui, state, framebuffer);

        self.draw_item_widget(
            &ui.notifications.discard_all.state,
            &ui.notifications.discard_all,
            false,
            1.0,
            theme,
            framebuffer,
        );
        for notification in ui
            .notifications
            .items
            .iter()
            .chain(&ui.notifications.items_done)
        {
            self.ui.draw_notification(
                notification,
                self.font_size * 0.2,
                theme,
                &mut self.masked,
                framebuffer,
            );
        }

        if let Some(ui) = &ui.confirm {
            self.ui.draw_confirm(
                ui,
                self.font_size * 0.2,
                theme,
                &mut self.masked,
                framebuffer,
            );
        }
    }

    fn draw_sync(&mut self, ui: &MenuUI, state: &MenuState, framebuffer: &mut ugli::Framebuffer) {
        let Some(sync) = &ui.sync else { return };
        let theme = state.context.get_options().theme;
        let t = crate::util::smoothstep(sync.window.show.time.get_ratio());

        let window = sync.state.position;
        let min_height = self.font_size * 2.0;
        let height = (t * window.height()).max(min_height);

        let window = window.with_height(height, 1.0);
        self.ui.draw_window(
            &mut self.masked,
            window,
            None,
            self.font_size * 0.2,
            theme,
            framebuffer,
            |framebuffer| {
                let hold = sync.hold.position;
                let hold = hold.extend_up(self.font_size * 0.2 - hold.height());
                self.ui.draw_quad(hold, theme.light, framebuffer);

                self.ui.draw_icon(&sync.close.icon, theme, framebuffer);
                self.ui.draw_text(&sync.title, framebuffer);
                self.ui.draw_text(&sync.status, framebuffer);

                self.ui
                    .draw_toggle(&sync.upload, self.font_size * 0.2, theme, framebuffer);
                self.ui
                    .draw_toggle(&sync.discard, self.font_size * 0.2, theme, framebuffer);

                self.ui.draw_text(&sync.response, framebuffer);
            },
        );
    }

    fn draw_levels(&mut self, ui: &MenuUI, state: &MenuState, framebuffer: &mut ugli::Framebuffer) {
        let ui = &ui.level_select;
        let theme = state.context.get_options().theme;

        self.ui.draw_text(&ui.tab_levels, framebuffer);
        self.ui.draw_text(&ui.tab_diffs, framebuffer);

        // Levels
        for level in &ui.levels {
            self.ui.draw_icon(&level.edited, theme, framebuffer);
            self.ui.draw_icon(&level.local, theme, framebuffer);
            let selected = state.switch_level == Some(level.index);
            self.draw_item_widget(&level.state, &level.text, selected, 1.0, theme, framebuffer);
            for (diff, color) in &level.diffs {
                let mut pp_quad = |pos: Aabb2<f32>, color| {
                    let size = pos.size().map(|x| {
                        let mut x = x as usize;
                        if x % 2 == 1 {
                            x += 1;
                        }
                        x
                    });
                    let pos = geng_utils::pixel::pixel_perfect_aabb(
                        pos.center(),
                        vec2(0.5, 0.5),
                        size,
                        &geng::PixelPerfectCamera,
                        framebuffer.size().as_f32(),
                    );
                    self.ui.draw_quad(pos, color, framebuffer);
                };
                pp_quad(diff.position, theme.light);
                pp_quad(
                    diff.position.extend_uniform(-diff.position.height() * 0.2),
                    theme.get_color(*color),
                );
            }
            self.ui.draw_outline(
                level.state.position,
                self.font_size * 0.1,
                theme.light,
                framebuffer,
            );
        }

        // Context menu
        for level in &ui.levels {
            self.draw_item_menu(&level.menu, theme, framebuffer);
        }

        // Difficulty status/hint text
        self.ui.draw_text(&ui.no_level_selected, framebuffer);
        self.ui.draw_text(&ui.no_diffs, framebuffer);

        // Difficulties
        for diff in &ui.diffs {
            self.ui.draw_icon(&diff.edited, theme, framebuffer);
            self.ui.draw_icon(&diff.local, theme, framebuffer);
            let selected = state.switch_diff == Some(diff.index);
            self.draw_item_widget(&diff.state, &diff.text, selected, 1.0, theme, framebuffer);
            self.ui.draw_outline(
                diff.state.position,
                self.font_size * 0.1,
                theme.light,
                framebuffer,
            );

            self.ui.draw_icon(&diff.grade, theme, framebuffer);
        }

        // Context menu
        for diff in &ui.diffs {
            self.draw_item_menu(&diff.menu, theme, framebuffer);
        }

        self.ui
            .draw_quad(ui.separator_level.position, theme.light, framebuffer);
        self.ui
            .draw_quad(ui.separator_diff.position, theme.light, framebuffer);
        self.ui.draw_outline(
            ui.state.position,
            self.font_size * 0.5,
            theme.light,
            framebuffer,
        );
    }

    fn draw_play_level(
        &mut self,
        ui: &MenuUI,
        state: &MenuState,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        let ui = &ui.play_level;
        let theme = state.context.get_options().theme;

        self.ui.draw_text(&ui.music, framebuffer);
        self.ui.draw_text(&ui.music_author, framebuffer);
        if ui.music_original.state.visible {
            let pos = ui
                .music_original
                .state
                .position
                .extend_uniform(self.font_size * 0.2);
            self.ui.fill_quad(pos, theme.light, framebuffer);
            self.ui
                .draw_text_colored(&ui.music_original, theme.dark, framebuffer);
        }
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
        let theme = state.context.get_options().theme;
        let width = self.font_size * 0.2;

        let danger_theme = Theme {
            light: theme.danger,
            ..theme
        };

        for icon in &ui.active_mods {
            self.ui.draw_icon(icon, danger_theme, framebuffer);
        }

        if !ui.body.visible {
            self.ui
                .draw_outline(ui.head.state.position, width, theme.danger, framebuffer);
            self.ui
                .draw_text_colored(&ui.head, theme.danger, framebuffer);
        } else {
            let theme = danger_theme;
            self.ui.draw_window(
                &mut self.masked,
                ui.body.position,
                Some(ui.head.state.position),
                width,
                theme,
                framebuffer,
                |framebuffer| {
                    for widget in &ui.mods {
                        let state = &widget.state;
                        if !state.visible {
                            continue;
                        }

                        let (bg_color, fg_color) = if widget.selected {
                            (theme.light, theme.dark)
                        } else {
                            (theme.dark, theme.light)
                        };

                        let width = widget.text.options.size * 0.2;
                        let shrink = if state.hovered && widget.selected {
                            width
                        } else {
                            0.0
                        };
                        let pos = state.position.extend_uniform(-shrink);
                        self.ui
                            .draw_quad(pos.extend_uniform(-width), bg_color, framebuffer);
                        if state.hovered || widget.selected {
                            self.ui.draw_outline(pos, width, theme.light, framebuffer);
                        }
                        self.ui.util.draw_text(
                            &widget.text.text,
                            geng_utils::layout::aabb_pos(
                                widget.text.state.position,
                                widget.text.options.align,
                            ),
                            widget.text.options.color(fg_color),
                            &geng::PixelPerfectCamera,
                            framebuffer,
                        );
                        let mut theme = theme;
                        theme.light = fg_color;
                        theme.dark = bg_color;
                        self.ui.draw_icon(&widget.icon, theme, framebuffer);
                    }
                    self.ui
                        .draw_text_colored(&ui.score_multiplier, theme.light, framebuffer);
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
        let theme = state.context.get_options().theme;

        let width = 12.0;
        let options = ui
            .options
            .state
            .position
            .extend_positive(vec2::splat(width));

        self.ui.draw_window(
            &mut self.masked,
            options,
            None,
            width,
            theme,
            framebuffer,
            |framebuffer| {
                self.ui.draw_profile(&ui.options.profile, framebuffer);

                self.ui
                    .draw_quad(ui.options.separator.position, theme.light, framebuffer);

                {
                    // Volume
                    let volume = &ui.options.volume;
                    self.ui.draw_text(&volume.title, framebuffer);
                    self.ui.draw_slider(&volume.master, theme, framebuffer);
                }

                {
                    // Palette
                    let palette = &ui.options.palette;
                    self.ui.draw_text(&palette.title, framebuffer);
                    for palette in &palette.palettes {
                        let mut theme = theme;
                        if palette.state.hovered {
                            std::mem::swap(&mut theme.dark, &mut theme.light);
                            self.ui
                                .fill_quad(palette.state.position, theme.dark, framebuffer);
                        }

                        self.ui
                            .draw_text_colored(&palette.name, theme.light, framebuffer);

                        let mut quad = |i: f32, color: Color| {
                            let pos = palette.visual.position;
                            let pos = Aabb2::point(pos.bottom_left())
                                .extend_positive(vec2::splat(pos.height()));
                            let pos = pos.translate(vec2(i * pos.width(), 0.0));
                            self.context.geng.draw2d().draw2d(
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

                {
                    // Gameplay
                    let gameplay = &ui.options.gameplay;
                    self.ui.draw_text(&gameplay.title, framebuffer);
                    self.ui
                        .draw_slider(&gameplay.music_offset, theme, framebuffer);
                }

                {
                    // Graphics
                    let graphics = &ui.options.graphics;
                    self.ui.draw_text(&graphics.title, framebuffer);
                    self.ui
                        .draw_toggle_widget(&graphics.crt, theme, framebuffer);
                    self.ui
                        .draw_slider(&graphics.crt_scanlines, theme, framebuffer);
                    self.ui
                        .draw_toggle_widget(&graphics.telegraph_color, theme, framebuffer);
                }

                {
                    // Cursor
                    let cursor = &ui.options.cursor;
                    self.ui.draw_text(&cursor.title, framebuffer);
                    self.ui
                        .draw_toggle_widget(&cursor.show_perfect_radius, theme, framebuffer);
                    self.ui
                        .draw_slider(&cursor.inner_radius, theme, framebuffer);
                    self.ui
                        .draw_slider(&cursor.outer_radius, theme, framebuffer);
                }
            },
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

        let theme = state.context.get_options().theme;

        let width = 12.0;
        let explore = ui.state.position;

        self.ui.draw_window(
            &mut self.masked,
            explore,
            None,
            width,
            theme,
            framebuffer,
            |framebuffer| {
                self.ui.draw_icon(&ui.reload.icon, theme, framebuffer);
                self.ui.draw_icon(&ui.close.icon, theme, framebuffer);

                let mut mask = self.masked2.start();

                if ui.levels.state.visible {
                    mask.mask_quad(ui.levels.items_state.position);
                    self.ui.draw_text(&ui.levels.status, &mut mask.color);
                    for item in &ui.levels.items {
                        self.ui
                            .draw_icon_button(&item.download, theme, &mut mask.color);
                        self.ui.draw_icon(&item.downloading, theme, &mut mask.color);
                        self.ui.draw_icon(&item.goto.icon, theme, &mut mask.color);
                        self.ui
                            .draw_icon(&item.play_music.icon, theme, &mut mask.color);
                        self.ui
                            .draw_icon(&item.pause_music.icon, theme, &mut mask.color);
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
    }

    fn draw_leaderboard(
        &mut self,
        ui: &MenuUI,
        state: &MenuState,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        if !ui.leaderboard.state.visible {
            return;
        };

        let theme = state.context.get_options().theme;
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

    fn draw_item_widget(
        &mut self,
        state: &WidgetState,
        text: &crate::ui::widget::TextWidget,
        selected: bool,
        width: f32,
        theme: Theme,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        let (bg_color, fg_color, _out_color) = if selected {
            (theme.light, theme.dark, theme.light)
        } else if state.hovered {
            (theme.light, theme.dark, theme.dark)
        } else {
            (theme.dark, theme.light, theme.light)
        };
        let outline_width = self.font_size * 0.1 * width;
        self.ui
            .fill_quad_width(state.position, outline_width, bg_color, framebuffer);
        self.ui.draw_text_colored(text, fg_color, framebuffer);
        // self.ui
        //     .draw_outline(text.state.position, outline_width, out_color, framebuffer);
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
                self.ui.draw_icon(&menu.sync.icon, theme, framebuffer);
                self.ui.draw_icon(&menu.edit.icon, theme, framebuffer);
                self.ui.draw_icon(&menu.delete.icon, theme, framebuffer);
            },
        );
    }
}
