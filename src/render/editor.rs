use super::{
    dither::DitherRender,
    util::{TextRenderOptions, UtilRender},
    *,
};

use crate::editor::{State, *};

use geng::prelude::ugli::{BlendEquation, BlendFactor, BlendMode, ChannelBlendMode};

pub struct EditorRender {
    geng: Geng,
    assets: Rc<Assets>,
    dither: DitherRender,
    dither_small: DitherRender,
    util: UtilRender,
    unit_quad: ugli::VertexBuffer<draw2d::TexturedVertex>,
    game_texture: ugli::Texture,
    ui_texture: ugli::Texture,
}

pub struct RenderOptions {
    pub hide_ui: bool,
    pub show_grid: bool,
}

impl EditorRender {
    pub fn new(geng: &Geng, assets: &Rc<Assets>) -> Self {
        let mut game_texture = geng_utils::texture::new_texture(geng.ugli(), vec2(1, 1));
        game_texture.set_filter(ugli::Filter::Nearest);
        let mut ui_texture = geng_utils::texture::new_texture(geng.ugli(), vec2(1, 1));
        ui_texture.set_filter(ugli::Filter::Nearest);

        Self {
            geng: geng.clone(),
            assets: assets.clone(),
            dither: DitherRender::new(geng, assets),
            dither_small: DitherRender::new_sized(geng, assets, vec2::splat(360)),
            util: UtilRender::new(geng, assets),
            unit_quad: geng_utils::geometry::unit_quad_geometry(geng.ugli()),
            game_texture,
            ui_texture,
        }
    }

    pub fn draw_editor(
        &mut self,
        editor: &Editor,
        ui: &EditorUI,
        options: &RenderOptions,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        self.dither_small.update_render_size(ui.light_size);
        geng_utils::texture::update_texture_size(
            &mut self.game_texture,
            ui.game.position.size().map(|x| x.round() as usize),
            self.geng.ugli(),
        );
        geng_utils::texture::update_texture_size(
            &mut self.ui_texture,
            framebuffer.size(),
            self.geng.ugli(),
        );

        self.draw_game(editor, options);
        self.draw_ui(editor, ui, options);

        let camera = &geng::PixelPerfectCamera;
        self.geng.draw2d().textured_quad(
            framebuffer,
            camera,
            ui.game.position,
            &self.game_texture,
            Color::WHITE,
        );
        self.geng.draw2d().textured_quad(
            framebuffer,
            camera,
            Aabb2::ZERO.extend_positive(framebuffer.size().as_f32()),
            &self.ui_texture,
            Color::WHITE,
        );
    }

    fn draw_ui(&mut self, editor: &Editor, ui: &EditorUI, _render_options: &RenderOptions) {
        let screen_buffer =
            &mut geng_utils::texture::attach_texture(&mut self.ui_texture, self.geng.ugli());

        let framebuffer_size = screen_buffer.size().as_f32();
        let camera = &geng::PixelPerfectCamera;
        ugli::clear(
            screen_buffer,
            Some(editor.level.config.theme.dark),
            None,
            None,
        );

        let font_size = ui.screen.position.height() * 0.04;
        let options = TextRenderOptions::new(font_size)
            .color(editor.level.config.theme.light)
            .align(vec2(0.5, 1.0));

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
                self.util.draw_checkbox(widget, options, screen_buffer);
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
            let mut dither_buffer = self.dither_small.start(Color::TRANSPARENT_BLACK);
            let mut collider = Collider::new(vec2::ZERO, light.shape);
            collider.rotation = Angle::from_degrees(light.rotation);
            let color = if light.danger {
                editor.level.config.theme.danger
            } else {
                editor.level.config.theme.light
            };
            self.util.draw_collider(
                &collider,
                color,
                &Camera2d {
                    center: vec2::ZERO,
                    rotation: Angle::ZERO,
                    fov: 3.0,
                },
                &mut dither_buffer,
            );
            self.dither_small.finish(editor.real_time, R32::ZERO);

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
            self.util
                .draw_checkbox(&ui.selected_light.danger, options, screen_buffer);
            self.util
                .draw_text_widget(&ui.selected_light.fade_in, options, screen_buffer);
            self.util
                .draw_text_widget(&ui.selected_light.fade_out, options, screen_buffer);
            self.util
                .draw_text_widget(&ui.selected_light.scale, options, screen_buffer);
        }

        // Beat
        self.util
            .draw_text_widget(&ui.current_beat, options, screen_buffer);

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

    fn draw_game(&mut self, editor: &Editor, options: &RenderOptions) {
        let game_buffer =
            &mut geng_utils::texture::attach_texture(&mut self.game_texture, self.geng.ugli());
        ugli::clear(
            game_buffer,
            Some(editor.level_state.relevant().config.theme.dark),
            None,
            None,
        );
        let screen_aabb = Aabb2::ZERO.extend_positive(game_buffer.size().as_f32());

        // Level
        let mut pixel_buffer = self
            .dither
            .start(editor.level_state.relevant().config.theme.dark);

        let (active_danger, base_alpha) = if let State::Movement { light, .. } = &editor.state {
            (light.light.danger, 0.5)
        } else {
            (false, 1.0)
        };

        let light_color = editor.level.config.theme.light;
        let danger_color = editor.level.config.theme.danger;

        let active_color = if active_danger {
            danger_color
        } else {
            light_color
        };

        let light_color = crate::util::with_alpha(light_color, base_alpha);
        let danger_color = crate::util::with_alpha(danger_color, base_alpha);

        let hover_color = crate::util::with_alpha(editor.config.theme.hover, base_alpha);
        let hovered_event = editor.level_state.hovered_event();

        let select_color = crate::util::with_alpha(editor.config.theme.select, base_alpha);
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

        if let Some(level) = &editor.level_state.dynamic_level {
            let alpha = if editor.level_state.static_level.is_some() {
                0.5
            } else {
                1.0
            };

            for tele in &level.telegraphs {
                let color = crate::util::with_alpha(get_color(tele.light.event_id), alpha);
                self.util.draw_outline(
                    &tele.light.collider,
                    0.02,
                    color,
                    &editor.model.camera,
                    &mut pixel_buffer,
                );
            }
            for light in &level.lights {
                let color = crate::util::with_alpha(get_color(light.event_id), alpha);
                self.util.draw_collider(
                    &light.collider,
                    color,
                    &editor.model.camera,
                    &mut pixel_buffer,
                );
            }
        }

        if let Some(level) = &editor.level_state.static_level {
            for tele in &level.telegraphs {
                let color = get_color(tele.light.event_id);
                self.util.draw_outline(
                    &tele.light.collider,
                    0.02,
                    color,
                    &editor.model.camera,
                    &mut pixel_buffer,
                );
            }
            for light in &level.lights {
                let color = get_color(light.event_id);
                self.util.draw_collider(
                    &light.collider,
                    color,
                    &editor.model.camera,
                    &mut pixel_buffer,
                );
            }
        }

        if !options.hide_ui {
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

        self.dither.finish(editor.real_time, R32::ZERO);

        geng_utils::texture::draw_texture_fit(
            self.dither.get_buffer(),
            screen_aabb,
            vec2(0.5, 0.5),
            &geng::PixelPerfectCamera,
            &self.geng,
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
                    light.light.movement.key_frames.len() - 2,
                    redo_stack.len()
                ),
                State::Place { .. } => "idk what should we do here".to_string(),
                State::Idle => "Level stack not implemented KEKW".to_string(),
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
