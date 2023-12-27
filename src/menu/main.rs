use super::*;

use crate::render::THEME;

pub struct MainMenu {
    geng: Geng,
    assets: Rc<Assets>,
    theme: Theme,
    transition: Option<geng::state::Transition>,
    dither: DitherRender,
    util_render: UtilRender,
    framebuffer_size: vec2<usize>,
    /// Cursor position in screen space.
    cursor_pos: vec2<f64>,
    active_touch: Option<u64>,
    cursor_world_pos: vec2<Coord>,
    camera: Camera2d,
    time: Time,
    play_button: HoverButton,
    player: Player,
    name: String,
}

impl MainMenu {
    pub fn new(geng: &Geng, assets: &Rc<Assets>) -> Self {
        let name: String = preferences::load(PLAYER_NAME_STORAGE).unwrap_or_default();
        let name = fix_name(&name);
        geng.window().start_text_edit(&name);
        Self {
            geng: geng.clone(),
            assets: assets.clone(),
            theme: Theme::default(),
            transition: None,
            dither: DitherRender::new(geng, assets),
            util_render: UtilRender::new(geng, assets),
            framebuffer_size: vec2(1, 1),
            cursor_pos: vec2::ZERO,
            active_touch: None,
            cursor_world_pos: vec2::ZERO,
            camera: Camera2d {
                center: vec2::ZERO,
                rotation: Angle::ZERO,
                fov: 10.0,
            },
            time: Time::ZERO,
            play_button: HoverButton {
                collider: Collider {
                    position: vec2(0.0, 0.0).as_r32(),
                    rotation: Angle::ZERO,
                    shape: Shape::Circle { radius: r32(1.0) },
                },
                hover_time: Lifetime::new(Time::ZERO, Time::ZERO..=r32(1.5)),
            },
            player: Player::new(
                Collider::new(vec2::ZERO, Shape::Circle { radius: r32(1.0) }),
                r32(0.0),
            ),
            name,
        }
    }

    fn play(&mut self) {
        self.geng.window().stop_text_edit();
        self.name = fix_name(&self.name);
        preferences::save(PLAYER_NAME_STORAGE, &self.name);

        let future = {
            let geng = self.geng.clone();
            let assets = self.assets.clone();

            async move {
                let manager = geng.asset_manager();
                let assets_path = run_dir().join("assets");
                let groups_path = assets_path.join("groups");

                let groups = load_groups(manager, &groups_path)
                    .await
                    .expect("failed to load groups");
                LevelMenu::new(&geng, &assets, groups)
            }
            .boxed_local()
        };
        self.transition = Some(geng::state::Transition::Push(Box::new(
            geng::LoadingScreen::new(
                &self.geng,
                geng::EmptyLoadingScreen::new(&self.geng),
                future,
            ),
        )));
    }
}

impl geng::State for MainMenu {
    fn transition(&mut self) -> Option<geng::state::Transition> {
        self.transition.take()
    }

    fn update(&mut self, delta_time: f64) {
        // In case we come back to that state after playing the game
        if !self.geng.window().is_editing_text() {
            self.geng.window().start_text_edit(&self.name);
        }

        let delta_time = Time::new(delta_time as f32);
        self.time += delta_time;

        let pos = self.cursor_pos.as_f32();
        let game_pos = geng_utils::layout::fit_aabb(
            self.dither.get_render_size().as_f32(),
            Aabb2::ZERO.extend_positive(self.framebuffer_size.as_f32()),
            vec2(0.5, 0.5),
        );
        let pos = pos - game_pos.bottom_left();
        self.cursor_world_pos = self.camera.screen_to_world(game_pos.size(), pos).as_r32();

        self.player.collider.position = self.cursor_world_pos;
        self.player.reset_distance();

        self.play_button.hover_time.change(
            if self.player.collider.check(&self.play_button.collider) {
                delta_time
            } else {
                -delta_time
            },
        );
        self.player
            .update_distance(&self.play_button.collider, false);
        if self.play_button.hover_time.is_max() {
            self.play_button.hover_time.set_ratio(Time::ZERO);
            self.play();
        }
    }

    fn fixed_update(&mut self, delta_time: f64) {
        let delta_time = Time::new(delta_time as _);
        self.player.update_tail(delta_time);
    }

    fn handle_event(&mut self, event: geng::Event) {
        match event {
            geng::Event::EditText(text) => {
                self.name = text;
                self.name = self.name.to_lowercase();
                // self.name.retain(|c| self.assets.font.can_render(c));
                self.name = self.name.chars().take(10).collect();
                self.geng.window().start_text_edit(&self.name);
            }
            geng::Event::CursorMove { position } => {
                self.cursor_pos = position;
            }
            geng::Event::TouchStart(touch) if self.active_touch.is_none() => {
                self.active_touch = Some(touch.id);
            }
            geng::Event::TouchMove(touch) if Some(touch.id) == self.active_touch => {
                self.cursor_pos = touch.position;
            }
            geng::Event::TouchEnd(touch) if Some(touch.id) == self.active_touch => {
                self.active_touch = None;
            }
            _ => {}
        }
    }

    fn draw(&mut self, screen_buffer: &mut ugli::Framebuffer) {
        self.framebuffer_size = screen_buffer.size();
        ugli::clear(screen_buffer, Some(self.theme.dark), None, None);

        let mut framebuffer = self.dither.start();

        let button = crate::render::smooth_button(&self.play_button, self.time + r32(0.5));
        self.util_render
            .draw_button(&button, "START", &THEME, &self.camera, &mut framebuffer);

        let fading = self.play_button.hover_time.get_ratio().as_f32() > 0.5;

        if !fading {
            geng_utils::texture::DrawTexture::new(&self.assets.sprites.title)
                .fit_height(
                    Aabb2::point(vec2(0.0, 3.5)).extend_symmetric(vec2(0.0, 1.2) / 2.0),
                    0.5,
                )
                .colored(THEME.light)
                .draw(&self.camera, &self.geng, &mut framebuffer);
            // self.util_render.draw_text(
            //     "CLOSE TO LIGHT",
            //     vec2(0.0, 3.5).as_r32(),
            //     1.2,
            //     vec2::splat(0.5),
            //     crate::render::COLOR_LIGHT,
            //     &self.camera,
            //     &mut framebuffer,
            // );

            self.util_render
                .draw_player(&self.player, &self.camera, &mut framebuffer);

            // Name
            self.util_render.draw_text(
                &self.name,
                vec2(0.0, -3.0).as_r32(),
                TextRenderOptions::new(0.8).color(THEME.light),
                &self.camera,
                &mut framebuffer,
            );
            self.util_render.draw_text(
                "TYPE YOUR NAME",
                vec2(0.0, -3.8).as_r32(),
                TextRenderOptions::new(0.7).color(THEME.light),
                &self.camera,
                &mut framebuffer,
            );
        }

        self.dither.finish(self.time, &self.theme);

        let aabb = Aabb2::ZERO.extend_positive(screen_buffer.size().as_f32());
        geng_utils::texture::DrawTexture::new(self.dither.get_buffer())
            .fit(aabb, vec2(0.5, 0.5))
            .draw(&geng::PixelPerfectCamera, &self.geng, screen_buffer);
    }
}

fn fix_name(name: &str) -> String {
    name.trim().to_lowercase()
}
