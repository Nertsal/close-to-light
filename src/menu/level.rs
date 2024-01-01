mod ui;

pub use self::ui::*;

use super::*;

use crate::{
    leaderboard::{Leaderboard, LeaderboardStatus},
    render::menu::MenuRender,
    ui::widget::CursorContext,
    Secrets,
};

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
    ui_focused: bool,
    cursor: CursorContext,

    camera: Camera2d,
    state: MenuState,
    player: Player,
    exit_button: HoverButton,
    play_button: HoverButton,
}

pub struct MenuState {
    pub leaderboard: Leaderboard,
    pub options: Options,
    pub config: LevelConfig,
    pub groups: Vec<GroupEntry>,
    /// Currently showing group.
    pub show_group: Option<ShowTime<usize>>,
    /// Switch to the group after current one finishes its animation.
    pub switch_group: Option<usize>,
    /// Currently showing level of the active group.
    pub show_level: Option<ShowTime<usize>>,
    /// Switch to the level of the active group after current one finishes its animation.
    pub switch_level: Option<usize>,
    /// Whether the level configuration and leaderboard screen should be up right now.
    pub level_up: bool,
    pub show_options: ShowTime<()>,
    pub options_request: Option<WidgetRequest>,
    pub show_level_config: ShowTime<()>,
    pub config_request: Option<WidgetRequest>,
    pub show_leaderboard: ShowTime<()>,
    pub leaderboard_request: Option<WidgetRequest>,
}

#[derive(Debug, Clone)]
pub struct ShowTime<T> {
    pub data: T,
    pub time: Bounded<Time>,
    /// Whether the time is going up or down.
    pub going_up: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum WidgetRequest {
    Open,
    Close,
    Reload,
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
}

impl LevelMenu {
    pub fn new(
        geng: &Geng,
        assets: &Rc<Assets>,
        groups: Vec<GroupEntry>,
        secrets: Option<Secrets>,
        options: Options,
    ) -> Self {
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
            ui_focused: false,
            cursor: CursorContext::new(),

            camera: Camera2d {
                center: vec2::ZERO,
                rotation: Angle::ZERO,
                fov: 10.0,
            },
            state: MenuState {
                leaderboard: Leaderboard::new(secrets.map(|s| s.leaderboard)),
                options,
                config: LevelConfig::default(),
                groups,
                show_group: None,
                switch_group: None,
                show_level: None,
                switch_level: None,
                level_up: false,
                show_options: ShowTime {
                    data: (),
                    time: Bounded::new_zero(r32(0.3)),
                    going_up: false,
                },
                options_request: None,
                show_level_config: ShowTime {
                    data: (),
                    time: Bounded::new_zero(r32(0.3)),
                    going_up: false,
                },
                config_request: None,
                show_leaderboard: ShowTime {
                    data: (),
                    time: Bounded::new_zero(r32(0.3)),
                    going_up: false,
                },
                leaderboard_request: None,
            },
            player: Player::new(
                Collider::new(vec2::ZERO, Shape::Circle { radius: r32(1.0) }),
                r32(0.0),
            ),
            exit_button: HoverButton::new(
                Collider::new(vec2(-7.6, 3.7).as_r32(), Shape::Circle { radius: r32(0.6) }),
                3.0,
            ),
            play_button: HoverButton::new(
                Collider {
                    position: vec2(6.0, 0.0).as_r32(),
                    rotation: Angle::ZERO,
                    shape: Shape::Circle { radius: r32(0.8) },
                },
                1.5,
            ),
        }
    }

    fn get_active_level(&self) -> Option<std::path::PathBuf> {
        if let Some(group) = &self.state.show_group {
            if let Some(group) = self.state.groups.get(group.data) {
                if let Some(level) = &self.state.show_level {
                    if let Some((path, _)) = group.levels.get(level.data) {
                        return Some(path.clone());
                    }
                }
            }
        }
        None
    }

    fn play_level(&mut self) {
        let Some(level_path) = self.get_active_level() else {
            log::error!("Trying to play a level, but there is no active level");
            return;
        };

        self.cursor.position = vec2::ZERO;
        self.play_button.hover_time.set(Time::ZERO);

        let future = {
            let geng = self.geng.clone();
            let assets = self.assets.clone();
            let leaderboard = self.state.leaderboard.clone();
            let options = self.state.options.clone();
            let config = self.state.config.clone();
            let player_name: String = preferences::load(PLAYER_NAME_STORAGE).unwrap_or_default();

            async move {
                let manager = geng.asset_manager();

                let (_, level_music, level) = load_level(manager, &level_path)
                    .await
                    .expect("failed to load level");

                let (group_name, level_name) = crate::group_level_from_path(level_path);
                let level = crate::game::PlayLevel {
                    group_name,
                    level_name,
                    config,
                    level,
                    music: level_music,
                    start_time: Time::ZERO,
                };
                crate::game::Game::new(&geng, &assets, options, level, leaderboard, player_name)
            }
        };
        self.transition = Some(geng::state::Transition::Push(Box::new(
            geng::LoadingScreen::new(
                &self.geng,
                geng::EmptyLoadingScreen::new(&self.geng),
                future,
            ),
        )));
        // Queue leaderboard fetch when coming back
        self.state.leaderboard.status = LeaderboardStatus::None;
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
        } else if let Some(group) = self.state.switch_group {
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

                    if current_level.time.is_max() {
                        self.state.level_up = true;
                    }
                }
            } else {
                self.state.level_up = false;
                current_level.time.change(-delta_time);
                current_level.going_up = false;

                if current_level.time.is_min() {
                    // Remove
                    self.state.show_level = None;
                }
            }
        } else if let Some(level) = self.state.switch_level {
            self.state.show_level = Some(ShowTime {
                data: level,
                time: Bounded::new_zero(r32(0.5)),
                going_up: false,
            });
        }
    }

    fn fetch_leaderboard(&mut self) {
        if let Some(group) = &self.state.show_group {
            if let Some(group) = self.state.groups.get(group.data) {
                if let Some(level) = &self.state.show_level {
                    if let Some((path, _)) = group.levels.get(level.data) {
                        let (group, level) = crate::group_level_from_path(path);

                        let mods = self.state.config.modifiers.clone();
                        let health = self.state.config.health.clone();

                        let meta = crate::leaderboard::ScoreMeta::new(group, level, mods, health);
                        self.state.leaderboard.change_meta(meta);
                    }
                }
            }
        }
    }

    fn update_leaderboard(&mut self, delta_time: Time) {
        if let Some(req) = self.state.leaderboard_request.take() {
            let board = &mut self.state.show_leaderboard;
            match req {
                WidgetRequest::Open => {
                    if board.time.is_min() {
                        board.going_up = true;
                        self.state.leaderboard_request = None;
                        self.fetch_leaderboard();
                    }
                }
                WidgetRequest::Close => board.going_up = false,
                WidgetRequest::Reload => {
                    self.fetch_leaderboard();
                }
            }
        }

        let board = &mut self.state.show_leaderboard;
        if self.state.show_level.is_none() {
            board.going_up = false;
        }
        let sign = r32(if board.going_up { 1.0 } else { -1.0 });
        board.time.change(sign * delta_time);
    }

    fn update_config(&mut self, delta_time: Time) {
        if let Some(req) = self.state.config_request {
            let config = &mut self.state.show_level_config;
            match req {
                WidgetRequest::Open => {
                    if config.time.is_min() {
                        config.going_up = true;
                        self.state.config_request = None;
                    }
                }
                WidgetRequest::Close => config.going_up = false,
                WidgetRequest::Reload => {
                    config.going_up = false;
                    if config.time.is_min() {
                        self.state.config_request = Some(WidgetRequest::Open);
                    }
                }
            }
        }

        let config = &mut self.state.show_level_config;
        if self.state.show_level.is_none() {
            config.going_up = false;
        }
        let sign = r32(if config.going_up { 1.0 } else { -1.0 });
        config.time.change(sign * delta_time);
    }

    fn update_options(&mut self, delta_time: Time) {
        if let Some(req) = self.state.options_request {
            let options = &mut self.state.show_options;
            match req {
                WidgetRequest::Open => {
                    if options.time.is_min() {
                        options.going_up = true;
                        self.state.options_request = None;
                        self.state.show_level_config.going_up = false;
                        self.state.show_leaderboard.going_up = false;
                    }
                }
                WidgetRequest::Close => options.going_up = false,
                WidgetRequest::Reload => {
                    options.going_up = false;
                    if options.time.is_min() {
                        self.state.options_request = Some(WidgetRequest::Open);
                    }
                }
            }
        }

        let options = &mut self.state.show_options;
        let sign = r32(if options.going_up { 1.0 } else { -1.0 });
        options.time.change(sign * delta_time);
    }
}

impl geng::State for LevelMenu {
    fn transition(&mut self) -> Option<geng::state::Transition> {
        self.transition.take()
    }

    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        self.framebuffer_size = framebuffer.size();
        ugli::clear(framebuffer, Some(self.state.options.theme.dark), None, None);

        let mut dither_buffer = self.dither.start();

        let fading = self.exit_button.is_fading() || self.play_button.is_fading();

        if !fading || self.exit_button.is_fading() {
            let button = crate::render::smooth_button(&self.exit_button, self.time);
            self.util.draw_button(
                &button,
                "EXIT",
                &crate::render::THEME,
                &self.camera,
                &mut dither_buffer,
            );
        }

        if !fading || self.play_button.is_fading() {
            let play_time = self
                .state
                .show_level
                .as_ref()
                .map_or(Time::ZERO, |show| show.time.get_ratio());
            let scale = crate::util::smoothstep(play_time);
            let mut button = self.play_button.clone();
            button.base_collider = button.base_collider.transformed(Transform::scale(scale));
            self.util.draw_button(
                &button,
                "PLAY",
                &crate::render::THEME,
                &self.camera,
                &mut dither_buffer,
            );
        }

        self.util
            .draw_player(&self.player, &self.camera, &mut dither_buffer);

        self.dither.finish(self.time, &self.state.options.theme);

        geng_utils::texture::DrawTexture::new(self.dither.get_buffer())
            .fit_screen(vec2(0.5, 0.5), framebuffer)
            .draw(&geng::PixelPerfectCamera, &self.geng, framebuffer);

        if fading {
            return;
        }

        self.ui_focused = self.ui.layout(
            &mut self.state,
            Aabb2::ZERO.extend_positive(framebuffer.size().as_f32()),
            self.cursor,
            self.last_delta_time.as_f32(),
            &self.geng,
        );
        self.cursor.scroll = 0.0;
        self.render.draw_ui(&self.ui, &self.state, framebuffer);
    }

    fn handle_event(&mut self, event: geng::Event) {
        match event {
            geng::Event::KeyPress {
                key: geng::Key::Escape,
            } => {
                if self.state.show_leaderboard.time.is_max() {
                    self.state.leaderboard_request = Some(WidgetRequest::Close);
                } else if self.state.show_level_config.time.is_max() {
                    self.state.config_request = Some(WidgetRequest::Close);
                } else if self.state.switch_level.take().is_some()
                    || self.state.switch_group.take().is_some()
                {
                } else {
                    // Go to main menu
                    self.transition = Some(geng::state::Transition::Pop);
                }
            }
            geng::Event::Wheel { delta } => {
                self.cursor.scroll += delta as f32;
            }
            geng::Event::CursorMove { position } => {
                self.cursor.position = position.as_f32();
            }
            _ => (),
        }
    }

    fn fixed_update(&mut self, delta_time: f64) {
        let delta_time = Time::new(delta_time as _);
        self.player.update_tail(delta_time);
    }

    fn update(&mut self, delta_time: f64) {
        let delta_time = Time::new(delta_time as f32);
        self.time += delta_time;

        self.cursor.update(geng_utils::key::is_key_pressed(
            self.geng.window(),
            [MouseButton::Left],
        ));

        let cursor_world = self
            .camera
            .screen_to_world(self.framebuffer_size.as_f32(), self.cursor.position);

        self.player.collider.position = cursor_world.as_r32();
        self.player.reset_distance();
        self.player
            .update_distance(&self.exit_button.base_collider, false);
        self.player
            .update_distance(&self.play_button.base_collider, false);

        if !self.ui_focused {
            let hovering = self
                .exit_button
                .base_collider
                .contains(cursor_world.as_r32());
            self.exit_button.update(hovering, delta_time);
        }
        if self.exit_button.hover_time.is_max() {
            self.transition = Some(geng::state::Transition::Pop);
        }

        if !self.ui_focused {
            let hovering = self
                .play_button
                .base_collider
                .contains(cursor_world.as_r32());
            self.play_button.update(hovering, delta_time);
        }
        if self.play_button.hover_time.is_max() {
            self.play_level();
        }

        self.state.leaderboard.poll();

        self.update_active_group(delta_time);
        self.update_active_level(delta_time);
        self.update_options(delta_time);
        self.update_leaderboard(delta_time);
        self.update_config(delta_time);

        self.last_delta_time = delta_time;
    }
}
