use super::*;

impl Editor {
    pub fn render_lights(&mut self) {
        self.rendered_lights.clear();
        self.rendered_telegraphs.clear();
        self.hovered_light = None;

        let (static_time, dynamic_time) = if let State::Playing { .. } = self.state {
            // TODO: self.music.play_position()
            (None, Some(self.time))
        } else {
            let time = self.current_beat * self.level.beat_time();
            let dynamic = if self.visualize_beat {
                Some((self.time / self.level.beat_time()).fract() * self.level.beat_time() + time)
            } else {
                None
            };
            (Some(time), dynamic)
        };

        let mut render_light = |index: Option<usize>, event: &TimedEvent, transparency: f32| {
            if event.beat <= self.current_beat {
                let start = event.beat * self.level.beat_time();
                let static_time = static_time.map(|t| t - start);
                let dynamic_time = dynamic_time.map(|t| t - start);

                match &event.event {
                    Event::Theme(_) => {}
                    Event::Light(event) => {
                        let light = event.light.clone().instantiate(self.level.beat_time());
                        let mut tele =
                            light.into_telegraph(event.telegraph.clone(), self.level.beat_time());
                        let duration = tele.light.movement.duration();

                        let static_light = static_time.and_then(|time| {
                            let time = time - tele.spawn_timer;
                            (time > Time::ZERO && time < duration).then(|| {
                                let transform = tele.light.movement.get(time);
                                tele.light.collider =
                                    tele.light.base_collider.transformed(transform);
                                tele.light.clone()
                            })
                        });

                        let hover = self.hovered_light.is_none()
                            && index.is_some()
                            && static_light
                                .as_ref()
                                .map(|light| light.collider.contains(self.cursor_world_pos))
                                .unwrap_or(false);
                        if hover {
                            self.hovered_light = index;
                        }

                        if let Some(time) = dynamic_time {
                            let transparency =
                                transparency * if static_time.is_some() { 0.5 } else { 1.0 };

                            // Telegraph
                            if time < duration {
                                let transform = tele.light.movement.get(time);
                                tele.light.collider =
                                    tele.light.base_collider.transformed(transform);
                                self.rendered_telegraphs
                                    .push((tele.clone(), transparency, hover));
                            }

                            // Light
                            let time = time - tele.spawn_timer;
                            if time > Time::ZERO && time < duration {
                                let transform = tele.light.movement.get(time);
                                tele.light.collider =
                                    tele.light.base_collider.transformed(transform);
                                self.rendered_lights.push((
                                    tele.light.clone(),
                                    transparency,
                                    hover,
                                ));
                            }
                        }

                        if let Some(time) = static_time {
                            // Telegraph
                            if time < duration {
                                let transform = tele.light.movement.get(time);
                                tele.light.collider =
                                    tele.light.base_collider.transformed(transform);
                                self.rendered_telegraphs
                                    .push((tele.clone(), transparency, hover));
                            }
                        }
                        if let Some(light) = static_light {
                            self.rendered_lights.push((light, transparency, hover));
                        }
                    }
                }
            }
        };

        for (i, e) in self.level.events.iter().enumerate() {
            let transparency = if let State::Movement { .. } = &self.state {
                0.5
            } else {
                1.0
            };
            render_light(Some(i), e, transparency);
        }
        if let State::Movement {
            start_beat, light, ..
        } = &self.state
        {
            render_light(None, &commit_light(*start_beat, light.clone()), 1.0);
        };
    }

    pub fn draw(&mut self, screen_buffer: &mut ugli::Framebuffer) {
        self.framebuffer_size = screen_buffer.size();
        ugli::clear(
            screen_buffer,
            Some(self.level.config.theme.dark),
            None,
            None,
        );
        let screen_aabb = Aabb2::ZERO.extend_positive(screen_buffer.size().as_f32());

        // Level
        let mut pixel_buffer = self.render.start(self.level.config.theme.dark);

        let color = self.level.config.theme.light;
        for (tele, transparency, _) in &self.rendered_telegraphs {
            self.util_render.draw_outline(
                &tele.light.collider,
                0.02,
                crate::util::with_alpha(color, *transparency),
                &self.model.camera,
                &mut pixel_buffer,
            );
        }
        for (light, transparency, _) in &self.rendered_lights {
            self.util_render.draw_collider(
                &light.collider,
                crate::util::with_alpha(color, *transparency),
                &self.model.camera,
                &mut pixel_buffer,
            );
        }

        if !self.hide_ui {
            // Current action
            if !matches!(self.state, State::Playing { .. }) {
                if let Some(&selected_shape) = self.model.config.shapes.get(self.selected_shape) {
                    let collider = Collider {
                        position: self.cursor_world_pos,
                        rotation: self.place_rotation,
                        shape: selected_shape,
                    };
                    self.util_render.draw_outline(
                        &collider,
                        0.05,
                        self.level.config.theme.light,
                        &self.model.camera,
                        &mut pixel_buffer,
                    );
                }
            }
        }

        let mut pixel_buffer = self.render.dither(self.time, R32::ZERO);

        // Hover
        let hover_color = Rgba::CYAN;
        for (tele, _, hover) in &self.rendered_telegraphs {
            if !*hover {
                continue;
            }
            self.util_render.draw_outline(
                &tele.light.collider,
                0.02,
                hover_color,
                &self.model.camera,
                &mut pixel_buffer,
            );
        }
        for (light, _, hover) in &self.rendered_lights {
            if !*hover {
                continue;
            }
            self.util_render.draw_collider(
                &light.collider,
                hover_color,
                &self.model.camera,
                &mut pixel_buffer,
            );
        }

        geng_utils::texture::draw_texture_fit(
            self.render.get_buffer(),
            screen_aabb,
            vec2(0.5, 0.5),
            &geng::PixelPerfectCamera,
            &self.geng,
            screen_buffer,
        );

        if !self.hide_ui {
            // World UI
            let mut ui_buffer =
                geng_utils::texture::attach_texture(&mut self.ui_texture, self.geng.ugli());
            ugli::clear(&mut ui_buffer, Some(Rgba::TRANSPARENT_BLACK), None, None);

            // Grid
            if self.show_grid {
                let color = Rgba {
                    r: 0.7,
                    g: 0.7,
                    b: 0.7,
                    a: 0.7,
                };
                let grid_size = self.grid_size.as_f32();
                let view = vec2(
                    self.model.camera.fov * ui_buffer.size().as_f32().aspect(),
                    self.model.camera.fov,
                )
                .map(|x| (x / 2.0 / grid_size).ceil() as i64);
                let thick = self.config.grid.thick_every as i64;
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
                        &self.model.camera,
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
                        &self.model.camera,
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
                screen_buffer,
            );
        }

        if !self.hide_ui {
            // UI
            let framebuffer_size = screen_buffer.size().as_f32();
            let camera = &geng::PixelPerfectCamera;
            let screen = Aabb2::ZERO.extend_positive(framebuffer_size);
            let font_size = framebuffer_size.y * 0.05;
            let font = self.geng.default_font();
            let text_color = self.level.config.theme.light;
            // let outline_color = crate::render::COLOR_DARK;
            // let outline_size = 0.05;

            // Current beat / Fade in/out
            let mut text = format!("Beat: {:.2}", self.current_beat);
            if self.geng.window().is_key_pressed(geng::Key::ControlLeft) {
                if let Some(event) = self
                    .hovered_light
                    .and_then(|light| self.level.events.get_mut(light))
                {
                    if let Event::Light(light) = &mut event.event {
                        if self.geng.window().is_key_pressed(geng::Key::ShiftLeft) {
                            if let Some(frame) = light.light.movement.key_frames.back_mut() {
                                text = format!("Fade out time: {}", frame.lerp_time);
                            }
                        } else if let Some(frame) = light.light.movement.key_frames.get(1) {
                            text = format!("Fade in time: {}", frame.lerp_time);
                        }
                    }
                }
            }
            font.draw(
                screen_buffer,
                camera,
                &text,
                vec2::splat(geng::TextAlign(0.5)),
                mat3::translate(
                    geng_utils::layout::aabb_pos(screen, vec2(0.5, 1.0)) + vec2(0.0, -font_size),
                ) * mat3::scale_uniform(font_size)
                    * mat3::translate(vec2(0.0, -0.5)),
                text_color,
            );

            if self.model.level != self.level {
                // Save indicator
                let text = "Ctrl+S to save the level";
                font.draw(
                    screen_buffer,
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
            let text = match &self.state {
                State::Playing { .. } => "".to_string(),
                State::Movement {
                    light, redo_stack, ..
                } => format!(
                    "New light stack\nUndo: {}\nRedo: {}\n",
                    light.light.movement.key_frames.len() - 2,
                    redo_stack.len()
                ),
                State::Place => "Level stack not implemented KEKW".to_string(),
            };
            font.draw(
                screen_buffer,
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
                screen_buffer,
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
            let text = if self.hovered_light.is_some() {
                "X to delete the light\nCtrl + scroll to change fade in time\nCtrl + Shift + scroll to change fade out time"
            } else {
                match &self.state {
                State::Place => "Click to create a new light\n1/2 to select different types",
                State::Movement { .. } => {
                    "Left click to create a new waypoint\nRight click to finish\nEscape to cancel"
                }
                State::Playing { .. } => "Playing the music...\nSpace to stop",
            }
            };
            font.draw(
                screen_buffer,
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
