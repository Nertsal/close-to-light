mod ui;

pub use self::ui::*;

use super::*;

use crate::{
    leaderboard::{Leaderboard, LeaderboardStatus},
    local::{CachedLevel, CachedMusic},
    render::{mask::MaskedRender, menu::MenuRender},
    ui::{widget::Widget, ShowTime, UiContext, WidgetRequest},
};

pub struct LevelMenu {
    context: Context,
    transition: Option<geng::state::Transition>,

    render: MenuRender,
    util: UtilRender,
    dither: DitherRender,
    masked: MaskedRender,

    framebuffer_size: vec2<usize>,
    last_delta_time: Time,
    time: Time,

    ui: MenuUI,
    ui_focused: bool,
    ui_context: UiContext,

    camera: Camera2d,
    state: MenuState,
    exit_button: HoverButton,
    play_button: HoverButton,
}

pub struct MenuState {
    pub context: Context,
    pub leaderboard: Leaderboard,
    pub player: Player,
    pub options: Options,
    pub config: LevelConfig,

    /// Currently showing music.
    pub selected_music: Option<ShowTime<Id>>,
    /// Currently showing group.
    pub selected_group: Option<ShowTime<Index>>,
    /// Currently showing level of the active group.
    pub selected_level: Option<ShowTime<usize>>,

    /// Switch to the music after current one finishes its animation.
    pub switch_music: Option<Id>,
    /// Switch to the group after current one finishes its animation.
    pub switch_group: Option<Index>,
    /// Switch to the level of the active group after current one finishes its animation.
    pub switch_level: Option<usize>,

    /// Whether to open a (group, level) in the editor.
    pub edit_level: Option<(Index, usize)>,
}

pub struct GroupEntry {
    pub meta: GroupInfo,
    pub logo: Option<ugli::Texture>,
}

impl Debug for GroupEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GroupEntry")
            .field("meta", &self.meta)
            .field("logo", &self.logo.as_ref().map(|_| "<logo>"))
            .finish()
    }
}

impl MenuState {
    fn select_music(&mut self, music: Id) {
        self.switch_music = Some(music);
        if self
            .selected_music
            .as_ref()
            .map_or(true, |selected| selected.data != music)
        {
            self.switch_group = None;
            self.switch_level = None;
        }
    }

    fn select_group(&mut self, group: Index) {
        self.switch_group = Some(group);
        if self
            .selected_group
            .as_ref()
            .map_or(true, |selected| selected.data != group)
        {
            self.switch_level = None;
        }
    }

    fn select_level(&mut self, level: usize) {
        self.switch_level = Some(level);
    }

    fn edit_level(&mut self, group: Index, level: usize) {
        self.edit_level = Some((group, level));
    }

    fn new_group(&mut self) {
        self.switch_group = None; // Deselect group
        let local = &self.context.local;
        // TODO: maybe ui to configure early
        local.new_group(0);
    }

    fn new_level(&mut self, group: Index) {
        let local = &self.context.local;
        let meta = LevelInfo {
            id: 0,
            name: "New Difficulty".into(),
            hash: String::new(),
            authors: Vec::new(),
        };
        local.new_level(group, meta);
    }
}

impl LevelMenu {
    pub fn new(
        context: Context,
        client: Option<&Arc<ctl_client::Nertboard>>,
        options: Options,
    ) -> Self {
        let mut player = Player::new(
            Collider::new(vec2::ZERO, Shape::Circle { radius: r32(1.0) }),
            r32(0.0),
        );
        player.info.name = preferences::load::<String>(crate::PLAYER_NAME_STORAGE)
            .unwrap_or_default()
            .into();

        let leaderboard = Leaderboard::new(&context.geng, client);

        Self {
            render: MenuRender::new(&context.geng, &context.assets),
            util: UtilRender::new(&context.geng, &context.assets),
            dither: DitherRender::new(&context.geng, &context.assets),
            masked: MaskedRender::new(&context.geng, &context.assets, vec2(1, 1)),

            framebuffer_size: vec2(1, 1),
            last_delta_time: Time::ONE,
            time: Time::ZERO,

            ui: MenuUI::new(context.clone()),
            ui_focused: false,
            ui_context: UiContext::new(&context.geng, options.theme),

            camera: Camera2d {
                center: vec2::ZERO,
                rotation: Angle::ZERO,
                fov: 10.0,
            },
            state: MenuState {
                context: context.clone(),
                leaderboard,
                player,
                options,
                config: LevelConfig::default(),

                selected_music: None,
                selected_group: None,
                selected_level: None,

                switch_music: None,
                switch_group: None,
                switch_level: None,

                edit_level: None,
            },
            exit_button: HoverButton::new(
                Collider::new(vec2(-7.6, 3.7).as_r32(), Shape::Circle { radius: r32(0.6) }),
                3.0,
            ),
            play_button: HoverButton::new(
                Collider {
                    position: vec2(4.9, -0.5).as_r32(),
                    rotation: Angle::ZERO,
                    shape: Shape::Circle { radius: r32(0.8) },
                },
                1.5,
            ),

            context,
            transition: None,
        }
    }

    fn get_active_level(&self) -> Option<(Rc<CachedMusic>, Rc<CachedLevel>)> {
        let local = self.context.local.inner.borrow();

        let group = self.state.selected_group.as_ref()?;
        let group = local.groups.get(group.data)?;

        let music = group.music.as_ref()?;

        let level = self.state.selected_level.as_ref()?;
        let level = group.levels.get(level.data)?;

        Some((Rc::clone(music), Rc::clone(level)))
    }

    fn play_level(&mut self) {
        let Some((music, level)) = self.get_active_level() else {
            log::error!("Trying to play a level, but there is no active level");
            return;
        };

        self.context.music.stop();
        self.ui_context.cursor.reset();
        self.play_button.hover_time.set(Time::ZERO);

        let future = {
            let context = self.context.clone();
            let leaderboard = self.state.leaderboard.clone();
            let options = self.state.options.clone();
            let config = self.state.config.clone();
            let player_name: String = preferences::load(PLAYER_NAME_STORAGE).unwrap_or_default();

            async move {
                let level = crate::game::PlayLevel {
                    level: level.clone(),
                    config,
                    music,
                    start_time: Time::ZERO,
                };
                crate::game::Game::new(context, options, level, leaderboard, player_name)
            }
        };
        self.transition = Some(geng::state::Transition::Push(Box::new(
            geng::LoadingScreen::new(
                &self.context.geng,
                geng::EmptyLoadingScreen::new(&self.context.geng),
                future,
            ),
        )));
        // Queue leaderboard fetch when coming back
        self.state.leaderboard.status = LeaderboardStatus::None;
    }

    fn update_active_music(&mut self, delta_time: Time) {
        let delta_time = delta_time.as_f32();
        if let Some(current_music) = &mut self.state.selected_music {
            if let Some(switch_music) = self.state.switch_music {
                if current_music.data != switch_music {
                    current_music.time.change(-delta_time);
                    current_music.going_up = false;

                    if current_music.time.is_min() {
                        // Switch
                        current_music.data = switch_music;
                    }
                } else {
                    current_music.time.change(delta_time);
                    current_music.going_up = true;
                }
            } else {
                current_music.time.change(-delta_time);
                current_music.going_up = false;

                if current_music.time.is_min() {
                    // Remove
                    self.state.selected_music = None;
                    self.ui.level_select.tab_groups.hide();
                    self.ui.level_select.tab_levels.hide();
                    self.ui.level_select.select_tab(LevelSelectTab::Music);
                }
            }
        } else if let Some(music) = self.state.switch_music {
            self.state.selected_music = Some(ShowTime {
                data: music,
                time: Bounded::new_zero(0.25),
                going_up: true,
            });
        }
    }

    fn update_active_group(&mut self, delta_time: Time) {
        let delta_time = delta_time.as_f32();
        if let Some(current_group) = &mut self.state.selected_group {
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
                    self.state.selected_group = None;
                    self.state.selected_level = None;
                    self.ui.level_select.tab_levels.hide();
                    self.ui.level_select.select_tab(LevelSelectTab::Group);
                }
            }
        } else if let Some(group) = self.state.switch_group {
            self.state.selected_group = Some(ShowTime {
                data: group,
                time: Bounded::new_zero(0.25),
                going_up: true,
            });
        }
    }

    fn update_active_level(&mut self, delta_time: Time) {
        let delta_time = delta_time.as_f32();
        if let Some(current_level) = &mut self.state.selected_level {
            if let Some(switch_level) = self.state.switch_level {
                if current_level.data != switch_level {
                    // self.state.show_leaderboard.going_up = false; // Hide leaderboard
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
                // self.state.level_up = false;
                current_level.time.change(-delta_time);
                current_level.going_up = false;

                if current_level.time.is_min() {
                    // Remove
                    self.state.selected_level = None;
                }
            }
        } else if let Some(level) = self.state.switch_level {
            self.state.selected_level = Some(ShowTime {
                data: level,
                time: Bounded::new_zero(0.25),
                going_up: true,
            });
        }
    }

    fn fetch_leaderboard(&mut self) {
        let mods = self.state.config.modifiers.clone();
        let health = self.state.config.health.clone();

        let meta = crate::leaderboard::ScoreMeta::new(mods, health);
        self.state.leaderboard.change_meta(meta);
    }

    fn update_leaderboard(&mut self) {
        if let Some(req) = self.ui.leaderboard.window.last_request.take() {
            match req {
                WidgetRequest::Open | WidgetRequest::Reload => {
                    self.fetch_leaderboard();
                }
                WidgetRequest::Close => {}
            }
        }
    }
}

impl geng::State for LevelMenu {
    fn transition(&mut self) -> Option<geng::state::Transition> {
        let transition = self.transition.take();
        if transition.is_some() {
            self.context.music.stop();
        }
        transition
    }

    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        self.framebuffer_size = framebuffer.size();
        ugli::clear(framebuffer, Some(self.state.options.theme.dark), None, None);
        self.masked.update_size(framebuffer.size());

        self.dither.set_noise(1.0);
        let mut dither_buffer = self.dither.start();

        let fading = self.exit_button.is_fading() || self.play_button.is_fading();

        // if !fading || self.exit_button.is_fading() {
        //     let button = crate::render::smooth_button(&self.exit_button, self.time);
        //     self.util.draw_button(
        //         &button,
        //         "EXIT",
        //         &crate::render::THEME,
        //         &self.camera,
        //         &mut dither_buffer,
        //     );
        // }

        if !fading || self.play_button.is_fading() {
            let play_time = r32(self
                .state
                .selected_level
                .as_ref()
                .map_or(0.0, |show| show.time.get_ratio()));
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
            .draw_player(&self.state.player, &self.camera, &mut dither_buffer);

        self.dither.finish(self.time, &self.state.options.theme);

        geng_utils::texture::DrawTexture::new(self.dither.get_buffer())
            .fit_screen(vec2(0.5, 0.5), framebuffer)
            .draw(&geng::PixelPerfectCamera, &self.context.geng, framebuffer);

        if !fading {
            let mut masked = self.masked.start();

            self.ui_focused = self.ui.layout(
                &mut self.state,
                Aabb2::ZERO.extend_positive(framebuffer.size().as_f32()),
                &mut self.ui_context,
            );
            self.render
                .draw_ui(&self.ui, &self.state, &mut masked.color);

            masked.mask_quad(self.ui.screen.position);

            let pixelated = false;

            if pixelated {
                self.dither.set_noise(0.0);
                let mut dither = self.dither.start();

                self.masked.draw(
                    ugli::DrawParameters {
                        blend_mode: Some(ugli::BlendMode::straight_alpha()),
                        ..default()
                    },
                    &mut dither,
                );

                // self.dither.finish(
                //     self.time,
                //     &Theme {
                //         dark: Color::TRANSPARENT_BLACK,
                //         ..self.state.options.theme
                //     },
                // );

                geng_utils::texture::DrawTexture::new(self.dither.get_buffer())
                    .fit_screen(vec2(0.5, 0.5), framebuffer)
                    .draw(&geng::PixelPerfectCamera, &self.context.geng, framebuffer);
            } else {
                self.masked.draw(
                    ugli::DrawParameters {
                        blend_mode: Some(ugli::BlendMode::straight_alpha()),
                        ..default()
                    },
                    framebuffer,
                );
            };

            if pixelated {}
        }
        self.ui_context.frame_end();
    }

    fn handle_event(&mut self, event: geng::Event) {
        match event {
            geng::Event::EditText(text) => {
                self.ui_context.text_edit.text = text;
            }
            geng::Event::KeyPress {
                key: geng::Key::Escape,
            } => {
                if let Some(sync) = &mut self.ui.sync {
                    sync.window.request = Some(WidgetRequest::Close);
                } else if self.ui.explore.window.show.time.is_max() {
                    self.ui.explore.window.request = Some(WidgetRequest::Close);
                // } else if self.ui.panels.options.window.show.time.is_max() {
                //     self.ui.panels.options.window.request = Some(WidgetRequest::Close);
                } else if self.ui.leaderboard.window.show.time.is_max() {
                    self.ui.leaderboard.window.request = Some(WidgetRequest::Close);
                } else if self.ui.level_config.window.show.time.is_max() {
                    self.ui.level_config.window.request = Some(WidgetRequest::Close);
                } else if self.state.switch_level.take().is_some()
                    || self.state.switch_group.take().is_some()
                    || self.state.switch_music.take().is_some()
                {
                } else {
                    // Go to main menu
                    self.transition = Some(geng::state::Transition::Pop);
                }
            }
            geng::Event::Wheel { delta } => {
                self.ui_context.cursor.scroll += delta as f32;
            }
            geng::Event::CursorMove { position } => {
                self.ui_context.cursor.cursor_move(position.as_f32());
            }
            _ => (),
        }
    }

    fn fixed_update(&mut self, delta_time: f64) {
        let delta_time = Time::new(delta_time as _);
        self.state.player.update_tail(delta_time);
    }

    fn update(&mut self, delta_time: f64) {
        let delta_time = Time::new(delta_time as f32);
        self.time += delta_time;

        self.ui_context
            .update(self.context.geng.window(), delta_time.as_f32());
        self.ui_context.theme = self.state.options.theme;

        if self.ui.explore.state.visible {
            let t = self.ui.explore.window.show.time.get_ratio();
            self.context
                .music
                .set_volume(self.state.options.volume.music() * t);
        } else {
            // Music volume
            let t = (1.0 - self.play_button.hover_time.get_ratio().as_f32())
                .min(show_ratio(&self.state.selected_music).unwrap_or(0.0));
            self.context
                .music
                .set_volume(self.state.options.volume.music() * t);

            // Playing music
            if let Some(active) = self
                .state
                .selected_music
                .as_ref()
                .and_then(|music| self.state.context.local.get_music(music.data))
            {
                self.context.music.switch(&active); // TODO: rng start
            } else {
                self.context.music.stop();
            }
        }

        let game_pos = geng_utils::layout::fit_aabb(
            self.dither.get_render_size().as_f32(),
            Aabb2::ZERO.extend_positive(self.framebuffer_size.as_f32()),
            vec2(0.5, 0.5),
        );
        let pos = self.ui_context.cursor.position - game_pos.bottom_left();
        let cursor_world = self.camera.screen_to_world(game_pos.size(), pos);

        self.state.player.collider.position = cursor_world.as_r32();
        self.state.player.reset_distance();
        self.state
            .player
            .update_distance_simple(&self.exit_button.base_collider);
        if self.state.selected_level.is_some() {
            self.state
                .player
                .update_distance_simple(&self.play_button.base_collider);
        }

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

        if !self.ui_focused && self.state.selected_level.is_some() {
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
        if let Some(player) = self.state.leaderboard.loaded.player {
            self.state.player.info.id = player;
        }

        self.update_active_music(delta_time);
        self.update_active_group(delta_time);
        self.update_active_level(delta_time);
        self.update_leaderboard();

        self.context.local.poll();

        if let Some((music, level)) = self.state.edit_level.take().and_then(|(group, level)| {
            // TODO: warn if smth wrong
            let local = self.state.context.local.inner.borrow();
            local.groups.get(group).and_then(|group| {
                group.music.as_ref().and_then(|music| {
                    group
                        .levels
                        .get(level)
                        .map(|level| (music.clone(), level.clone()))
                })
            })
        }) {
            let context = self.context.clone();
            let manager = self.context.geng.asset_manager().clone();
            let assets_path = run_dir().join("assets");
            let options = self.state.options.clone();
            let level = crate::game::PlayLevel {
                music,
                level,
                config: self.state.config.clone(),
                start_time: Time::ZERO,
            };
            let future = async move {
                let config: crate::editor::EditorConfig =
                    geng::asset::Load::load(&manager, &assets_path.join("editor.ron"), &())
                        .await
                        .expect("failed to load editor config");

                crate::editor::EditorState::new(context, config, options, level)
            };
            let state = geng::LoadingScreen::new(
                &self.context.geng,
                geng::EmptyLoadingScreen::new(&self.context.geng),
                future,
            );

            self.transition = Some(geng::state::Transition::Push(Box::new(state)));
        }

        self.last_delta_time = delta_time;
    }
}

fn show_ratio<T>(show: &Option<ShowTime<T>>) -> Option<f32> {
    show.as_ref().map(|show| show.time.get_ratio())
}
