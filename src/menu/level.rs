mod ui;

pub use self::ui::*;

use super::*;

use crate::render::menu::MenuRender;

use geng::MouseButton;

pub struct LevelMenu {
    geng: Geng,
    assets: Rc<Assets>,
    transition: Option<geng::state::Transition>,
    render: MenuRender,
    util: UtilRender,
    dither: DitherRender,

    framebuffer_size: vec2<usize>,
    last_delta_time: Time,
    time: Time,

    ui: MenuUI,
    cursor_pos: vec2<f64>,

    camera: Camera2d,
    state: MenuState,
    exit_button: HoverButton,
}

#[derive(Debug)]
pub struct MenuState {
    pub theme: Theme,
    pub groups: Vec<GroupEntry>,
    /// Currently showing group.
    pub show_group: Option<ShowTime<usize>>,
    /// Switch to the group after current one finishes its animation.
    pub switch_group: Option<usize>,
    /// Currently showing level of the active group.
    pub show_level: Option<ShowTime<usize>>,
    /// Switch to the level of the active group after current one finishes its animation.
    pub switch_level: Option<usize>,
    pub show_level_config: ShowTime<()>,
    pub show_leaderboard: ShowTime<()>,
    play_level: Option<(std::path::PathBuf, LevelConfig)>,
}

#[derive(Debug, Clone)]
pub struct ShowTime<T> {
    pub data: T,
    pub time: Bounded<Time>,
    /// Whether the time is going up or down.
    pub going_up: bool,
}

pub struct GroupEntry {
    pub meta: GroupMeta,
    pub levels: Vec<(std::path::PathBuf, LevelMeta)>,
    pub logo: Option<ugli::Texture>,
}

impl Debug for GroupEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GroupEntry")
            .field("meta", &self.meta)
            .field("logo", &self.logo.as_ref().map(|_| "<logo>"))
            .field("levels", &self.levels)
            .finish()
    }
}

impl MenuState {
    fn show_group(&mut self, group: usize) {
        self.switch_group = Some(group);
    }

    fn show_level(&mut self, level: Option<usize>) {
        self.switch_level = level;
    }

    fn show_leaderboard(&mut self) {
        self.show_leaderboard.going_up = true;
    }

    fn play_level(&mut self, level: std::path::PathBuf, config: LevelConfig) {
        self.play_level = Some((level, config));
    }
}

impl LevelMenu {
    pub fn new(geng: &Geng, assets: &Rc<Assets>, groups: Vec<GroupEntry>) -> Self {
        Self {
            geng: geng.clone(),
            assets: assets.clone(),
            transition: None,
            render: MenuRender::new(geng, assets),
            util: UtilRender::new(geng, assets),
            dither: DitherRender::new(geng, assets),

            framebuffer_size: vec2(1, 1),
            last_delta_time: Time::ONE,
            time: Time::ZERO,

            ui: MenuUI::new(assets),
            cursor_pos: vec2::ZERO,

            camera: Camera2d {
                center: vec2::ZERO,
                rotation: Angle::ZERO,
                fov: 10.0,
            },
            state: MenuState {
                theme: Theme::default(),
                groups,
                show_group: None,
                switch_group: None,
                show_level: None,
                switch_level: None,
                show_level_config: ShowTime {
                    data: (),
                    time: Bounded::new_zero(r32(0.3)),
                    going_up: false,
                },
                show_leaderboard: ShowTime {
                    data: (),
                    time: Bounded::new_zero(r32(0.3)),
                    going_up: false,
                },
                play_level: None,
            },
            exit_button: HoverButton::new(
                Collider::new(vec2(-7.6, 3.7).as_r32(), Shape::Circle { radius: r32(0.6) }),
                3.0,
            ),
        }
    }

    fn play_level(&mut self, level_path: std::path::PathBuf, config: LevelConfig) {
        let future = {
            let geng = self.geng.clone();
            let assets = self.assets.clone();
            let player_name: String = preferences::load(PLAYER_NAME_STORAGE).unwrap_or_default();

            async move {
                let manager = geng.asset_manager();

                let (_, level_music, level) = load_level(manager, &level_path)
                    .await
                    .expect("failed to load level");

                let secrets: Option<crate::Secrets> =
                    geng::asset::Load::load(manager, &run_dir().join("secrets.toml"), &())
                        .await
                        .ok();
                let secrets = secrets.or_else(|| {
                    Some(crate::Secrets {
                        leaderboard: crate::LeaderboardSecrets {
                            url: option_env!("LEADERBOARD_URL")?.to_string(),
                            key: option_env!("LEADERBOARD_KEY")?.to_string(),
                        },
                    })
                });

                let (group_name, level_name) = crate::group_level_from_path(level_path);
                let level = crate::game::PlayLevel {
                    group_name,
                    level_name,
                    config,
                    level,
                    music: level_music,
                    start_time: Time::ZERO,
                };
                crate::game::Game::new(
                    &geng,
                    &assets,
                    level,
                    secrets.map(|s| s.leaderboard),
                    player_name,
                )
            }
        };
        self.transition = Some(geng::state::Transition::Push(Box::new(
            geng::LoadingScreen::new(
                &self.geng,
                geng::EmptyLoadingScreen::new(&self.geng),
                future,
            ),
        )));
    }

    fn update_active_group(&mut self, delta_time: Time) {
        if let Some(current_group) = &mut self.state.show_group {
            if let Some(switch_group) = self.state.switch_group {
                if current_group.data != switch_group {
                    // Change level first
                    self.state.switch_level = None;
                    // if self.state.show_level.is_some() {
                    //     return;
                    // }

                    current_group.time.change(-delta_time);
                    current_group.going_up = false;

                    if current_group.time.is_min() {
                        // Switch
                        current_group.data = switch_group;
                    }
                } else {
                    current_group.time.change(delta_time);
                    current_group.going_up = true;
                }
            } else {
                current_group.time.change(-delta_time);
                current_group.going_up = false;

                if current_group.time.is_min() {
                    // Remove
                    self.state.show_group = None;
                }
            }
        } else if let Some(group) = self.state.switch_group.take() {
            self.state.show_group = Some(ShowTime {
                data: group,
                time: Bounded::new_zero(r32(0.25)),
                going_up: false,
            });
        }
    }

    fn update_active_level(&mut self, delta_time: Time) {
        if let Some(current_level) = &mut self.state.show_level {
            if let Some(switch_level) = self.state.switch_level {
                if current_level.data != switch_level {
                    self.state.show_leaderboard.going_up = false; // Hide leaderboard
                    current_level.time.change(-delta_time);
                    current_level.going_up = false;

                    if current_level.time.is_min() {
                        // Switch
                        current_level.data = switch_level;
                    }
                } else {
                    current_level.time.change(delta_time);
                    current_level.going_up = true;
                }
            } else {
                current_level.time.change(-delta_time);
                current_level.going_up = false;

                if current_level.time.is_min() {
                    // Remove
                    self.state.show_level = None;
                }
            }
        } else if let Some(level) = self.state.switch_level.take() {
            self.state.show_level = Some(ShowTime {
                data: level,
                time: Bounded::new_zero(r32(0.25)),
                going_up: false,
            });
        }
    }

    fn update_leaderboard(&mut self, delta_time: Time) {
        let board = &mut self.state.show_leaderboard;
        if self.state.show_level.is_none() {
            board.going_up = false;
        }
        // TODO start fetching somewhere
        let sign = r32(if board.going_up { 1.0 } else { -1.0 });
        board.time.change(sign * delta_time);
    }
}

impl geng::State for LevelMenu {
    fn transition(&mut self) -> Option<geng::state::Transition> {
        self.transition.take()
    }

    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        self.framebuffer_size = framebuffer.size();
        ugli::clear(framebuffer, Some(self.state.theme.dark), None, None);

        let mut dither_buffer = self.dither.start();
        let button = crate::render::smooth_button(&self.exit_button, self.time);
        self.util.draw_button(
            &button,
            "EXIT",
            &crate::render::THEME,
            &self.camera,
            &mut dither_buffer,
        );
        self.dither.finish(self.time, &Theme::default());

        geng_utils::texture::DrawTexture::new(self.dither.get_buffer())
            .fit_screen(vec2(0.5, 0.5), framebuffer)
            .draw(&geng::PixelPerfectCamera, &self.geng, framebuffer);

        if self.exit_button.is_fading() {
            return;
        }

        self.ui.layout(
            &mut self.state,
            Aabb2::ZERO.extend_positive(framebuffer.size().as_f32()),
            self.cursor_pos.as_f32(),
            geng_utils::key::is_key_pressed(self.geng.window(), [MouseButton::Left]),
            self.last_delta_time.as_f32(),
            &self.geng,
        );
        self.render.draw_ui(&self.ui, &self.state, framebuffer);

        if let Some((level, config)) = self.state.play_level.take() {
            self.play_level(level, config);
        }
    }

    fn handle_event(&mut self, event: geng::Event) {
        match event {
            geng::Event::KeyPress {
                key: geng::Key::Escape,
            } => {
                if self.state.switch_group.take().is_some() {
                } else {
                    // Go to main menu
                    self.transition = Some(geng::state::Transition::Pop);
                }
            }
            geng::Event::CursorMove { position } => {
                self.cursor_pos = position;
            }
            _ => (),
        }
    }

    fn update(&mut self, delta_time: f64) {
        let delta_time = Time::new(delta_time as f32);
        self.time += delta_time;

        let cursor_world = self
            .camera
            .screen_to_world(self.framebuffer_size.as_f32(), self.cursor_pos.as_f32());
        let hovering = self
            .exit_button
            .base_collider
            .contains(cursor_world.as_r32());
        self.exit_button.update(hovering, delta_time);
        if self.exit_button.hover_time.is_max() {
            self.transition = Some(geng::state::Transition::Pop);
        }

        self.update_active_group(delta_time);
        self.update_active_level(delta_time);
        self.update_leaderboard(delta_time);

        self.last_delta_time = delta_time;
    }
}
