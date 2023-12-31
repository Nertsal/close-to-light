use super::*;

impl EditorRender {
    pub(super) fn draw_ui(
        &mut self,
        editor: &Editor,
        ui: &EditorUI,
        _render_options: &RenderOptions,
    ) {
        let screen_buffer =
            &mut geng_utils::texture::attach_texture(&mut self.ui_texture, self.geng.ugli());
        let theme = &editor.model.options.theme;

        let framebuffer_size = screen_buffer.size().as_f32();
        let camera = &geng::PixelPerfectCamera;
        ugli::clear(screen_buffer, Some(theme.dark), None, None);

        let font_size = ui.screen.position.height() * 0.04;
        let options = TextRenderOptions::new(font_size).align(vec2(0.5, 1.0));

        {
            // Level info
            let pos = vec2(
                ui.level_info.position.center().x,
                ui.level_info.position.max.y,
            );
            self.util
                .draw_text("Level", pos, options, camera, screen_buffer);

            let pos = pos - vec2(0.0, font_size);
            let name = editor
                .level_path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("<invalid>");
            self.util
                .draw_text(name, pos, options, camera, screen_buffer);
        }

        {
            // General
            for widget in [&ui.visualize_beat, &ui.show_grid, &ui.snap_grid] {
                self.ui.draw_checkbox(widget, options, screen_buffer);
            }
        }

        self.ui.draw_button_widget(&ui.new_palette, screen_buffer);
        self.ui.draw_button_widget(&ui.new_light, screen_buffer);

        if ui.new_selector.visible {
            // Selector
            for widget in [&ui.new_circle, &ui.new_line] {
                let light = &widget.light;
                let mut dither_buffer = self.dither_small.start();
                let collider = Collider::new(vec2::ZERO, light.shape);
                let color = if widget.state.hovered {
                    editor.config.theme.hover
                } else {
                    THEME.light
                };
                self.util.draw_light(
                    &collider,
                    color,
                    &Camera2d {
                        center: vec2::ZERO,
                        rotation: Angle::ZERO,
                        fov: 3.0,
                    },
                    &mut dither_buffer,
                );
                self.dither_small
                    .finish(editor.real_time, &theme.transparent());

                let size = ui.light_size.as_f32();
                let pos = geng_utils::layout::aabb_pos(widget.state.position, vec2(0.5, 0.5));
                let aabb = Aabb2::point(pos).extend_symmetric(size / 2.0);
                self.geng.draw2d().textured_quad(
                    screen_buffer,
                    camera,
                    aabb,
                    self.dither_small.get_buffer(),
                    Color::WHITE,
                );
            }
        }

        if ui.selected_text.state.visible {
            // Selected light
            let pos = geng_utils::layout::aabb_pos(ui.selected_text.state.position, vec2(0.5, 1.0));
            self.util.draw_text(
                &ui.selected_text.text,
                pos,
                options.size(font_size * 0.7),
                camera,
                screen_buffer,
            );
        }

        if ui.selected_light.light.state.visible {
            let light = &ui.selected_light.light.light;
            let mut dither_buffer = self.dither_small.start();
            let mut collider = Collider::new(vec2::ZERO, light.shape);
            collider.rotation = light.movement.initial.rotation;
            let color = if light.danger {
                THEME.danger
            } else {
                THEME.light
            };
            self.util.draw_light(
                &collider,
                color,
                &Camera2d {
                    center: vec2::ZERO,
                    rotation: Angle::ZERO,
                    fov: 3.0,
                },
                &mut dither_buffer,
            );
            self.dither_small
                .finish(editor.real_time, &theme.transparent());

            let size = ui.light_size.as_f32();
            let pos = geng_utils::layout::aabb_pos(
                ui.selected_light.light.state.position,
                vec2(0.5, 1.0),
            );
            let pos = pos - vec2(0.0, font_size + size.y / 2.0);
            let aabb = Aabb2::point(pos).extend_symmetric(size / 2.0);
            self.geng.draw2d().textured_quad(
                screen_buffer,
                camera,
                aabb,
                self.dither_small.get_buffer(),
                Color::WHITE,
            );

            let options = options.align(vec2(0.0, 0.5));
            self.ui
                .draw_checkbox(&ui.selected_light.danger, options, screen_buffer);
            self.ui
                .draw_text_widget(&ui.selected_light.fade_in, screen_buffer);
            self.ui
                .draw_text_widget(&ui.selected_light.fade_out, screen_buffer);
            self.ui
                .draw_text_widget(&ui.selected_light.scale, screen_buffer);
        }

        {
            // Timeline
            self.ui.draw_text_widget(&ui.current_beat, screen_buffer);

            let mut quad =
                |aabb, color| self.geng.draw2d().quad(screen_buffer, camera, aabb, color);
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
            if let Some(event) = event.and_then(|i| editor.level.events.get(i)) {
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

        // Leave the game area transparent
        ugli::draw(
            screen_buffer,
            &self.assets.shaders.solid,
            ugli::DrawMode::TriangleFan,
            &self.unit_quad,
            (
                ugli::uniforms! {
                    u_model_matrix: mat3::translate(ui.game.position.center()) * mat3::scale(ui.game.position.size() / 2.0),
                    u_color: Color::TRANSPARENT_BLACK,
                },
                camera.uniforms(framebuffer_size),
            ),
            ugli::DrawParameters {
                blend_mode: Some(BlendMode::combined(ChannelBlendMode {
                    src_factor: BlendFactor::One,
                    dst_factor: BlendFactor::Zero,
                    equation: BlendEquation::Add,
                })),
                ..default()
            },
        );

        // Game border
        let width = 5.0;
        self.util.draw_outline(
            &Collider::aabb(ui.game.position.extend_uniform(width).map(r32)),
            width,
            Color::WHITE,
            camera,
            screen_buffer,
        );
    }
}
