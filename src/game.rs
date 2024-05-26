mod ui;

pub use self::ui::GameUI;
use self::ui::UiContext;

use crate::{leaderboard::Leaderboard, local::CachedMusic, prelude::*, render::game::GameRender};

pub struct Game {
    context: Context,
    transition: Option<geng::state::Transition>,
    render: GameRender,

    model: Model,
    debug_mode: bool,

    framebuffer_size: vec2<usize>,
    delta_time: Time,

    active_touch: Option<u64>,
    ui: GameUI,
    ui_focused: bool,
    ui_context: UiContext,
}

#[derive(Debug, Clone)]
pub struct PlayLevel {
    pub group_index: Index,
    pub level_index: usize,
    pub level: Rc<LevelFull>,
    pub config: LevelConfig,
    pub music: Rc<CachedMusic>,
    pub start_time: Time,
}

impl Game {
    pub fn new(
        context: Context,
        options: Options,
        level: PlayLevel,
        leaderboard: Leaderboard,
    ) -> Self {
        Self::preloaded(
            context.clone(),
            Model::new(context, options, level.clone(), leaderboard),
        )
    }

    fn preloaded(context: Context, model: Model) -> Self {
        Self {
            framebuffer_size: vec2(1, 1),
            delta_time: r32(0.1),

            active_touch: None,
            ui: GameUI::new(&context.assets),
            ui_focused: false,
            ui_context: UiContext::new(&context.geng, model.options.theme),

            model,
            debug_mode: false,

            transition: None,
            render: GameRender::new(&context.geng, &context.assets),
            context,
        }
    }
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
                &mut self.ui_context,
            );
            self.render
                .draw_ui(&self.ui, &self.model, self.debug_mode, framebuffer);
        }
        self.ui_context.frame_end();
    }

    fn handle_event(&mut self, event: geng::Event) {
        match event {
            geng::Event::KeyPress { key } => match key {
                geng::Key::Escape => self.transition = Some(geng::state::Transition::Pop),
                geng::Key::F11 => self.context.geng.window().toggle_fullscreen(),
                geng::Key::F1 => self.debug_mode = !self.debug_mode,
                _ => {}
            },
            geng::Event::Wheel { delta } => {
                self.ui_context.cursor.scroll += delta as f32;
            }
            geng::Event::CursorMove { position } => {
                self.ui_context.cursor.cursor_move(position.as_f32());
            }
            geng::Event::TouchStart(touch) if self.active_touch.is_none() => {
                self.active_touch = Some(touch.id);
            }
            geng::Event::TouchMove(touch) if Some(touch.id) == self.active_touch => {
                self.ui_context.cursor.cursor_move(touch.position.as_f32());
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
        if let Some(player) = self.model.leaderboard.loaded.player {
            self.model.player.info.id = player;
        }

        self.ui_context
            .update(self.context.geng.window(), delta_time.as_f32());

        if let Some(transition) = self.model.transition.take() {
            match transition {
                Transition::LoadLeaderboard { submit_score } => {
                    let player_name = self.model.player.info.name.clone();
                    let submit_score = submit_score && !player_name.trim().is_empty();
                    let raw_score = self.model.score.calculated.combined;
                    let score = submit_score.then_some(raw_score);

                    let meta = crate::leaderboard::ScoreMeta::new(
                        self.model.level.config.modifiers.clone(),
                        self.model.level.config.health.clone(),
                    );

                    if submit_score {
                        self.model
                            .leaderboard
                            .submit(score, self.model.level.level.meta.id, meta);
                    } else {
                        self.model.leaderboard.loaded.meta = meta.clone();
                        // Save highscores on lost runs only locally
                        self.model.leaderboard.loaded.reload_local(Some(
                            &crate::leaderboard::SavedScore {
                                level: self.model.level.level.meta.id,
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

        let pos = self.ui_context.cursor.position;
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
