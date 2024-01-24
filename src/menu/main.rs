use super::*;

use crate::{
    render::{ui::UiRender, THEME},
    Secrets,
};

pub struct MainMenu {
    geng: Geng,
    assets: Rc<Assets>,
    secrets: Option<Secrets>,
    options: Options,
    transition: Option<geng::state::Transition>,
    client: Option<Arc<ctl_client::Nertboard>>,

    dither: DitherRender,
    util_render: UtilRender,
    ui_render: UiRender,

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
    password: String,
    text_edit: TextEdit<TextTarget>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TextTarget {
    Username,
    Password,
}

pub struct TextEdit<S> {
    pub geng: Geng,
    pub state: Option<TextEditState<S>>,
}

#[derive(Debug, Clone)]
pub struct TextEditState<S> {
    pub status: S,
    pub text: String,
    pub options: TextEditOptions,
}

#[derive(Debug, Clone)]
pub struct TextEditOptions {
    pub max_len: usize,
    pub lowercase: bool,
}

impl Default for TextEditOptions {
    fn default() -> Self {
        Self {
            max_len: 10,
            lowercase: true,
        }
    }
}

impl<S> TextEdit<S> {
    pub fn new(geng: &Geng) -> Self {
        Self {
            geng: geng.clone(),
            state: None,
        }
    }

    pub fn start(&mut self, status: S, text: String, options: TextEditOptions) {
        let mut state = TextEditState::new(status, options);
        state.update(text);
        self.geng.window().start_text_edit(&state.text);
        self.state = Some(state);
    }

    pub fn stop(&mut self) -> Option<String> {
        let state = self.state.take()?;
        self.geng.window().stop_text_edit();
        Some(state.text)
    }

    pub fn update(&mut self, text: String) {
        let Some(state) = &mut self.state else {
            log::error!("Editing text but state says otherwise");
            self.geng.window().stop_text_edit();
            return;
        };

        state.update(text);
        self.geng.window().start_text_edit(&state.text);
    }
}

impl<S> TextEditState<S> {
    pub fn new(status: S, options: TextEditOptions) -> Self {
        Self {
            status,
            text: String::new(),
            options,
        }
    }

    pub fn update(&mut self, mut text: String) {
        if self.options.lowercase {
            text = text.to_lowercase();
        }
        text = text.chars().take(self.options.max_len).collect();
        self.text = text;
    }
}

impl MainMenu {
    pub fn new(
        geng: &Geng,
        assets: &Rc<Assets>,
        secrets: Option<Secrets>,
        options: Options,
    ) -> Self {
        let name: String = preferences::load(PLAYER_NAME_STORAGE).unwrap_or_default();
        let name = fix_name(&name);
        geng.window().start_text_edit(&name);
        Self {
            geng: geng.clone(),
            assets: assets.clone(),
            transition: None,
            client: secrets.as_ref().map(|secrets| {
                Arc::new(
                    ctl_client::Nertboard::new(
                        &secrets.leaderboard.url,
                        Some(secrets.leaderboard.key.clone()),
                    )
                    .unwrap(),
                )
            }),
            secrets,
            options,

            dither: DitherRender::new(geng, assets),
            util_render: UtilRender::new(geng, assets),
            ui_render: UiRender::new(geng, assets),

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
            play_button: HoverButton::new(
                Collider {
                    position: vec2(0.0, 0.0).as_r32(),
                    rotation: Angle::ZERO,
                    shape: Shape::Circle { radius: r32(1.0) },
                },
                1.5,
            ),
            player: Player::new(
                Collider::new(vec2::ZERO, Shape::Circle { radius: r32(1.0) }),
                r32(0.0),
            ),
            name,
            password: String::new(),
            text_edit: TextEdit::new(geng),
        }
    }

    fn play(&mut self) {
        self.text_edit.stop();
        self.name = fix_name(&self.name);
        preferences::save(PLAYER_NAME_STORAGE, &self.name);

        let future = {
            let geng = self.geng.clone();
            let assets = self.assets.clone();
            let secrets = self.secrets.clone();
            let options = self.options.clone();

            async move {
                let manager = geng.asset_manager();
                let assets_path = run_dir().join("assets");
                let groups_path = assets_path.join("groups");

                let groups = load_groups(manager, &groups_path)
                    .await
                    .expect("failed to load groups");
                LevelMenu::new(&geng, &assets, groups, secrets, options)
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

    fn edit_username(&mut self) {
        self.text_edit.start(
            TextTarget::Username,
            self.name.clone(),
            TextEditOptions {
                max_len: 10,
                lowercase: true,
            },
        );
    }

    fn edit_password(&mut self) {
        self.text_edit.start(
            TextTarget::Password,
            self.password.clone(),
            TextEditOptions {
                max_len: 20,
                lowercase: false,
            },
        );
    }
}

impl geng::State for MainMenu {
    fn transition(&mut self) -> Option<geng::state::Transition> {
        self.transition.take()
    }

    fn update(&mut self, delta_time: f64) {
        // In case we come back to that state after playing the game
        if self.text_edit.state.is_none() {
            self.edit_username();
        }
        if let Some(state) = &self.text_edit.state {
            match state.status {
                TextTarget::Username => self.name = state.text.clone(),
                TextTarget::Password => self.password = state.text.clone(),
            }
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

        let hovering = self.player.collider.check(&self.play_button.base_collider);
        self.play_button.update(hovering, delta_time);
        self.player
            .update_distance_simple(&self.play_button.base_collider);
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
                self.text_edit.update(text);
            }
            geng::Event::KeyPress { key } => match key {
                geng::Key::Tab => {
                    if let Some(state) = &self.text_edit.state {
                        match state.status {
                            TextTarget::Username => self.edit_password(),
                            TextTarget::Password => self.edit_username(),
                        };
                    }
                }
                geng::Key::Enter => {
                    if let Some(client) = self.client.clone() {
                        let creds = ctl_client::core::auth::Credentials {
                            username: self.name.clone(),
                            password: self.password.clone(),
                        };
                        let mut task = crate::task::Task::new(async move {
                            client.register(&creds).await?;
                            client.login(&creds).await?;
                            anyhow::Ok(())
                        });
                        while task.poll().is_none() {}
                    }
                }
                _ => {}
            },
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
        ugli::clear(screen_buffer, Some(self.options.theme.dark), None, None);

        let mut framebuffer = self.dither.start();

        let button = crate::render::smooth_button(&self.play_button, self.time + r32(0.5));
        self.util_render
            .draw_button(&button, "START", &THEME, &self.camera, &mut framebuffer);

        if !self.play_button.is_fading() {
            if let Some(pos) = self
                .camera
                .world_to_screen(framebuffer.size().as_f32(), vec2(0.0, 3.5))
            {
                self.ui_render.draw_texture(
                    Aabb2::point(pos).extend_symmetric(vec2(0.0, 1.2) / 2.0),
                    &self.assets.sprites.title,
                    THEME.light,
                    &mut framebuffer,
                );
            }

            self.util_render
                .draw_player(&self.player, &self.camera, &mut framebuffer);
        }

        self.dither.finish(self.time, &self.options.theme);

        let aabb = Aabb2::ZERO.extend_positive(screen_buffer.size().as_f32());
        geng_utils::texture::DrawTexture::new(self.dither.get_buffer())
            .fit(aabb, vec2(0.5, 0.5))
            .draw(&geng::PixelPerfectCamera, &self.geng, screen_buffer);

        if !self.play_button.is_fading() {
            let normal_color = self.options.theme.light;
            let active_color = self.options.theme.highlight;

            let get_color = |target: TextTarget| {
                let active = self
                    .text_edit
                    .state
                    .as_ref()
                    .map_or(false, |state| state.status == target);
                if active {
                    active_color
                } else {
                    normal_color
                }
            };

            // Name
            let color = get_color(TextTarget::Username);
            self.util_render.draw_text(
                &self.name,
                vec2(0.0, -3.0).as_r32(),
                TextRenderOptions::new(0.8).color(color),
                &self.camera,
                screen_buffer,
            );
            self.util_render.draw_text(
                "TYPE YOUR NAME",
                vec2(0.0, -3.8).as_r32(),
                TextRenderOptions::new(0.7).color(color),
                &self.camera,
                screen_buffer,
            );

            // Password
            let color = get_color(TextTarget::Password);
            let password = "*".repeat(self.password.len());
            self.util_render.draw_text(
                format!("Password: {}", password),
                vec2(3.0, -4.0),
                TextRenderOptions::new(0.7)
                    .align(vec2(0.0, 0.5))
                    .color(color),
                &self.camera,
                screen_buffer,
            );
        }
    }
}

fn fix_name(name: &str) -> String {
    name.trim().to_lowercase()
}
