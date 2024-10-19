use crate::ui::layout::AreaOps;

use super::*;

impl EditorRender {
    pub(super) fn draw_ui(&mut self, editor: &Editor, ui: &EditorUI) {
        let framebuffer =
            &mut geng_utils::texture::attach_texture(&mut self.ui_texture, self.geng.ugli());
        let theme = editor.context.get_options().theme;
        self.font_size = framebuffer.size().y as f32 * 0.04;

        let camera = &geng::PixelPerfectCamera;
        ugli::clear(framebuffer, Some(Color::TRANSPARENT_BLACK), None, None);

        let font_size = ui.screen.position.height() * 0.04;

        if ui.config.state.visible {
            self.draw_tab_config(editor, &ui.config);
        }

        if ui.edit.state.visible {
            let framebuffer =
                &mut geng_utils::texture::attach_texture(&mut self.ui_texture, self.geng.ugli());

            // Game border
            let width = 5.0;
            self.util.draw_outline(
                &Collider::aabb(ui.game.position.extend_uniform(width).map(r32)),
                width,
                theme.light,
                camera,
                framebuffer,
            );

            self.draw_tab_edit(editor, &ui.edit);
        }

        let framebuffer =
            &mut geng_utils::texture::attach_texture(&mut self.ui_texture, self.geng.ugli());

        self.ui.draw_toggle_button(
            &ui.tab_edit.text,
            ui.edit.state.visible,
            false,
            theme,
            framebuffer,
        );
        self.ui.draw_toggle_button(
            &ui.tab_config.text,
            ui.config.state.visible,
            false,
            theme,
            framebuffer,
        );

        self.ui.draw_button(&ui.exit, theme, framebuffer);

        if ui.help_text.state.visible {
            let width = font_size * 0.1;
            let pos = Aabb2::from_corners(
                ui.help.state.position.top_left() + vec2(-1.0, 1.0) * 2.0 * width,
                ui.help_text.state.position.bottom_right(),
            );
            self.ui.draw_quad(pos, theme.dark, framebuffer);
            self.ui.draw_outline(pos, width, theme.light, framebuffer);
        }
        self.ui.draw_icon(&ui.help, theme, framebuffer);
        self.ui.draw_text(&ui.help_text, framebuffer);

        self.ui.draw_text(&ui.unsaved, framebuffer);
        self.ui.draw_button(&ui.save, theme, framebuffer);

        if let Some(ui) = &ui.confirm {
            self.ui.draw_confirm(
                ui,
                self.font_size * 0.2,
                editor.context.get_options().theme,
                &mut self.mask,
                framebuffer,
            );
        }
    }

    fn draw_tab_config(&mut self, editor: &Editor, ui: &EditorConfigWidget) {
        if !ui.state.visible {
            return;
        }

        let framebuffer =
            &mut geng_utils::texture::attach_texture(&mut self.ui_texture, self.geng.ugli());
        let theme = editor.context.get_options().theme;

        self.ui.draw_text(&ui.timing, framebuffer);
        self.ui.draw_value(&ui.bpm, framebuffer);
        self.ui.draw_value(&ui.offset, framebuffer);

        self.ui.draw_text(&ui.music, framebuffer);
        self.ui.draw_text(&ui.level, framebuffer);
        self.ui.draw_input(&ui.level_name, framebuffer);
        self.ui.draw_button(&ui.level_delete, theme, framebuffer);
        self.ui.draw_button(&ui.level_create, theme, framebuffer);
        self.ui.draw_text(&ui.all_levels, framebuffer);

        let active = editor
            .level_edit
            .as_ref()
            .map(|editor| editor.static_level.level_index);
        for (i, (up, down, level)) in ui.all_level_names.iter().enumerate() {
            let selected = active == Some(i);
            self.ui.draw_icon(up, theme, framebuffer);
            self.ui.draw_icon(down, theme, framebuffer);
            self.ui
                .draw_toggle_button(level, selected, false, theme, framebuffer);
        }

        self.ui.draw_text(&ui.timeline, framebuffer);
        self.ui.draw_value(&ui.scroll_by, framebuffer);
        self.ui.draw_value(&ui.shift_scroll, framebuffer);
        self.ui.draw_value(&ui.alt_scroll, framebuffer);
    }

    fn draw_tab_edit(&mut self, editor: &Editor, ui: &EditorEditWidget) {
        if !ui.state.visible {
            return;
        }

        let framebuffer =
            &mut geng_utils::texture::attach_texture(&mut self.ui_texture, self.geng.ugli());

        let Some(level_editor) = &editor.level_edit else {
            self.ui.draw_text(&ui.warn_select_level, framebuffer);
            return;
        };

        let theme = editor.context.get_options().theme;

        let camera = &geng::PixelPerfectCamera;

        // Event
        self.ui.draw_text(&ui.new_event, framebuffer);
        // self.ui.draw_button(&ui.new_palette, theme, framebuffer);
        self.ui.draw_button(&ui.new_circle, theme, framebuffer);
        self.ui.draw_button(&ui.new_line, theme, framebuffer);
        self.ui.draw_button(&ui.new_waypoint, theme, framebuffer);

        // View
        self.ui.draw_text(&ui.view, framebuffer);
        self.ui
            .draw_checkbox(&ui.show_only_selected, theme, framebuffer);
        self.ui
            .draw_checkbox(&ui.visualize_beat, theme, framebuffer);
        self.ui.draw_checkbox(&ui.show_grid, theme, framebuffer);
        self.ui.draw_value(&ui.view_zoom, framebuffer);

        // Placement
        self.ui.draw_text(&ui.placement, framebuffer);
        self.ui.draw_checkbox(&ui.snap_grid, theme, framebuffer);
        self.ui.draw_value(&ui.grid_size, framebuffer);

        // Light
        self.ui.draw_text(&ui.light, framebuffer);
        self.ui.draw_button(&ui.light_delete, theme, framebuffer);
        self.ui.draw_checkbox(&ui.light_danger, theme, framebuffer);
        self.ui.draw_value(&ui.light_fade_in, framebuffer);
        self.ui.draw_value(&ui.light_fade_out, framebuffer);

        // Waypoints
        self.ui.draw_button(&ui.waypoint, theme, framebuffer);
        self.ui
            .draw_icon_button(&ui.prev_waypoint, theme, framebuffer);
        self.ui
            .draw_icon_button(&ui.next_waypoint, theme, framebuffer);
        self.ui.draw_text(&ui.current_waypoint, framebuffer);
        self.ui.draw_button(&ui.waypoint_delete, theme, framebuffer);
        self.ui.draw_value(&ui.waypoint_scale, framebuffer);
        self.ui.draw_value(&ui.waypoint_angle, framebuffer);

        self.ui.draw_dropdown(
            &ui.waypoint_curve,
            self.font_size * 0.2,
            &mut self.mask,
            framebuffer,
        );
        self.ui.draw_dropdown(
            &ui.waypoint_interpolation,
            self.font_size * 0.2,
            &mut self.mask,
            framebuffer,
        );

        {
            // Timeline
            self.ui.draw_text(&ui.current_beat, framebuffer);

            let mut quad = |aabb, color| self.geng.draw2d().quad(framebuffer, camera, aabb, color);
            let bar = ui.timeline.bar.position;
            quad(bar, theme.light);

            if ui.timeline.left.visible {
                // Selected area
                let from = ui.timeline.left.position;
                let to = if ui.timeline.right.visible {
                    ui.timeline.right.position
                } else {
                    ui.timeline.current_beat.position
                };
                quad(
                    Aabb2::point(from.center())
                        .extend_right(to.center().x - from.center().x)
                        .extend_symmetric(vec2(0.0, from.height() * 0.8) / 2.0),
                    Color::GRAY,
                );
            }

            if ui.timeline.selected.visible {
                quad(ui.timeline.selected.position, theme.highlight);
            }

            let triangle = |aabb: Aabb2<f32>, color, framebuffer: &mut ugli::Framebuffer| {
                let vertices = [
                    aabb.align_pos(vec2(0.5, 1.0)),
                    aabb.bottom_left(),
                    aabb.bottom_right(),
                ]
                .map(|a_pos| draw2d::ColoredVertex {
                    a_pos,
                    a_color: Rgba::WHITE,
                });
                self.geng.draw2d().draw(
                    framebuffer,
                    camera,
                    &vertices,
                    color,
                    ugli::DrawMode::Triangles,
                );
            };
            let diamond = |aabb: Aabb2<f32>, color, framebuffer: &mut ugli::Framebuffer| {
                let vertices = [
                    aabb.align_pos(vec2(0.0, 0.5)),
                    aabb.align_pos(vec2(0.5, 0.0)),
                    aabb.align_pos(vec2(1.0, 0.5)),
                    aabb.align_pos(vec2(0.5, 1.0)),
                ]
                .map(|a_pos| draw2d::ColoredVertex {
                    a_pos,
                    a_color: Rgba::WHITE,
                });
                self.geng.draw2d().draw(
                    framebuffer,
                    camera,
                    &vertices,
                    color,
                    ugli::DrawMode::TriangleFan,
                );
            };

            // All lights
            for (id, light) in ui.timeline.lights.values().flatten() {
                let color = if level_editor.selected_light == Some(*id) {
                    theme.highlight
                } else if level_editor.level.events.get(id.event).map_or(
                    false,
                    |event| match &event.event {
                        Event::Light(light) => light.light.danger,
                        _ => false,
                    },
                ) {
                    theme.danger
                } else {
                    theme.light
                };
                triangle(light.position, color, framebuffer);
            }
            // Waypoints
            for (_, waypoint) in &ui.timeline.waypoints {
                diamond(waypoint.position, theme.highlight, framebuffer);
            }

            // // Selected light timespan
            // let event = if let State::Waypoints { event, .. } = level_editor.state {
            //     Some(event)
            // } else {
            //     level_editor.selected_light.map(|id| id.event)
            // };
            // if let Some(event) = event.and_then(|i| level_editor.level.events.get(i)) {
            //     let from_time = event.beat;
            //     if let Event::Light(event) = &event.event {
            //         let from_time = from_time + event.telegraph.precede_time;
            //         let to_time = from_time + event.light.movement.total_duration();

            //         let from = ui.timeline.time_to_screen(from_time);
            //         let to = ui.timeline.time_to_screen(to_time);
            //         let timespan = Aabb2::point(from)
            //             .extend_right(to.x - from.x)
            //             .extend_symmetric(vec2(0.0, 0.2 * font_size) / 2.0);
            //         quad(timespan, theme.highlight);

            //         for (_, _, time) in event.light.movement.timed_positions() {
            //             let time = from_time + time;
            //             let point = ui.timeline.time_to_screen(time);
            //             let timespan =
            //                 Aabb2::point(point).extend_symmetric(vec2(0.05, 0.4) * font_size / 2.0);
            //             quad(timespan, theme.highlight);
            //         }
            //     }
            // }

            let mut quad = |aabb, color| self.geng.draw2d().quad(framebuffer, camera, aabb, color);

            // Selected bounds
            if ui.timeline.left.visible {
                quad(ui.timeline.left.position, Color::GRAY);
            }
            if ui.timeline.right.visible {
                quad(ui.timeline.right.position, Color::GRAY);
            }

            if ui.timeline.replay.visible {
                quad(
                    ui.timeline.replay.position,
                    Color::try_from("#aaa").unwrap(),
                );
            }
            quad(ui.timeline.current_beat.position, theme.light);
        }

        // Tooltip
        if ui.tooltip.state.visible {
            let position = ui.tooltip.state.position;
            let width = self.font_size * 0.1;
            self.ui.fill_quad(
                position.extend_uniform(width / 2.0),
                theme.dark,
                framebuffer,
            );
            self.ui.draw_text(&ui.tooltip.title, framebuffer);
            self.ui.draw_text(&ui.tooltip.text, framebuffer);
            self.ui
                .draw_outline(position, width, theme.light, framebuffer);
        }
    }
}
