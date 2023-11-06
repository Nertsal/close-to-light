use super::*;

impl EditorRender {
    pub(super) fn draw_game(&mut self, editor: &Editor, options: &RenderOptions) {
        let game_buffer =
            &mut geng_utils::texture::attach_texture(&mut self.game_texture, self.geng.ugli());
        ugli::clear(
            game_buffer,
            Some(editor.level_state.relevant().config.theme.dark),
            None,
            None,
        );
        let screen_aabb = Aabb2::ZERO.extend_positive(game_buffer.size().as_f32());

        macro_rules! draw_game {
            ($alpha:expr) => {{
                self.dither.finish(editor.real_time, R32::ZERO);
                self.geng.draw2d().textured_quad(
                    game_buffer,
                    &geng::PixelPerfectCamera,
                    screen_aabb,
                    self.dither.get_buffer(),
                    crate::util::with_alpha(Color::WHITE, $alpha),
                );
                self.dither.start(Color::TRANSPARENT_BLACK)
            }};
        }

        // Level
        let light_color = editor.level.config.theme.light;
        let danger_color = editor.level.config.theme.danger;

        let active_danger = if let State::Movement { light, .. } = &editor.state {
            light.light.danger
        } else {
            false
        };

        let active_color = if active_danger {
            danger_color
        } else {
            light_color
        };

        let hover_color = editor.config.theme.hover;
        let hovered_event = editor.level_state.hovered_event();

        let select_color = editor.config.theme.select;
        let selected_event = editor.selected_light.map(|i| i.event);

        let get_color =
            |event_id: Option<usize>| -> Color {
                if let Some(event_id) = event_id {
                    let check = |a: Option<usize>| -> bool { a == Some(event_id) };
                    let base_color = if check(selected_event) {
                        select_color
                    } else if check(hovered_event) {
                        hover_color
                    } else {
                        light_color
                    };
                    let mod_color = if editor.level.events.get(event_id).map_or(false, |e| match &e
                        .event
                    {
                        Event::Light(event) => event.light.danger,
                        _ => false,
                    }) {
                        danger_color
                    } else {
                        base_color
                    };

                    let a = Hsva::<f32>::from(base_color);
                    let b = Hsva::<f32>::from(mod_color);
                    Color::from(Hsva {
                        h: (a.h + b.h) / 2.0,
                        s: (a.s + b.s) / 2.0,
                        v: (a.v + b.v) / 2.0,
                        a: (a.a + b.a) / 2.0,
                    })
                } else {
                    active_color
                }
            };

        let static_alpha = if let State::Place { .. }
        | State::Movement { .. }
        | State::Waypoints { .. } = editor.state
        {
            0.5
        } else {
            1.0
        };
        let dynamic_alpha = if editor.level_state.static_level.is_some() {
            0.5
        } else {
            1.0
        } * static_alpha;

        let draw_telegraph = |tele: &LightTelegraph, framebuffer: &mut ugli::Framebuffer| {
            let color = get_color(tele.light.event_id);
            self.util.draw_outline(
                &tele.light.collider,
                0.02,
                color,
                &editor.model.camera,
                framebuffer,
            );
        };
        let draw_light = |light: &Light, framebuffer: &mut ugli::Framebuffer| {
            let color = get_color(light.event_id);
            self.util
                .draw_collider(&light.collider, color, &editor.model.camera, framebuffer);
        };

        // Dynamic
        let mut pixel_buffer = self
            .dither
            .start(editor.level_state.relevant().config.theme.dark);

        if let Some(level) = &editor.level_state.dynamic_level {
            for tele in &level.telegraphs {
                draw_telegraph(tele, &mut pixel_buffer);
            }
            for light in &level.lights {
                draw_light(light, &mut pixel_buffer);
            }
        }

        let mut pixel_buffer = draw_game!(dynamic_alpha);

        if let Some(level) = &editor.level_state.static_level {
            for tele in &level.telegraphs {
                draw_telegraph(tele, &mut pixel_buffer);
            }
            for light in &level.lights {
                draw_light(light, &mut pixel_buffer);
            }
        }
        let mut pixel_buffer = draw_game!(static_alpha);

        if !options.hide_ui {
            let mut pixel_buffer = if let State::Movement {
                start_beat,
                ref light,
                ..
            } = editor.state
            {
                let time = editor.current_beat - start_beat
                    + light.light.movement.fade_in
                    + light.telegraph.precede_time;
                let draw_active = |time: Time, pixel_buffer: &mut ugli::Framebuffer| {
                    let event = commit_light(light.clone());
                    let (tele, light) = render_light(&event, time, None);
                    if let Some(tele) = tele {
                        draw_telegraph(&tele, pixel_buffer);
                    }
                    if let Some(light) = light {
                        draw_light(&light, pixel_buffer);
                    }
                };

                let mut pixel_buffer = if editor.visualize_beat {
                    // Active movement
                    let time = time + (editor.real_time / editor.level.beat_time()).fract();
                    draw_active(time, &mut pixel_buffer);
                    draw_game!(0.75)
                } else {
                    pixel_buffer
                };

                // Active static
                draw_active(time, &mut pixel_buffer);
                draw_game!(1.0)
            } else {
                pixel_buffer
            };

            // Current action
            let shape = match editor.state {
                State::Place { shape, danger } => Some((shape, danger)),
                State::Movement { ref light, .. } => Some((light.light.shape, light.light.danger)),
                _ => None,
            };
            if let Some((shape, danger)) = shape {
                let collider = Collider {
                    position: editor.cursor_world_pos,
                    rotation: editor.place_rotation,
                    shape: shape.scaled(editor.place_scale),
                };
                let color = if danger {
                    editor.level.config.theme.danger
                } else {
                    editor.level.config.theme.light
                };
                self.util.draw_outline(
                    &collider,
                    0.05,
                    color,
                    &editor.model.camera,
                    &mut pixel_buffer,
                );
            }
        }
        let mut pixel_buffer = draw_game!(1.0);

        if let State::Waypoints { event, .. } = &editor.state {
            let event = *event;
            if let Some(event) = editor.level.events.get(event) {
                if let Event::Light(event) = &event.event {
                    let color = if event.light.danger {
                        danger_color
                    } else {
                        light_color
                    };

                    // A dashed line moving through the waypoints to show general direction
                    let mut positions: Vec<vec2<f32>> = editor
                        .level_state
                        .waypoints
                        .iter()
                        .flat_map(|waypoints| &waypoints.points)
                        .map(|point| point.collider.position.as_f32())
                        .collect();
                    positions.dedup();
                    let options = util::DashRenderOptions {
                        width: 0.15,
                        color,
                        dash_length: 0.1,
                        space_length: 0.2,
                    };
                    if let Some(&to) = positions.get(1) {
                        let pos = positions.first_mut().unwrap();
                        let period = options.dash_length + options.space_length;
                        let speed = 1.0;
                        let t = ((editor.real_time.as_f32() * speed) / period).fract() * period;
                        *pos += (to - *pos).normalize_or_zero() * t;
                    }
                    let chain = Chain::new(positions);
                    self.util.draw_dashed_chain(
                        &chain,
                        &options,
                        &editor.model.camera,
                        &mut pixel_buffer,
                    );

                    if let Some(waypoints) = &editor.level_state.waypoints {
                        // Draw waypoints themselves
                        for (i, point) in waypoints.points.iter().enumerate() {
                            if !point.visible {
                                continue;
                            }
                            let color = if point.original == waypoints.selected {
                                select_color
                            } else if Some(i) == waypoints.hovered {
                                hover_color
                            } else {
                                color
                            };
                            self.util.draw_outline(
                                &point.collider,
                                0.05,
                                color,
                                &editor.model.camera,
                                &mut pixel_buffer,
                            );
                            self.util.draw_text(
                                format!("{}", i + 1),
                                point.collider.position,
                                TextRenderOptions::new(1.5),
                                &editor.model.camera,
                                &mut pixel_buffer,
                            )
                        }
                    }
                }
            }
        }
        draw_game!(1.0);

        if !options.hide_ui {
            // World UI
            let mut ui_buffer =
                geng_utils::texture::attach_texture(&mut self.ui_texture, self.geng.ugli());
            ugli::clear(&mut ui_buffer, Some(Rgba::TRANSPARENT_BLACK), None, None);

            // Grid
            if options.show_grid {
                let color = Rgba {
                    r: 0.7,
                    g: 0.7,
                    b: 0.7,
                    a: 0.7,
                };
                let grid_size = editor.grid_size.as_f32();
                let view = vec2(
                    editor.model.camera.fov * ui_buffer.size().as_f32().aspect(),
                    editor.model.camera.fov,
                )
                .map(|x| (x / 2.0 / grid_size).ceil() as i64);
                let thick = editor.config.grid.thick_every as i64;
                for x in -view.x..=view.x {
                    // Vertical
                    let width = if thick > 0 && x % thick == 0 {
                        0.05
                    } else {
                        0.01
                    };
                    let x = x as f32;
                    let y = view.y as f32;
                    self.geng.draw2d().draw2d(
                        &mut ui_buffer,
                        &editor.model.camera,
                        &draw2d::Segment::new(
                            Segment(vec2(x, -y) * grid_size, vec2(x, y) * grid_size),
                            width,
                            color,
                        ),
                    );
                }
                for y in -view.y..=view.y {
                    // Horizontal
                    let width = if thick > 0 && y % thick == 0 {
                        0.05
                    } else {
                        0.01
                    };
                    let y = y as f32;
                    let x = view.x as f32;
                    self.geng.draw2d().draw2d(
                        &mut ui_buffer,
                        &editor.model.camera,
                        &draw2d::Segment::new(
                            Segment(vec2(-x, y) * grid_size, vec2(x, y) * grid_size),
                            width,
                            color,
                        ),
                    );
                }
            }

            geng_utils::texture::draw_texture_fit(
                &self.ui_texture,
                screen_aabb,
                vec2(0.5, 0.5),
                &geng::PixelPerfectCamera,
                &self.geng,
                game_buffer,
            );
        }

        if !options.hide_ui {
            // UI
            let framebuffer_size = game_buffer.size().as_f32();
            let camera = &geng::PixelPerfectCamera;
            let screen = Aabb2::ZERO.extend_positive(framebuffer_size);
            let font_size = framebuffer_size.y * 0.05;
            let font = self.geng.default_font();
            let text_color = editor.level.config.theme.light;
            // let outline_color = crate::render::COLOR_DARK;
            // let outline_size = 0.05;

            // Current beat / Fade in/out
            // let mut text = String::new();
            // if self.geng.window().is_key_pressed(geng::Key::ControlLeft) {
            //     if let Some(event) = hovered_event.and_then(|i| editor.level.events.get(i)) {
            //         if let Event::Light(light) = &event.event {
            //             if self.geng.window().is_key_pressed(geng::Key::ShiftLeft) {
            //                 if let Some(frame) = light.light.movement.key_frames.back() {
            //                     text = format!("Fade out time: {}", frame.lerp_time);
            //                 }
            //             } else if let Some(frame) = light.light.movement.key_frames.get(1) {
            //                 text = format!("Fade in time: {}", frame.lerp_time);
            //             }
            //         }
            //     }
            // }
            // font.draw(
            //     game_buffer,
            //     camera,
            //     &text,
            //     vec2::splat(geng::TextAlign(0.5)),
            //     mat3::translate(
            //         geng_utils::layout::aabb_pos(screen, vec2(0.5, 1.0)) + vec2(0.0, -font_size),
            //     ) * mat3::scale_uniform(font_size)
            //         * mat3::translate(vec2(0.0, -0.5)),
            //     text_color,
            // );

            if editor.model.level != editor.level {
                // Save indicator
                let text = "Ctrl+S to save the level";
                font.draw(
                    game_buffer,
                    camera,
                    text,
                    vec2::splat(geng::TextAlign::RIGHT),
                    mat3::translate(
                        geng_utils::layout::aabb_pos(screen, vec2(1.0, 1.0))
                            + vec2(-1.0, -1.0) * font_size,
                    ) * mat3::scale_uniform(font_size * 0.5),
                    text_color,
                );
            }

            // Undo/redo stack
            let text = match &editor.state {
                State::Playing { .. } => "".to_string(),
                State::Movement {
                    light, redo_stack, ..
                } => format!(
                    "New light stack\nUndo: {}\nRedo: {}\n",
                    light.light.movement.key_frames.len(),
                    redo_stack.len()
                ),
                State::Place { .. } => "idk what should we do here".to_string(),
                State::Idle => "Level stack not implemented KEKW".to_string(),
                State::Waypoints { .. } => "Waypoing stack TODO".to_string(),
            };
            font.draw(
                game_buffer,
                camera,
                &text,
                vec2(geng::TextAlign::LEFT, geng::TextAlign::CENTER),
                mat3::translate(
                    geng_utils::layout::aabb_pos(screen, vec2(0.0, 0.5))
                        + vec2(1.0, 1.0) * font_size,
                ) * mat3::scale_uniform(font_size * 0.5)
                    * mat3::translate(vec2(0.0, -0.5)),
                text_color,
            );

            // Help
            let text =
            "Scroll or arrow keys to go forward or backward in time\nHold Shift to scroll by quarter beats\nSpace to play the music\nF to pause movement\nQ/E to rotate\n` (backtick) to toggle grid snap\nCtrl+` to toggle grid visibility";
            font.draw(
                game_buffer,
                camera,
                text,
                vec2::splat(geng::TextAlign::RIGHT),
                mat3::translate(
                    geng_utils::layout::aabb_pos(screen, vec2(1.0, 1.0))
                        + vec2(-1.0, -1.0) * font_size,
                ) * mat3::scale_uniform(font_size * 0.5),
                text_color,
            );

            // Status
            let text = if editor.selected_light.is_some() {
                "X to delete the light\nCtrl + scroll to change fade in time\nCtrl + Shift + scroll to change fade out time"
            } else {
                match &editor.state {
                    State::Idle => "Click on a light to configure\n1/2 to spawn a new one",
                    State::Place { .. } => "Click to set the spawn position for the new light",
                    State::Movement { .. } => {
                        "Left click to create a new waypoint\nRight click to finish\nEscape to cancel"
                    }
                    State::Playing { .. } => "Playing the music...\nSpace to stop",
                    State::Waypoints { ..} => "Drag, rotate, and scale waypoints",
                }
            };
            font.draw(
                game_buffer,
                camera,
                text,
                vec2(geng::TextAlign::CENTER, geng::TextAlign::BOTTOM),
                mat3::translate(
                    geng_utils::layout::aabb_pos(screen, vec2(0.5, 0.0))
                        + vec2(0.0, 1.5 * font_size),
                ) * mat3::scale_uniform(font_size)
                    * mat3::translate(vec2(0.0, 1.0)),
                text_color,
            );
        }
    }
}
