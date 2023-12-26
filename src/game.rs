use crate::{
    assets::Assets, leaderboard::Leaderboard, model::*, render::game::GameRender,
    LeaderboardSecrets,
};

use std::thread::JoinHandle;

use geng::prelude::*;
use geng_utils::conversions::Vec2RealConversions;

pub struct Game {
    geng: Geng,
    leaderboard_handle: Option<JoinHandle<std::io::Result<Leaderboard>>>,
    transition: Option<geng::state::Transition>,
    render: GameRender,
    model: Model,
    framebuffer_size: vec2<usize>,
    /// Cursor position in screen space.
    cursor_pos: vec2<f64>,
    active_touch: Option<u64>,
}

impl Game {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        geng: &Geng,
        assets: &Rc<Assets>,
        config: LevelConfig,
        level: Level,
        level_music: Music,
        leaderboard: Option<LeaderboardSecrets>,
        player_name: String,
        start_time: Time,
    ) -> Self {
        Self::preloaded(
            geng,
            assets,
            Model::new(
                assets,
                config,
                level,
                level_music,
                leaderboard,
                player_name,
                start_time,
            ),
        )
    }

    fn preloaded(geng: &Geng, assets: &Rc<Assets>, model: Model) -> Self {
        Self {
            geng: geng.clone(),
            leaderboard_handle: None,
            transition: None,
            render: GameRender::new(geng, assets),
            model,
            framebuffer_size: vec2(1, 1),
            cursor_pos: vec2::ZERO,
            active_touch: None,
        }
    }

    fn load_leaderboard(&mut self, submit_score: bool) {
        if let Some(secrets) = &self.model.secrets {
            self.model.leaderboard = LeaderboardState::Pending;
            let player_name = self.model.player.name.clone();
            let submit_score = submit_score && !player_name.trim().is_empty();
            let score = submit_score.then_some(self.model.score.as_f32() as i32);
            let secrets = secrets.clone();

            let handle = std::thread::spawn(move || {
                let runtime = tokio::runtime::Runtime::new()?;
                let leaderboard = runtime.block_on(crate::leaderboard::Leaderboard::submit(
                    player_name,
                    score,
                    secrets,
                ));
                Ok(leaderboard)
            });
            self.leaderboard_handle = Some(handle);
        }
    }
}

impl geng::State for Game {
    fn transition(&mut self) -> Option<geng::state::Transition> {
        self.transition.take()
    }

    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        self.framebuffer_size = framebuffer.size();
        ugli::clear(framebuffer, Some(self.model.config.theme.dark), None, None);
        self.render.draw_world(&self.model, framebuffer);
        self.render.draw_ui(&self.model, framebuffer);
    }

    fn handle_event(&mut self, event: geng::Event) {
        match event {
            geng::Event::KeyPress { key } => match key {
                geng::Key::Escape => self.transition = Some(geng::state::Transition::Pop),
                geng::Key::F11 => self.geng.window().toggle_fullscreen(),
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

    fn update(&mut self, delta_time: f64) {
        let _delta_time = Time::new(delta_time as _);

        if let Some(handle) = self.leaderboard_handle.take() {
            // Poll leaderboard
            if handle.is_finished() {
                match handle.join() {
                    Ok(Ok(leaderboard)) => {
                        log::info!("Loaded leaderboard");
                        self.model.leaderboard = LeaderboardState::Ready(leaderboard);
                    }
                    Ok(Err(err)) => {
                        log::error!("Failed to load leaderboard: {}", err);
                        self.model.leaderboard = LeaderboardState::Failed;
                    }
                    Err(_) => {
                        log::error!("Failed to join leaderboard handle");
                        self.model.leaderboard = LeaderboardState::Failed;
                    }
                }
            } else {
                self.leaderboard_handle = Some(handle);
            }
        }

        if let Some(transition) = self.model.transition.take() {
            match transition {
                Transition::LoadLeaderboard { submit_score } => {
                    self.load_leaderboard(submit_score);
                }
            }
        }
    }

    fn fixed_update(&mut self, delta_time: f64) {
        let delta_time = Time::new(delta_time as _);

        let pos = self.cursor_pos.as_f32();
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
