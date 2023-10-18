use super::*;

use crate::editor::{State, *};

use geng::prelude::ugli::{BlendEquation, BlendFactor, BlendMode, ChannelBlendMode};

pub struct EditorRender {
    geng: Geng,
    assets: Rc<Assets>,
    render: Render,
    util: UtilRender,
    game_texture: ugli::Texture,
    ui_texture: ugli::Texture,
}

pub struct RenderOptions {
    pub hide_ui: bool,
    pub show_grid: bool,
}

impl EditorRender {
    pub fn new(geng: &Geng, assets: &Rc<Assets>) -> Self {
        let mut game_texture =
            geng_utils::texture::new_texture(geng.ugli(), vec2(1080 * 16 / 9, 1080));
        game_texture.set_filter(ugli::Filter::Nearest);
        let mut ui_texture =
            geng_utils::texture::new_texture(geng.ugli(), vec2(1080 * 16 / 9, 1080));
        ui_texture.set_filter(ugli::Filter::Nearest);

        Self {
            geng: geng.clone(),
            assets: assets.clone(),
            render: Render::new(geng, assets),
            util: UtilRender::new(geng, assets),
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
        self.draw_game(editor, options);
        self.draw_ui(editor, ui, options);

        // let framebuffer_size = framebuffer.size().as_f32();
        let camera = &geng::PixelPerfectCamera;
        self.geng.draw2d().textured_quad(
            framebuffer,
            camera,
            ui.game,
            &self.game_texture,
            Color::WHITE,
        );
        self.geng.draw2d().textured_quad(
            framebuffer,
            camera,
            ui.screen,
            &self.ui_texture,
            Color::WHITE,
        );
    }

    fn draw_ui(&mut self, editor: &Editor, ui: &EditorUI, render_options: &RenderOptions) {
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

        let font_size = ui.screen.height() * 0.04;
        let options = TextRenderOptions::new(font_size)
            .color(editor.level.config.theme.light)
            .align(vec2(0.5, 1.0));

        {
            // Level info
            let pos = vec2(ui.level_info.center().x, ui.level_info.max.y);
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
            let options = options.align(vec2(0.0, 0.5));
            let mut pos = vec2(ui.general.min.x + font_size, ui.general.max.y - font_size);
            for (name, checked) in [
                ("Show movement", editor.visualize_beat),
                ("Show grid", render_options.show_grid),
                ("Snap to grid", editor.snap_to_grid),
            ] {
                let checkbox = Aabb2::point(pos).extend_uniform(font_size / 3.0);
                if checked {
                    let checkbox = checkbox.extend_uniform(-font_size * 0.05);
                    for (a, b) in [
                        (checkbox.bottom_left(), checkbox.top_right()),
                        (checkbox.top_left(), checkbox.bottom_right()),
                    ] {
                        self.geng.draw2d().draw2d(
                            screen_buffer,
                            camera,
                            &draw2d::Segment::new(Segment(a, b), font_size * 0.07, options.color),
                        );
                    }
                }
                self.util.draw_outline(
                    &Collider::aabb(checkbox.map(r32)),
                    font_size * 0.1,
                    options.color,
                    camera,
                    screen_buffer,
                );
                self.util.draw_text(
                    name,
                    pos + vec2(font_size, 0.0),
                    options,
                    camera,
                    screen_buffer,
                );

                pos -= vec2(0.0, font_size);
            }
        }

        // Leave the game area transparent
        ugli::draw(
            screen_buffer,
            &self.assets.shaders.solid,
            ugli::DrawMode::TriangleFan,
            &self.render.unit_quad,
            (
                ugli::uniforms! {
                    u_model_matrix: mat3::translate(ui.game.center()) * mat3::scale(ui.game.size() / 2.0),
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
            &Collider::aabb(ui.game.extend_uniform(width).map(r32)),
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
            .render
            .start(editor.level_state.relevant().config.theme.dark);

        let base_alpha = if let State::Movement { .. } = &editor.state {
            0.5
        } else {
            1.0
        };
        let color = crate::util::with_alpha(editor.level.config.theme.light, base_alpha);

        let hover_color = crate::util::with_alpha(Rgba::CYAN, base_alpha);
        let hovered_event = editor.level_state.hovered_event();

        if let Some(level) = &editor.level_state.dynamic_level {
            let alpha = if editor.level_state.static_level.is_some() {
                0.5
            } else {
                1.0
            };
            let color = crate::util::with_alpha(color, alpha);
            for tele in &level.telegraphs {
                let color = if hovered_event.is_some() && hovered_event == tele.light.event_id {
                    hover_color
                } else {
                    color
                };
                self.util.draw_outline(
                    &tele.light.collider,
                    0.02,
                    color,
                    &editor.model.camera,
                    &mut pixel_buffer,
                );
            }
            for light in &level.lights {
                let color = if hovered_event.is_some() && hovered_event == light.event_id {
                    hover_color
                } else {
                    color
                };
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
                let color = if hovered_event.is_some() && hovered_event == tele.light.event_id {
                    hover_color
                } else {
                    color
                };
                self.util.draw_outline(
                    &tele.light.collider,
                    0.02,
                    color,
                    &editor.model.camera,
                    &mut pixel_buffer,
                );
            }
            for light in &level.lights {
                let color = if hovered_event.is_some() && hovered_event == light.event_id {
                    hover_color
                } else {
                    color
                };
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
            if !matches!(editor.state, State::Playing { .. }) {
                if let Some(&selected_shape) = editor.model.config.shapes.get(editor.selected_shape)
                {
                    let collider = Collider {
                        position: editor.cursor_world_pos,
                        rotation: editor.place_rotation,
                        shape: selected_shape,
                    };
                    self.util.draw_outline(
                        &collider,
                        0.05,
                        editor.level.config.theme.light,
                        &editor.model.camera,
                        &mut pixel_buffer,
                    );
                }
            }
        }

        self.render.dither(editor.real_time, R32::ZERO);

        geng_utils::texture::draw_texture_fit(
            self.render.get_buffer(),
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
            let mut text = format!("Beat: {:.2}", editor.current_beat);
            if self.geng.window().is_key_pressed(geng::Key::ControlLeft) {
                if let Some(event) = hovered_event.and_then(|i| editor.level.events.get(i)) {
                    if let Event::Light(light) = &event.event {
                        if self.geng.window().is_key_pressed(geng::Key::ShiftLeft) {
                            if let Some(frame) = light.light.movement.key_frames.back() {
                                text = format!("Fade out time: {}", frame.lerp_time);
                            }
                        } else if let Some(frame) = light.light.movement.key_frames.get(1) {
                            text = format!("Fade in time: {}", frame.lerp_time);
                        }
                    }
                }
            }
            font.draw(
                game_buffer,
                camera,
                &text,
                vec2::splat(geng::TextAlign(0.5)),
                mat3::translate(
                    geng_utils::layout::aabb_pos(screen, vec2(0.5, 1.0)) + vec2(0.0, -font_size),
                ) * mat3::scale_uniform(font_size)
                    * mat3::translate(vec2(0.0, -0.5)),
                text_color,
            );

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
                State::Place => "Level stack not implemented KEKW".to_string(),
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
            let text = if editor.level_state.hovered_light.is_some() {
                "X to delete the light\nCtrl + scroll to change fade in time\nCtrl + Shift + scroll to change fade out time"
            } else {
                match &editor.state {
                State::Place => "Click to create a new light\n1/2 to select different types",
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
