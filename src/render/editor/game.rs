use geng_utils::conversions::Aabb2RealConversions;

use super::*;

struct RenderHelper<'a> {
    screen_aabb: Aabb2<f32>,
    interpolation_cache: &'a mut InterpolationCache,
    post_vfx: PostVfx,
    level_assets: &'a LevelAssets,

    editor: &'a Editor,
    level_editor: &'a LevelEditor,
    theme: Theme,

    light_color: Color,
    danger_color: Color,
    hover_color: Color,
    select_color: Color,
    selecting_area: bool,

    get_color: Box<dyn Fn(Option<usize>, Theme) -> Color + 'a>,
}

impl EditorRender {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn draw_game(
        &mut self,
        editor: &Editor,
        screen_aabb: Aabb2<f32>,
        interpolation_cache: &mut InterpolationCache,
        post_vfx: PostVfx,
        level_assets: &LevelAssets,
    ) {
        let mut theme = editor.context.get_options().theme;

        if let Some(level_editor) = &editor.level_edit {
            let swap_t = level_editor.model.vfx.palette_swap.current.as_f32();
            let (light, dark) = (theme.light, theme.dark);
            theme.light = Color::lerp(light, dark, swap_t);
            theme.dark = Color::lerp(dark, light, swap_t);
        }

        let light_color = theme.light;
        let danger_color = theme.danger;

        let game_buffer = &mut self.post_render.begin(self.game_texture.size(), theme.dark);

        ugli::clear(game_buffer, Some(theme.dark), None, None);

        let Some(level_editor) = &editor.level_edit else {
            let game_buffer = &mut geng_utils::texture::attach_texture(
                &mut self.game_texture,
                self.context.geng.ugli(),
            );
            self.post_render
                .post_process(&self.context.get_options(), &post_vfx, game_buffer);
            return;
        };

        {
            // Light SDF
            let framebuffer = &mut geng_utils::texture::attach_texture(
                &mut self.lights_sdf,
                self.context.geng.ugli(),
            );
            ugli::clear(framebuffer, Some(Color::TRANSPARENT_BLACK), None, None);
            self.util.draw_level_sdf(
                level_editor.level_state.relevant(),
                &level_editor.model.camera,
                framebuffer,
            );
        }

        // Level

        let hover_color = editor.config.theme.hover;
        let selecting_area = matches!(
            editor.drag.as_ref().map(|drag| &drag.target),
            Some(DragTarget::SelectionArea { .. })
        );
        let hovered_event = (!selecting_area)
            .then(|| level_editor.level_state.hovered_event())
            .flatten();

        let select_color = editor.config.theme.select;

        let get_color = |event_id: Option<usize>, theme: Theme| -> Color {
            let light_color = theme.light;
            let danger_color = theme.danger;
            let active_color = light_color;

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

        self.draw_game_with(&mut RenderHelper {
            screen_aabb,
            interpolation_cache,
            post_vfx,
            level_assets,

            editor,
            level_editor,
            theme,

            light_color,
            danger_color,
            hover_color,
            select_color,
            selecting_area,

            get_color: Box::new(get_color),
        });
    }

    fn render_lights_sdf(&mut self, helper: &RenderHelper<'_>) {
        let mut pixel_buffer =
            geng_utils::texture::attach_texture(&mut self.lights_sdf, self.context.geng.ugli());
        ugli::clear(
            &mut pixel_buffer,
            Some(Color::TRANSPARENT_BLACK),
            None,
            None,
        );
        self.util.draw_level_sdf(
            helper.level_editor.level_state.relevant(),
            &helper.level_editor.model.camera,
            &mut pixel_buffer,
        );
    }

    fn draw_game_with(&mut self, helper: &mut RenderHelper<'_>) {
        self.render_lights_sdf(helper);

        // Prepare shaders and parameters
        let (active_shaders, shader_uniforms_common, shader_uniforms) =
            crate::render::prepare_shaders(
                helper.theme,
                &helper.level_editor.level,
                &helper.level_editor.model.vfx.shaders,
                helper.level_editor.real_time,
                helper.level_editor.current_time.value,
                helper.level_assets,
                &self.lights_sdf,
            );

        macro_rules! apply_shaders {
            ($order:pat) => {{
                for (shader_time, shader, program) in &active_shaders {
                    let $order = shader.layer else {
                        continue;
                    };
                    let (texture, mut buffer) = self.post_render.apply_processing();
                    ugli::draw(
                        &mut buffer,
                        program,
                        ugli::DrawMode::TriangleFan,
                        &self.util.unit_quad,
                        (
                            &shader_uniforms_common,
                            shader_uniforms(*shader_time, shader),
                            ugli::uniforms! {
                                u_texture: texture,
                            },
                        ),
                        ugli::DrawParameters {
                            blend_mode: Some(ugli::BlendMode::straight_alpha()),
                            ..default()
                        },
                    );
                }
                &mut self.post_render.continu()
            }};
        }

        // Render background shaders
        let game_buffer = apply_shaders!(ShaderLayer::Background);

        let game_screen = Aabb2::ZERO.extend_positive(game_buffer.size().as_f32());
        macro_rules! draw_game {
            ($alpha:expr, $buffer:expr) => {{
                self.dither
                    .finish(helper.level_editor.real_time, &helper.theme.transparent());
                self.context.geng.draw2d().textured_quad(
                    $buffer,
                    &geng::PixelPerfectCamera,
                    game_screen,
                    self.dither.get_buffer(),
                    crate::util::with_alpha(Color::WHITE, $alpha),
                );
                self.dither.start()
            }};
        }

        // Level
        let static_alpha = if let EditingState::Place { .. } | EditingState::Waypoints { .. } =
            helper.level_editor.state
        {
            0.75
        } else {
            1.0
        };
        let dynamic_alpha = if helper.level_editor.level_state.static_level.is_some() {
            0.5
        } else {
            1.0
        } * static_alpha;

        // Dynamic
        let mut pixel_buffer = self.dither.start();

        // Dynamic Lights
        if let Some(level) = &helper.level_editor.level_state.dynamic_level {
            for light in &level.lights {
                helper.draw_light(light, &mut pixel_buffer, &self.util);
            }
        }
        draw_game!(dynamic_alpha, game_buffer);
        let mut pixel_buffer = self.dither.post();

        // Dynamic Telegraphs
        if let Some(level) = &helper.level_editor.level_state.dynamic_level {
            for tele in &level.telegraphs {
                helper.draw_telegraph(tele, &mut pixel_buffer, &self.util, false);
            }
        }
        self.context.geng.draw2d().textured_quad(
            game_buffer,
            &geng::PixelPerfectCamera,
            game_screen,
            self.dither.get_buffer(),
            crate::util::with_alpha(Color::WHITE, dynamic_alpha),
        );
        let mut pixel_buffer = self.dither.start();

        // Static Lights
        if let Some(level) = &helper.level_editor.level_state.static_level {
            for light in &level.lights {
                helper.draw_light(light, &mut pixel_buffer, &self.util);
            }
        }
        draw_game!(static_alpha, game_buffer);
        let mut pixel_buffer = self.dither.post();

        // Static Telegraphs
        if let Some(level) = &helper.level_editor.level_state.static_level {
            for tele in &level.telegraphs {
                helper.draw_telegraph(tele, &mut pixel_buffer, &self.util, false);
            }
        }
        self.context.geng.draw2d().textured_quad(
            game_buffer,
            &geng::PixelPerfectCamera,
            game_screen,
            self.dither.get_buffer(),
            crate::util::with_alpha(Color::WHITE, static_alpha),
        );
        let mut pixel_buffer = self.dither.start();

        // Render post processing (early) shaders
        let _ = apply_shaders!(ShaderLayer::Foreground);

        // Render level state
        let game_buffer = &mut geng_utils::texture::attach_texture(
            &mut self.game_texture,
            self.context.geng.ugli(),
        );
        self.post_render
            .self_process(&self.context.get_options(), &helper.post_vfx);
        // Spotlight effect
        let spotlight = helper
            .level_editor
            .model
            .vfx
            .spotlight
            .value
            .current
            .as_f32();
        if spotlight > 0.0 {
            self.post_render.apply_sdf_mask(&self.lights_sdf, spotlight);
        }

        // Render post processing (late) shaders
        let _ = apply_shaders!(ShaderLayer::PostProcess);

        // Finalize post processing
        self.post_render.render_noop(game_buffer);

        {
            // Current action
            let shape = match helper.level_editor.state {
                EditingState::Place { shape, danger } => Some((shape, danger)),
                _ => None,
            };
            if let Some((shape, danger)) = shape {
                let collider = Collider {
                    position: helper.editor.cursor_world_pos_snapped,
                    rotation: helper.level_editor.place_rotation,
                    shape: shape.scaled(helper.level_editor.place_scale),
                };
                let color = if danger { THEME.danger } else { THEME.light };
                self.util.draw_outline(
                    &collider,
                    0.05,
                    color,
                    &helper.level_editor.model.camera,
                    &mut pixel_buffer,
                );
            }
        }
        let mut pixel_buffer = draw_game!(1.0, game_buffer);

        // TODO: adapt to movement density
        /// How much time away are the waypoints still visible
        const VISIBILITY: Time = TIME_IN_FLOAT_TIME * 5;
        /// The minimum transparency level of waypoints outside visibility
        const MIN_ALPHA: f32 = 0.2;
        /// Waypoints past this time-distance are not rendered at all
        const MAX_VISIBILITY: Time = TIME_IN_FLOAT_TIME * 15;
        // Calculate the waypoint visibility at the given relative timestamp
        let visibility = |timed_event: &TimedEvent, beat: Time| {
            let d = (timed_event.time + beat - helper.level_editor.current_time.value).abs();
            if d > MAX_VISIBILITY {
                return 0.0;
            }
            let d = d as f32 / VISIBILITY as f32;
            (1.0 - d.sqr()).clamp(MIN_ALPHA, 1.0)
        };

        let lights_movement_preview = match &helper.level_editor.selection {
            Selection::Lights(lights) => lights.clone(),
            Selection::Waypoints(light_id, _) => vec![*light_id],
            _ => vec![],
        };
        for light_id in lights_movement_preview {
            if let Some(timed_event) = helper.level_editor.level.events.get(light_id.event) {
                let visibility = |beat| visibility(timed_event, beat);

                if let Event::Light(event) = &timed_event.event {
                    let color = if event.danger {
                        helper.danger_color
                    } else {
                        helper.light_color
                    };
                    let alpha = if let EditingState::Waypoints { .. } = helper.level_editor.state {
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
                    if !event.movement.waypoints.is_empty() && num_points > 0 {
                        let baked = helper.interpolation_cache.get_or_bake(&event.movement);
                        let period = time_to_seconds(event.movement.duration()).max(r32(0.01)); // NOTE: avoid dividing by 0
                        let speed = r32(1.0 / 8.0); // game time per real time
                        let positions: Vec<draw2d::ColoredVertex> = (0..=num_points)
                            .map(|i| {
                                let t = r32(i as f32 / num_points as f32);
                                let t = (helper.level_editor.real_time / period * speed + t)
                                    .fract()
                                    * period;
                                let t = seconds_to_time(t) + event.movement.get_fade_in();
                                let alpha = visibility(t);
                                draw2d::ColoredVertex {
                                    a_pos: event.movement.get_baked(t, baked).translation.as_f32(),
                                    a_color: crate::util::with_alpha(color, alpha),
                                }
                            })
                            .collect();

                        self.util.draw_dashed_movement(
                            &positions,
                            &options,
                            &helper.level_editor.model.camera,
                            &mut pixel_buffer,
                        );
                    }
                }
            }
        }

        if let EditingState::Waypoints { light_id, .. } = &helper.level_editor.state {
            let light_id = *light_id;
            if let Some(timed_event) = helper.level_editor.level.events.get(light_id.event) {
                let visibility = |beat| visibility(timed_event, beat);

                if let Event::Light(event) = &timed_event.event {
                    let color = if event.danger {
                        THEME.danger
                    } else {
                        THEME.light
                    };

                    let options = util::DashRenderOptions {
                        width: 0.15,
                        dash_length: 0.1,
                        space_length: 0.2,
                    };

                    if let Some(waypoints) = &helper.level_editor.level_state.waypoints {
                        // Draw waypoints themselves
                        for (i, point) in waypoints.points.iter().enumerate() {
                            if !point.visible {
                                continue;
                            }
                            let color = if let Some(id) = point.original
                                && helper
                                    .level_editor
                                    .selection
                                    .is_waypoint_selected(waypoints.light, id)
                            {
                                helper.select_color
                            } else if !helper.selecting_area && Some(i) == waypoints.hovered {
                                helper.hover_color
                            } else {
                                color
                            };

                            let mut alpha = 1.0;
                            let _original = point.original.and_then(|i| {
                                helper
                                    .level_editor
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
                                    &helper.level_editor.model.camera,
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
                                let t = ((helper.level_editor.real_time.as_f32() * speed) / period)
                                    .fract()
                                    * period;
                                from.a_pos += (to.a_pos - from.a_pos).normalize_or_zero() * t;
                                self.util.draw_dashed_chain(
                                    &[from, to],
                                    &util::DashRenderOptions {
                                        width: options.width * 0.5,
                                        ..options
                                    },
                                    &helper.level_editor.model.camera,
                                    &mut pixel_buffer,
                                );
                            }

                            self.util.draw_outline(
                                &point.actual,
                                0.05,
                                crate::util::with_alpha(color, alpha),
                                &helper.level_editor.model.camera,
                                &mut pixel_buffer,
                            );
                            self.util.draw_text_with(
                                format!("{}", i + 1),
                                point.control.position,
                                0.0,
                                TextRenderOptions::new(1.5).color(color),
                                ugli::DrawParameters {
                                    blend_mode: Some(util::blend_additive()),
                                    ..default()
                                },
                                &helper.level_editor.model.camera,
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
        draw_game!(1.0, game_buffer);

        // Camera view bounds
        let camera_view = match helper.level_editor.model.camera.fov {
            Camera2dFov::Cover {
                width,
                height,
                scale,
            } => vec2(width, height) * scale * helper.editor.view_zoom.current,
            _ => unreachable!(),
        };
        let camera_view = Aabb2::point(helper.level_editor.model.camera.center)
            .extend_symmetric(camera_view / 2.0);
        let mut camera_view = Collider::aabb(camera_view.as_r32());
        camera_view.rotation = helper.level_editor.model.camera.rotation.map(r32);
        self.util.draw_outline(
            &camera_view,
            0.1,
            helper.theme.highlight,
            &helper.level_editor.model.camera,
            game_buffer,
        );

        {
            // World UI
            let mut ui_buffer =
                geng_utils::texture::attach_texture(&mut self.ui_texture, self.context.geng.ugli());
            ugli::clear(&mut ui_buffer, Some(Rgba::TRANSPARENT_BLACK), None, None);

            let world_to_screen = |pos| {
                crate::util::world_to_screen(
                    &helper.level_editor.model.camera,
                    helper.screen_aabb.size(),
                    pos,
                )
            };

            let grid_thick = 5.0; // in pixels
            let grid_thin = 3.0; // in pixels

            // Grid
            if helper.editor.render_options.show_grid {
                let color = crate::util::with_alpha(
                    Color::lerp(helper.theme.dark, helper.theme.light, 0.7),
                    0.8,
                );
                let grid_size = helper.editor.grid.cell_size.as_f32();

                let view = helper
                    .level_editor
                    .model
                    .camera
                    .view_area(ui_buffer.size().as_f32());
                let view = Aabb2::points_bounding_box(
                    [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)]
                        .map(|(x, y)| (view.transform * vec3(x, y, 1.0)).into_2d()),
                )
                .expect("unit quad has 4 corners");
                let view = view.size() / 2.0 / grid_size;
                let view = view.map(|x| x.ceil() as i64);

                let ppp = |pos| {
                    let pos = world_to_screen(pos);
                    pos.map(f32::floor)
                };

                let mut grid_geometry = Vec::new();
                let mk_v = |a_pos| draw2d::ColoredVertex {
                    a_pos,
                    a_color: color,
                };

                let thick = helper.editor.config.grid.thick_every as i64;
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
            if let Some(drag) = &helper.editor.drag
                && let DragTarget::SelectionArea { .. } = drag.target
            {
                let color = Color::lerp(helper.theme.dark, helper.theme.highlight, 0.5);
                let selection =
                    Aabb2::from_corners(drag.from_world_raw, helper.editor.cursor_world_pos)
                        .map_bounds(|p| world_to_screen(p.as_f32()));
                let pixel = ctl_render_core::get_pixel_scale(ui_buffer.size());
                let width = 2.0 * pixel;
                self.ui.fill_quad(
                    selection.extend_uniform(width),
                    crate::util::with_alpha(color, 0.5),
                    &mut ui_buffer,
                );
                self.ui
                    .draw_outline(selection, width, color, &mut ui_buffer);
            }

            geng_utils::texture::DrawTexture::new(&self.ui_texture)
                // .fit(screen_aabb, vec2(0.5, 0.5))
                .draw(&geng::PixelPerfectCamera, &self.context.geng, game_buffer);
        }
    }
}

impl RenderHelper<'_> {
    fn draw_telegraph(
        &self,
        tele: &LightTelegraph,
        framebuffer: &mut ugli::Framebuffer,
        util: &UtilRender,
        dithered: bool,
    ) {
        let theme = if dithered { THEME } else { self.theme };
        let mut color = (self.get_color)(tele.light.event_id, theme);
        if !dithered {
            color = color.map_rgb(|x| {
                x * ctl_assets::GraphicsLightsOptions::default()
                    .telegraph_brightness(self.level_editor.model.vfx.palette_swap.current.as_f32())
            }); // telegraph brightness
        }
        util.draw_outline(
            &tele.light.collider,
            0.02,
            color,
            &self.level_editor.model.camera,
            framebuffer,
        );
    }

    fn draw_light(&self, light: &Light, framebuffer: &mut ugli::Framebuffer, util: &UtilRender) {
        let color = (self.get_color)(light.event_id, THEME);
        util.draw_light_gradient(
            &light.collider,
            light.hollow,
            color,
            &self.level_editor.model.camera,
            framebuffer,
        );
    }
}
