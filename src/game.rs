mod ui;

pub use self::ui::GameUI;
use self::ui::UiContext;

use crate::{
    prelude::*,
    render::{game::GameRender, post::PostRender},
};

use ctl_local::Leaderboard;

pub struct Game {
    context: Context,
    transition: Option<geng::state::Transition>,
    render: GameRender,
    post: PostRender,

    model: Model,
    debug_mode: bool,

    framebuffer_size: vec2<usize>,
    delta_time: FloatTime,

    active_touch: Option<u64>,
    ui: GameUI,
    ui_focused: bool,
    ui_context: UiContext,
}

impl Game {
    pub fn new(context: Context, level: PlayLevel, leaderboard: Leaderboard) -> Self {
        if let Some(music) = &level.group.music {
            context.set_status(format!(
                "Playing {} - {}",
                music.meta.name, level.level.meta.name
            ));
        }

        if level.group.music.is_none() {
            log::warn!(
                "Starting level {:?} but no music got loaded.",
                level.level.meta.name
            );
        }

        Self::preloaded(
            context.clone(),
            Model::new(context, level.clone(), leaderboard),
        )
    }

    fn preloaded(context: Context, model: Model) -> Self {
        Self {
            framebuffer_size: vec2(1, 1),
            delta_time: r32(0.1),

            active_touch: None,
            ui: GameUI::new(&context.assets),
            ui_focused: false,
            ui_context: UiContext::new(context.clone()),

            model,
            debug_mode: false,

            transition: None,
            render: GameRender::new(context.clone()),
            post: PostRender::new(context.clone()),
            context,
        }
    }
}

impl geng::State for Game {
    fn transition(&mut self) -> Option<geng::state::Transition> {
        let trans = self.transition.take();

        if trans.is_some() {
            self.context.pop_status();
        }

        trans
    }

    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        self.framebuffer_size = framebuffer.size();
        let options = self.context.get_options();
        let theme = options
            .theme
            .swap(self.model.vfx.palette_swap.current.as_f32());
        ugli::clear(framebuffer, Some(theme.dark), None, None);

        let buffer = &mut self.post.begin(framebuffer.size(), theme.dark);

        let fading = self.model.restart_button.is_fading() || self.model.exit_button.is_fading();

        self.render.draw_world(&self.model, self.debug_mode, buffer);

        if !fading {
            self.ui_focused = self.ui.layout(
                &mut self.model,
                Aabb2::ZERO.extend_positive(buffer.size().as_f32()),
                &mut self.ui_context,
            );
            self.render
                .draw_ui(&self.ui, &self.model, self.debug_mode, buffer);
        }
        self.ui_context.frame_end();

        if !self.model.level.config.modifiers.clean_auto {
            let mut dither_buffer = self.render.dither.start();
            self.render.util.draw_player(
                &self.model.player,
                &self.model.camera,
                &mut dither_buffer,
            );
            self.render
                .dither
                .finish(self.model.real_time, &theme.transparent());
            geng_utils::texture::DrawTexture::new(self.render.dither.get_buffer())
                .fit_screen(vec2(0.5, 0.5), buffer)
                .draw(&geng::PixelPerfectCamera, &self.context.geng, buffer);
        }

        self.post.post_process(
            crate::render::post::PostVfx {
                time: self.model.real_time,
                crt: options.graphics.crt.enabled,
                rgb_split: self.model.vfx.rgb_split.value.current.as_f32(),
                colors: options.graphics.colors,
            },
            framebuffer,
        );
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
            geng::Event::MousePress {
                button: geng::MouseButton::Left,
            } => self.model.cursor_clicked = true,
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
        let delta_time = FloatTime::new(delta_time as _);
        self.context.update(delta_time);
        self.delta_time = delta_time;

        self.context
            .geng
            .window()
            .set_cursor_type(geng::CursorType::None);

        self.model.leaderboard.get_mut().poll();
        if let Some(player) = self.model.leaderboard.get_loaded().player {
            self.model.player.info.id = player;
        }

        self.ui_context.update(delta_time.as_f32());

        if let Some(transition) = self.model.transition.take() {
            match transition {
                Transition::LoadLeaderboard { submit_score } => {
                    let score = &self.model.score;
                    let raw_score = score.calculated.combined;

                    let meta = ctl_local::ScoreMeta::new(
                        self.model.level.config.modifiers.clone(),
                        self.model.level.config.health.clone(),
                        score.clone(),
                        self.model.current_completion(),
                    );

                    self.model.leaderboard.get_mut().reload_submit(
                        Some(raw_score),
                        submit_score,
                        self.model.level.level.meta.clone(),
                        meta,
                    );
                }
                Transition::Exit => self.transition = Some(geng::state::Transition::Pop),
            }
        }
    }

    fn fixed_update(&mut self, delta_time: f64) {
        let delta_time = FloatTime::new(delta_time as _);

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
        self.model.cursor_clicked = false;
    }
}
