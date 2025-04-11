use super::*;

impl EditorRender {
    pub(super) fn draw_game(&mut self, editor: &Editor, visible: bool) {
        let options = &editor.render_options;
        let mut theme = editor.context.get_options().theme;

        let game_buffer = &mut geng_utils::texture::attach_texture(
            &mut self.game_texture,
            self.context.geng.ugli(),
        );

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
                self.context.geng.draw2d().textured_quad(
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

        let active_color = light_color;

        let hover_color = editor.config.theme.hover;
        let selecting_area = matches!(
            editor.drag.as_ref().map(|drag| &drag.target),
            Some(DragTarget::SelectionArea { .. })
        );
        let hovered_event = (!selecting_area)
            .then(|| level_editor.level_state.hovered_event())
            .flatten();

        let select_color = editor.config.theme.select;

        let get_color = |event_id: Option<usize>| -> Color {
            if let Some(event_id) = event_id {
                let check = |a: Option<usize>| -> bool { a == Some(event_id) };
                let base_color =
                    if level_editor
                        .level
                        .events
                        .get(event_id)
                        .is_some_and(|e| match &e.event {
                            Event::Light(event) => event.danger,
                            _ => false,
                        })
                    {
                        danger_color
                    } else {
                        light_color
                    };
                let mod_color = if !editor.show_only_selected
                    && level_editor
                        .selection
                        .is_light_selected(LightId { event: event_id })
                {
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

        let static_alpha = if let State::Place { .. } | State::Waypoints { .. } = level_editor.state
        {
            0.75
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

        {
            // Current action
            let shape = match level_editor.state {
                State::Place { shape, danger } => Some((shape, danger)),
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

        // TODO: adapt to movement density
        /// How much time away are the waypoints still visible
        const VISIBILITY: Time = TIME_IN_FLOAT_TIME * 5;
        /// The minimum transparency level of waypoints outside visibility
        const MIN_ALPHA: f32 = 0.2;
        /// Waypoints past this time-distance are not rendered at all
        const MAX_VISIBILITY: Time = TIME_IN_FLOAT_TIME * 15;
        // Calculate the waypoint visibility at the given relative timestamp
        let visibility = |timed_event: &TimedEvent, beat: Time| {
            let d = (timed_event.time + beat - level_editor.current_time.value).abs();
            if d > MAX_VISIBILITY {
                return 0.0;
            }
            let d = d as f32 / VISIBILITY as f32;
            (1.0 - d.sqr()).clamp(MIN_ALPHA, 1.0)
        };

        if let Selection::Lights(lights) = &level_editor.selection {
            for light_id in lights {
                if let Some(timed_event) = level_editor.level.events.get(light_id.event) {
                    let visibility = |beat| visibility(timed_event, beat);

                    if let Event::Light(event) = &timed_event.event {
                        let color = if event.danger {
                            danger_color
                        } else {
                            light_color
                        };
                        let alpha = if let State::Waypoints { .. } = level_editor.state {
                            1.0
                        } else {
                            0.5
                        };
                        let color = crate::util::with_alpha(color, alpha);

                        let options = util::DashRenderOptions {
                            width: 0.15,
                            dash_length: 0.1,
                            space_length: 0.2,
                        };

                        // A dashed line moving through the waypoints to show general direction
                        const POINTS_DENSITY: f32 = 5.0;
                        let num_points = (POINTS_DENSITY * event.movement.total_distance().as_f32())
                            .round() as usize;
                        if !event.movement.key_frames.is_empty() && num_points > 0 {
                            let period =
                                time_to_seconds(event.movement.movement_duration()).max(r32(0.01)); // NOTE: avoid dividing by 0
                            let speed = r32(1.0 / 8.0); // game time per real time
                            let positions: Vec<draw2d::ColoredVertex> = (0..=num_points)
                                .map(|i| {
                                    let t = r32(i as f32 / num_points as f32);
                                    let t = (level_editor.real_time / period * speed + t).fract()
                                        * period;
                                    let t = seconds_to_time(t) + event.movement.fade_in;
                                    let alpha = visibility(t);
                                    draw2d::ColoredVertex {
                                        a_pos: event.movement.get(t).translation.as_f32(), // TODO: check performance
                                        a_color: crate::util::with_alpha(color, alpha),
                                    }
                                })
                                .collect();

                            self.util.draw_dashed_movement(
                                &positions,
                                &options,
                                &level_editor.model.camera,
                                &mut pixel_buffer,
                            );
                        }
                    }
                }
            }
        }

        if let State::Waypoints { light_id, .. } = &level_editor.state {
            let light_id = *light_id;
            if let Some(timed_event) = level_editor.level.events.get(light_id.event) {
                let visibility = |beat| visibility(timed_event, beat);

                if let Event::Light(event) = &timed_event.event {
                    let color = if event.danger {
                        danger_color
                    } else {
                        light_color
                    };

                    let options = util::DashRenderOptions {
                        width: 0.15,
                        dash_length: 0.1,
                        space_length: 0.2,
                    };

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

                            let mut alpha = 1.0;
                            let _original = point.original.and_then(|i| {
                                level_editor
                                    .level
                                    .events
                                    .get(waypoints.light.event)
                                    .and_then(|event| {
                                        if let Event::Light(light) = &event.event {
                                            let beat = light.movement.get_time(i)?;
                                            alpha = visibility(beat);
                                            return Some((event.time, beat));
                                        }
                                        None
                                    })
                            });
                            if alpha <= 0.0 {
                                continue;
                            }

                            if point.actual != point.control {
                                self.util.draw_outline(
                                    &point.control,
                                    0.025,
                                    crate::util::with_alpha(color, alpha),
                                    &level_editor.model.camera,
                                    &mut pixel_buffer,
                                );

                                // Connect control and actual with a line
                                let color = crate::util::with_alpha(color, alpha * 0.75);
                                let mut from = draw2d::ColoredVertex {
                                    a_pos: point.control.position.as_f32(),
                                    a_color: color,
                                };
                                let to = draw2d::ColoredVertex {
                                    a_pos: point.actual.position.as_f32(),
                                    a_color: color,
                                };

                                let period = options.dash_length + options.space_length;
                                let speed = 1.0;
                                let t = ((level_editor.real_time.as_f32() * speed) / period)
                                    .fract()
                                    * period;
                                from.a_pos += (to.a_pos - from.a_pos).normalize_or_zero() * t;
                                self.util.draw_dashed_chain(
                                    &[from, to],
                                    &util::DashRenderOptions {
                                        width: options.width * 0.5,
                                        ..options
                                    },
                                    &level_editor.model.camera,
                                    &mut pixel_buffer,
                                );
                            }

                            self.util.draw_outline(
                                &point.actual,
                                0.05,
                                crate::util::with_alpha(color, alpha),
                                &level_editor.model.camera,
                                &mut pixel_buffer,
                            );
                            let text_color = crate::util::with_alpha(THEME.light, alpha);
                            self.util.draw_text_with(
                                format!("{}", i + 1),
                                point.control.position,
                                0.0,
                                TextRenderOptions::new(1.5).color(text_color),
                                ugli::DrawParameters {
                                    blend_mode: Some(util::additive()),
                                    ..default()
                                },
                                &level_editor.model.camera,
                                &mut pixel_buffer,
                            );

                            // let beat_time = point.original.map_or(
                            //     Some(level_editor.current_time.target),
                            //     |_| {
                            //         original.map(|(original_beat, relative_beat)| {
                            //             original_beat + relative_beat
                            //         })
                            //     },
                            // );
                            // if let Some(beat) = beat_time {
                            //     self.util.draw_text_with(
                            //         format!("at {}", beat),
                            //         point.control.position - vec2(0.0, 0.6).as_r32(),
                            //         0.0,
                            //         TextRenderOptions::new(0.6).color(text_color),
                            //         ugli::DrawParameters {
                            //             blend_mode: Some(util::additive()),
                            //             ..default()
                            //         },
                            //         &level_editor.model.camera,
                            //         &mut pixel_buffer,
                            //     );
                            // }
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

        {
            // World UI
            let mut ui_buffer =
                geng_utils::texture::attach_texture(&mut self.ui_texture, self.context.geng.ugli());
            ugli::clear(&mut ui_buffer, Some(Rgba::TRANSPARENT_BLACK), None, None);

            let grid_thick = 5.0; // in pixels
            let grid_thin = 3.0; // in pixels

            // Grid
            if options.show_grid {
                let color = crate::util::with_alpha(Color::lerp(theme.dark, theme.light, 0.7), 0.8);
                let grid_size = editor.grid.cell_size.as_f32();
                let view = vec2(
                    level_editor.model.camera.fov * ui_buffer.size().as_f32().aspect(),
                    level_editor.model.camera.fov,
                ) / 2.0
                    / grid_size;
                let view = view.map(|x| x.ceil() as i64);
                let thick = editor.config.grid.thick_every as i64;

                let buffer_size = ui_buffer.size().as_f32();
                let ppp = |pos| {
                    let camera = &level_editor.model.camera;
                    let pos = camera_world_to_screen(pos, camera, buffer_size);
                    pos.map(f32::floor)
                };

                let mut grid_geometry = Vec::new();
                let mk_v = |a_pos| draw2d::ColoredVertex {
                    a_pos,
                    a_color: color,
                };

                for x in -view.x..=view.x {
                    // Vertical
                    let width = if thick > 0 && x % thick == 0 {
                        grid_thick
                    } else {
                        grid_thin
                    };
                    let x = x as f32;
                    let y = view.y as f32;

                    let p1 = ppp(vec2(x, -y) * grid_size);
                    let p2 = ppp(vec2(x, y) * grid_size);
                    let normal = (p2 - p1).normalize_or_zero().rotate_90();
                    let offset = normal * width / 2.0;
                    let [a, b, c, d] = [p1 + offset, p1 - offset, p2 - offset, p2 + offset];
                    grid_geometry.extend([a, b, c, a, c, d].map(mk_v));
                }
                for y in -view.y..=view.y {
                    // Horizontal
                    let width = if thick > 0 && y % thick == 0 {
                        grid_thick
                    } else {
                        grid_thin
                    };
                    let y = y as f32;
                    let x = view.x as f32;

                    let p1 = ppp(vec2(-x, y) * grid_size);
                    let p2 = ppp(vec2(x, y) * grid_size);
                    let normal = (p2 - p1).normalize_or_zero().rotate_90();
                    let offset = normal * width / 2.0;
                    let [a, b, c, d] = [p1 + offset, p1 - offset, p2 - offset, p2 + offset];
                    grid_geometry.extend([a, b, c, a, c, d].map(mk_v));
                }

                let grid_geometry =
                    ugli::VertexBuffer::new_dynamic(self.context.geng.ugli(), grid_geometry);
                let frame_size = ui_buffer.size().as_f32();
                ugli::draw(
                    &mut ui_buffer,
                    &self.context.assets.shaders.solid,
                    ugli::DrawMode::Triangles,
                    &grid_geometry,
                    (
                        ugli::uniforms! {
                            u_model_matrix: mat3::identity(),
                            u_color: Color::WHITE,
                        },
                        geng::PixelPerfectCamera.uniforms(frame_size),
                    ),
                    draw_parameters(),
                );
            }

            // Selection
            if let Some(drag) = &editor.drag {
                if let DragTarget::SelectionArea { .. } = drag.target {
                    let color = Color::lerp(theme.dark, theme.highlight, 0.5);
                    let selection =
                        Aabb2::from_corners(drag.from_world_raw, editor.cursor_world_pos)
                            .map_bounds(|p| {
                                crate::util::world_to_screen(
                                    &level_editor.model.camera,
                                    game_buffer.size().as_f32(),
                                    p.as_f32(),
                                )
                            });
                    let pixel = crate::render::ui::pixel_scale(ui_buffer.size());
                    let width = 2.0 * pixel;
                    self.ui.fill_quad(
                        selection.extend_uniform(width),
                        crate::util::with_alpha(color, 0.5),
                        &mut ui_buffer,
                    );
                    self.ui
                        .draw_outline(selection, width, color, &mut ui_buffer);
                }
            }

            geng_utils::texture::DrawTexture::new(&self.ui_texture)
                .fit(screen_aabb, vec2(0.5, 0.5))
                .draw(&geng::PixelPerfectCamera, &self.context.geng, game_buffer);
        }
    }
}

fn camera_world_to_screen(
    pos: vec2<f32>,
    camera: &impl geng::AbstractCamera2d,
    framebuffer_size: vec2<f32>,
) -> vec2<f32> {
    let pos = (camera.projection_matrix(framebuffer_size) * camera.view_matrix()) * pos.extend(1.0);
    let pos = pos.xy() / pos.z;
    // if pos.x.abs() > 1.0 || pos.y.abs() > 1.0 {
    //     return None;
    // }
    vec2(
        (pos.x + 1.0) / 2.0 * framebuffer_size.x,
        (pos.y + 1.0) / 2.0 * framebuffer_size.y,
    )
}
