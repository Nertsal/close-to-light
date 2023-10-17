use crate::{
    prelude::*,
    render::{Render, UtilRender},
};

pub struct MainMenu {
    geng: Geng,
    assets: Rc<Assets>,
    config: Config,
    theme: LevelTheme,
    transition: Option<geng::state::Transition>,
    render: Render,
    util_render: UtilRender,
    framebuffer_size: vec2<usize>,
    /// Cursor position in screen space.
    cursor_pos: vec2<f64>,
    active_touch: Option<u64>,
    cursor_world_pos: vec2<Coord>,
    camera: Camera2d,
    time: Time,
    play_button: HoverButton,
    player: Collider,
    name: String,
}

impl MainMenu {
    pub fn new(geng: &Geng, assets: &Rc<Assets>, config: Config) -> Self {
        let name: String = preferences::load("name").unwrap_or_default();
        geng.window().start_text_edit(&name);
        Self {
            geng: geng.clone(),
            assets: assets.clone(),
            theme: LevelTheme::default(),
            transition: None,
            render: Render::new(geng, assets),
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
            player: Collider::new(
                vec2::ZERO,
                Shape::Circle {
                    radius: r32(config.player.radius),
                },
            ),
            config,
            name,
        }
    }

    fn play(&mut self) {
        self.name = self.name.trim().to_string();

        self.geng.window().stop_text_edit();
        preferences::save("name", &self.name);

        let future = {
            let geng = self.geng.clone();
            let assets = self.assets.clone();
            let config = self.config.clone();
            let player_name = self.name.clone();

            async move {
                let manager = geng.asset_manager();
                let assets_path = run_dir().join("assets");

                let level: Level =
                    geng::asset::Load::load(manager, &assets_path.join("level.json"), &())
                        .await
                        .expect("failed to load level");

                let secrets: Option<crate::Secrets> =
                    geng::asset::Load::load(manager, &run_dir().join("secrets.toml"), &())
                        .await
                        .ok();
                let secrets = secrets.or_else(|| {
                    Some(crate::Secrets {
                        leaderboard: crate::LeaderboardSecrets {
                            id: option_env!("LEADERBOARD_ID")?.to_string(),
                            key: option_env!("LEADERBOARD_KEY")?.to_string(),
                        },
                    })
                });

                crate::game::Game::new(
                    &geng,
                    &assets,
                    config,
                    level,
                    secrets.map(|s| s.leaderboard),
                    player_name,
                    Time::ZERO,
                )
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
            self.render.get_render_size().as_f32(),
            Aabb2::ZERO.extend_positive(self.framebuffer_size.as_f32()),
            vec2(0.5, 0.5),
        );
        let pos = pos - game_pos.bottom_left();
        self.cursor_world_pos = self.camera.screen_to_world(game_pos.size(), pos).as_r32();

        self.player.position = self.cursor_world_pos;

        self.play_button
            .hover_time
            .change(if self.player.check(&self.play_button.collider) {
                delta_time
            } else {
                -delta_time
            });
        if self.play_button.hover_time.is_max() {
            self.play_button.hover_time.set_ratio(Time::ZERO);
            self.play();
        }
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

        let mut framebuffer = self.render.start(self.theme.dark);

        let button = crate::render::smooth_button(&self.play_button, self.time + r32(0.5));
        self.util_render.draw_button(
            &button,
            "START",
            &self.theme,
            &self.camera,
            &mut framebuffer,
        );

        let fading = self.play_button.hover_time.get_ratio().as_f32() > 0.5;

        if !fading {
            geng_utils::texture::draw_texture_fit_height(
                &self.assets.title,
                Aabb2::point(vec2(0.0, 3.5)).extend_symmetric(vec2(0.0, 1.2) / 2.0),
                0.5,
                &self.camera,
                &self.geng,
                &mut framebuffer,
            );
            // self.util_render.draw_text(
            //     "CLOSE TO LIGHT",
            //     vec2(0.0, 3.5).as_r32(),
            //     1.2,
            //     vec2::splat(0.5),
            //     crate::render::COLOR_LIGHT,
            //     &self.camera,
            //     &mut framebuffer,
            // );

            self.util_render.draw_outline(
                &self.player,
                0.05,
                self.theme.player,
                &self.camera,
                &mut framebuffer,
            );

            // Name
            self.util_render.draw_text(
                &self.name,
                vec2(0.0, -3.0).as_r32(),
                0.8,
                vec2::splat(0.5),
                self.theme.light,
                &self.camera,
                &mut framebuffer,
            );
            self.util_render.draw_text(
                "TYPE YOUR NAME",
                vec2(0.0, -3.8).as_r32(),
                0.7,
                vec2::splat(0.5),
                self.theme.light,
                &self.camera,
                &mut framebuffer,
            );
        }

        self.render.dither(self.time, R32::ZERO); // TODO

        let aabb = Aabb2::ZERO.extend_positive(screen_buffer.size().as_f32());
        geng_utils::texture::draw_texture_fit(
            self.render.get_buffer(),
            aabb,
            vec2(0.5, 0.5),
            &geng::PixelPerfectCamera,
            &self.geng,
            screen_buffer,
        );
    }
}
