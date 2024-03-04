mod ui;

use self::ui::CursorContext;
pub use self::ui::GameUI;

use crate::{assets::Assets, leaderboard::Leaderboard, model::*, render::game::GameRender};

// use std::thread::JoinHandle;

use geng::prelude::*;
use geng_utils::conversions::Vec2RealConversions;

pub struct Game {
    geng: Geng,
    transition: Option<geng::state::Transition>,
    render: GameRender,

    model: Model,
    debug_mode: bool,
    group_name: String,
    level_name: String,

    framebuffer_size: vec2<usize>,
    delta_time: Time,

    active_touch: Option<u64>,
    cursor: CursorContext,
    ui: GameUI,
    ui_focused: bool,
}

#[derive(Debug, Clone)]
pub struct PlayLevel {
    pub group_name: String,
    pub group_meta: GroupMeta,
    pub level_name: String,
    pub level_meta: LevelMeta,
    pub config: LevelConfig,
    pub level: Level,
    pub music: Music,
    pub start_time: Time,
}

impl PlayLevel {
    pub fn level_path(&self) -> std::path::PathBuf {
        run_dir()
            .join("assets")
            .join("groups")
            .join(&self.group_name)
            .join("levels")
            .join(&self.level_name)
            .join("level.json")
    }
}

impl Game {
    pub fn new(
        geng: &Geng,
        assets: &Rc<Assets>,
        options: Options,
        level: PlayLevel,
        leaderboard: Leaderboard,
        player_name: String,
    ) -> Self {
        Self::preloaded(
            geng,
            assets,
            Model::new(assets, options, level.clone(), leaderboard, player_name),
            level.group_name,
            level.level_name,
        )
    }

    fn preloaded(
        geng: &Geng,
        assets: &Rc<Assets>,
        model: Model,
        group_name: String,
        level_name: String,
    ) -> Self {
        Self {
            geng: geng.clone(),
            transition: None,
            render: GameRender::new(geng, assets),

            model,
            debug_mode: false,
            group_name,
            level_name,

            framebuffer_size: vec2(1, 1),
            delta_time: r32(0.1),

            active_touch: None,
            cursor: CursorContext::new(),
            ui: GameUI::new(assets),
            ui_focused: false,
        }
    }

    // fn load_leaderboard(&mut self, submit_score: bool) {
    //     if let Some(secrets) = &self.model.secrets {
    //         self.model.leaderboard = LeaderboardState::Pending;
    //         let player_name = self.model.player.name.clone();
    //         let submit_score = submit_score && !player_name.trim().is_empty();
    //         let score = submit_score.then_some(self.model.score.as_f32().ceil() as i32);
    //         let secrets = secrets.clone();

    //         let meta = crate::leaderboard::ScoreMeta::new(
    //             self.group_name.clone(),
    //             self.level_name.clone(),
    //             self.model.config.modifiers.clone(),
    //             self.model.config.health.clone(),
    //         );

    //         let future = async move {
    //             crate::leaderboard::Leaderboard::submit(player_name, score, &meta, secrets).await
    //         };
    //         self.leaderboard_task = Some(Task::new(future));
    //     }
    // }
}

impl geng::State for Game {
    fn transition(&mut self) -> Option<geng::state::Transition> {
        self.transition.take()
    }

    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        self.framebuffer_size = framebuffer.size();
        ugli::clear(framebuffer, Some(self.model.options.theme.dark), None, None);

        let fading = self.model.restart_button.is_fading() || self.model.exit_button.is_fading();

        self.render
            .draw_world(&self.model, self.debug_mode, framebuffer);

        if !fading {
            self.ui_focused = self.ui.layout(
                &mut self.model,
                Aabb2::ZERO.extend_positive(framebuffer.size().as_f32()),
                self.cursor,
                self.delta_time.as_f32(),
                &self.geng,
            );
            self.render
                .draw_ui(&self.ui, &self.model, self.debug_mode, framebuffer);
        }
        self.cursor.scroll = 0.0;
    }

    fn handle_event(&mut self, event: geng::Event) {
        match event {
            geng::Event::KeyPress { key } => match key {
                geng::Key::Escape => self.transition = Some(geng::state::Transition::Pop),
                geng::Key::F11 => self.geng.window().toggle_fullscreen(),
                geng::Key::F1 => self.debug_mode = !self.debug_mode,
                _ => {}
            },
            geng::Event::Wheel { delta } => {
                self.cursor.scroll += delta as f32;
            }
            geng::Event::CursorMove { position } => {
                self.cursor.position = position.as_f32();
            }
            geng::Event::TouchStart(touch) if self.active_touch.is_none() => {
                self.active_touch = Some(touch.id);
            }
            geng::Event::TouchMove(touch) if Some(touch.id) == self.active_touch => {
                self.cursor.position = touch.position.as_f32();
            }
            geng::Event::TouchEnd(touch) if Some(touch.id) == self.active_touch => {
                self.active_touch = None;
            }
            _ => {}
        }
    }

    fn update(&mut self, delta_time: f64) {
        let delta_time = Time::new(delta_time as _);
        self.delta_time = delta_time;
        self.model.leaderboard.poll();

        if let Some(transition) = self.model.transition.take() {
            match transition {
                Transition::LoadLeaderboard { submit_score } => {
                    let player_name = self.model.player.name.clone();
                    let do_submit_score = submit_score && !player_name.trim().is_empty();

                    let score = &self.model.score;
                    let raw_score = score.calculated.combined;
                    let submit_score = do_submit_score.then_some(raw_score);

                    let meta = crate::leaderboard::ScoreMeta::new(
                        self.group_name.clone(),
                        self.level_name.clone(),
                        self.model.level.config.modifiers.clone(),
                        self.model.level.config.health.clone(),
                        score.clone(),
                    );

                    if do_submit_score {
                        self.model
                            .leaderboard
                            .submit(player_name, submit_score, meta);
                    } else {
                        self.model.leaderboard.loaded.category = meta.category.clone();
                        // Save highscores on lost runs only locally
                        self.model.leaderboard.loaded.reload_local(Some(
                            &crate::leaderboard::SavedScore {
                                player: player_name,
                                score: raw_score,
                                meta,
                            },
                        ));
                        self.model.leaderboard.refetch();
                    }
                }
                Transition::Exit => self.transition = Some(geng::state::Transition::Pop),
            }
        }
    }

    fn fixed_update(&mut self, delta_time: f64) {
        let delta_time = Time::new(delta_time as _);

        let pos = self.cursor.position;
        let game_pos = geng_utils::layout::fit_aabb(
            self.render.get_render_size().as_f32(),
            Aabb2::ZERO.extend_positive(self.framebuffer_size.as_f32()),
            vec2(0.5, 0.5),
        );
        let pos = pos - game_pos.bottom_left();
        let target_pos = self
            .model
            .camera
            .screen_to_world(game_pos.size(), pos)
            .as_r32();
        self.model.update(target_pos, delta_time);
    }
}
