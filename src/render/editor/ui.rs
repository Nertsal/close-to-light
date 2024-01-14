use super::*;

impl EditorRender {
    pub(super) fn draw_ui(&mut self, editor: &Editor, ui: &EditorUI) {
        let framebuffer =
            &mut geng_utils::texture::attach_texture(&mut self.ui_texture, self.geng.ugli());
        let theme = &editor.model.options.theme;

        let camera = &geng::PixelPerfectCamera;
        ugli::clear(framebuffer, Some(Color::TRANSPARENT_BLACK), None, None);

        let font_size = ui.screen.position.height() * 0.04;
        let options = TextRenderOptions::new(font_size).align(vec2(0.5, 1.0));

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

        if ui.config.state.visible {
            self.draw_tab_config(editor, &ui.config);
        }

        if ui.edit.state.visible {
            self.draw_tab_edit(editor, &ui.edit, options);

            let framebuffer =
                &mut geng_utils::texture::attach_texture(&mut self.ui_texture, self.geng.ugli());

            // Game border
            let width = 5.0;
            self.util.draw_outline(
                &Collider::aabb(ui.game.position.extend_uniform(width).map(r32)),
                width,
                Color::WHITE,
                camera,
                framebuffer,
            );
        }
    }

    fn draw_tab_config(&mut self, _editor: &Editor, ui: &EditorConfigWidget) {
        if !ui.state.visible {
            return;
        }

        let framebuffer =
            &mut geng_utils::texture::attach_texture(&mut self.ui_texture, self.geng.ugli());

        self.ui.draw_text(&ui.timing, framebuffer);
        self.ui.draw_value(&ui.bpm, framebuffer);
        self.ui.draw_value(&ui.offset, framebuffer);

        self.ui.draw_text(&ui.music, framebuffer);
        self.ui.draw_text(&ui.level, framebuffer);

        self.ui.draw_text(&ui.timeline, framebuffer);
        self.ui.draw_value(&ui.scroll_by, framebuffer);
        self.ui.draw_value(&ui.shift_scroll, framebuffer);
        self.ui.draw_value(&ui.alt_scroll, framebuffer);
    }

    fn draw_tab_edit(
        &mut self,
        editor: &Editor,
        ui: &EditorEditWidget,
        options: TextRenderOptions,
    ) {
        if !ui.state.visible {
            return;
        }

        let framebuffer =
            &mut geng_utils::texture::attach_texture(&mut self.ui_texture, self.geng.ugli());

        let camera = &geng::PixelPerfectCamera;
        let font_size = options.size;

        // Event
        self.ui.draw_text(&ui.new_event, framebuffer);
        self.ui.draw_button(&ui.new_palette, framebuffer);
        self.ui.draw_button(&ui.new_circle, framebuffer);
        self.ui.draw_button(&ui.new_line, framebuffer);

        // View
        self.ui.draw_text(&ui.view, framebuffer);
        self.ui
            .draw_checkbox(&ui.visualize_beat, options, framebuffer);
        self.ui.draw_checkbox(&ui.show_grid, options, framebuffer);
        self.ui.draw_value(&ui.view_zoom, framebuffer);

        // Placement
        self.ui.draw_text(&ui.placement, framebuffer);
        self.ui.draw_checkbox(&ui.snap_grid, options, framebuffer);
        self.ui.draw_value(&ui.grid_size, framebuffer);

        // Light
        self.ui.draw_text(&ui.light, framebuffer);
        self.ui
            .draw_checkbox(&ui.light_danger, options, framebuffer);
        self.ui.draw_value(&ui.light_fade_in, framebuffer);
        self.ui.draw_value(&ui.light_fade_out, framebuffer);

        // Waypoints
        self.ui.draw_button(&ui.waypoint, framebuffer);
        self.ui.draw_value(&ui.waypoint_scale, framebuffer);
        self.ui.draw_value(&ui.waypoint_angle, framebuffer);

        {
            // Timeline
            self.ui.draw_text(&ui.current_beat, framebuffer);

            let mut quad = |aabb, color| self.geng.draw2d().quad(framebuffer, camera, aabb, color);
            let timeline = ui.timeline.state.position;
            let line = Aabb2::point(timeline.center())
                .extend_symmetric(vec2(timeline.width(), font_size * 0.1) / 2.0);
            quad(line, Color::WHITE);

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

            // Light timespan
            let event = if let State::Waypoints { event, .. } = editor.state {
                Some(event)
            } else {
                editor.selected_light.map(|id| id.event)
            };
            if let Some(event) = event.and_then(|i| editor.level.level.events.get(i)) {
                let from = event.beat;
                if let Event::Light(event) = &event.event {
                    let from = from + event.telegraph.precede_time;
                    let to = from + event.light.movement.total_duration();

                    let from = ui.timeline.time_to_screen(from);
                    let to = ui.timeline.time_to_screen(to);
                    let timespan = Aabb2::point(from)
                        .extend_right(to.x - from.x)
                        .extend_symmetric(vec2(0.0, 0.2 * font_size) / 2.0);
                    quad(timespan, Color::CYAN);
                }
            }

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
            quad(ui.timeline.current_beat.position, Color::WHITE);
        }
    }
}
