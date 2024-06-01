use super::*;

impl EditorRender {
    pub(super) fn draw_game(&mut self, editor: &Editor, visible: bool) {
        let options = &editor.render_options;
        let mut theme = editor.options.theme;

        let game_buffer =
            &mut geng_utils::texture::attach_texture(&mut self.game_texture, self.geng.ugli());

        if let Some(level_editor) = &editor.level_edit {
            if level_editor.level_state.relevant().swap_palette {
                std::mem::swap(&mut theme.light, &mut theme.dark);
            }
        }

        ugli::clear(game_buffer, Some(theme.dark), None, None);
        let screen_aabb = Aabb2::ZERO.extend_positive(game_buffer.size().as_f32());

        let Some(level_editor) = &editor.level_edit else {
            return;
        };
        if !visible {
            return;
        }

        macro_rules! draw_game {
            ($alpha:expr) => {{
                self.dither
                    .finish(level_editor.real_time, &theme.transparent());
                self.geng.draw2d().textured_quad(
                    game_buffer,
                    &geng::PixelPerfectCamera,
                    screen_aabb,
                    self.dither.get_buffer(),
                    crate::util::with_alpha(Color::WHITE, $alpha),
                );
                self.dither.start()
            }};
        }

        // Level
        let light_color = THEME.light;
        let danger_color = THEME.danger;

        let active_danger = if let State::Movement { light, .. } = &level_editor.state {
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
        let hovered_event = level_editor.level_state.hovered_event();

        let select_color = editor.config.theme.select;
        let selected_event = level_editor.selected_light.map(|i| i.event);

        let get_color =
            |event_id: Option<usize>| -> Color {
                if let Some(event_id) = event_id {
                    let check = |a: Option<usize>| -> bool { a == Some(event_id) };
                    let base_color = if level_editor.level.events.get(event_id).map_or(false, |e| {
                        match &e.event {
                            Event::Light(event) => event.light.danger,
                            _ => false,
                        }
                    }) {
                        danger_color
                    } else {
                        light_color
                    };
                    let mod_color = if check(selected_event) {
                        select_color
                    } else if check(hovered_event) {
                        hover_color
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
        | State::Waypoints { .. } = level_editor.state
        {
            0.5
        } else {
            1.0
        };
        let dynamic_alpha = if level_editor.level_state.static_level.is_some() {
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
                &level_editor.model.camera,
                framebuffer,
            );
        };
        let draw_light = |light: &Light, framebuffer: &mut ugli::Framebuffer| {
            let color = get_color(light.event_id);
            self.util.draw_light_gradient(
                &light.collider,
                color,
                &level_editor.model.camera,
                framebuffer,
            );
        };

        // Dynamic
        let mut pixel_buffer = self.dither.start();

        if let Some(level) = &level_editor.level_state.dynamic_level {
            for tele in &level.telegraphs {
                draw_telegraph(tele, &mut pixel_buffer);
            }
            for light in &level.lights {
                draw_light(light, &mut pixel_buffer);
            }
        }

        let mut pixel_buffer = draw_game!(dynamic_alpha);

        if let Some(level) = &level_editor.level_state.static_level {
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
            } = level_editor.state
            {
                let time = level_editor.current_beat - start_beat
                    + light.light.movement.fade_in
                    + light.telegraph.precede_time;
                let draw_active = |time: Time, pixel_buffer: &mut ugli::Framebuffer| {
                    let event = commit_light(light.clone());
                    let (tele, light) =
                        render_light(&event, time, None, &level_editor.model.level.config);
                    for tele in tele {
                        draw_telegraph(&tele, pixel_buffer);
                    }
                    if let Some(light) = light {
                        draw_light(&light, pixel_buffer);
                    }
                };

                let mut pixel_buffer = if editor.visualize_beat {
                    // Active movement
                    let time = time
                        + (level_editor.real_time
                            / level_editor.static_level.group.music.meta.beat_time())
                        .fract();
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
            let shape = match level_editor.state {
                State::Place { shape, danger } => Some((shape, danger)),
                State::Movement { ref light, .. } => Some((light.light.shape, light.light.danger)),
                _ => None,
            };
            if let Some((shape, danger)) = shape {
                let collider = Collider {
                    position: editor.cursor_world_pos_snapped,
                    rotation: level_editor.place_rotation,
                    shape: shape.scaled(level_editor.place_scale),
                };
                let color = if danger { THEME.danger } else { THEME.light };
                self.util.draw_outline(
                    &collider,
                    0.05,
                    color,
                    &level_editor.model.camera,
                    &mut pixel_buffer,
                );
            }
        }
        let mut pixel_buffer = draw_game!(1.0);

        if let State::Waypoints { event, .. } = &level_editor.state {
            let event = *event;
            if let Some(event) = level_editor.level.events.get(event) {
                if let Event::Light(event) = &event.event {
                    let color = if event.light.danger {
                        danger_color
                    } else {
                        light_color
                    };

                    // A dashed line moving through the waypoints to show general direction
                    let mut positions: Vec<vec2<f32>> = level_editor
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
                        let t =
                            ((level_editor.real_time.as_f32() * speed) / period).fract() * period;
                        *pos += (to - *pos).normalize_or_zero() * t;
                    }
                    let chain = Chain::new(positions);
                    self.util.draw_dashed_chain(
                        &chain,
                        &options,
                        &level_editor.model.camera,
                        &mut pixel_buffer,
                    );

                    if let Some(waypoints) = &level_editor.level_state.waypoints {
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
                                &level_editor.model.camera,
                                &mut pixel_buffer,
                            );
                            self.util.draw_text(
                                format!("{}", i + 1),
                                point.collider.position,
                                TextRenderOptions::new(1.5).color(THEME.light),
                                &level_editor.model.camera,
                                &mut pixel_buffer,
                            );
                            if let Some(i) = point.original {
                                if let Some(event) = level_editor.level.events.get(waypoints.event)
                                {
                                    if let Event::Light(light) = &event.event {
                                        if let Some(beat) = light.light.movement.get_time(i) {
                                            let beat =
                                                event.beat + light.telegraph.precede_time + beat;
                                            self.util.draw_text(
                                                format!("at {}", beat),
                                                point.collider.position - vec2(0.0, 0.6).as_r32(),
                                                TextRenderOptions::new(0.6).color(THEME.light),
                                                &level_editor.model.camera,
                                                &mut pixel_buffer,
                                            );
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        draw_game!(1.0);

        let gameplay_fov = 10.0;
        let gameplay_area =
            Aabb2::ZERO.extend_symmetric(vec2(16.0 / 9.0, 1.0) * gameplay_fov / 2.0);
        self.util.draw_outline(
            &Collider::aabb(gameplay_area.map(r32)),
            0.1,
            theme.highlight,
            &level_editor.model.camera,
            game_buffer,
        );

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
                    level_editor.model.camera.fov * ui_buffer.size().as_f32().aspect(),
                    level_editor.model.camera.fov,
                )
                .map(|x| (x / 2.0 / grid_size).ceil() as i64);
                let thick = editor.config.grid.thick_every as i64;
                for x in -view.x..=view.x {
                    // Vertical
                    let width = if thick > 0 && x % thick == 0 {
                        0.05
                    } else {
                        0.02
                    };
                    let x = x as f32;
                    let y = view.y as f32;
                    self.geng.draw2d().draw2d(
                        &mut ui_buffer,
                        &level_editor.model.camera,
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
                        &level_editor.model.camera,
                        &draw2d::Segment::new(
                            Segment(vec2(-x, y) * grid_size, vec2(x, y) * grid_size),
                            width,
                            color,
                        ),
                    );
                }
            }

            geng_utils::texture::DrawTexture::new(&self.ui_texture)
                .fit(screen_aabb, vec2(0.5, 0.5))
                .draw(&geng::PixelPerfectCamera, &self.geng, game_buffer);
        }
    }
}
