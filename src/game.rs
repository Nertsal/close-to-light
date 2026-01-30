mod ui;

pub use self::ui::GameUI;
use self::ui::UiContext;

use crate::{
    prelude::*,
    render::{game::GameRender, post::PostRender},
};

use ctl_local::Leaderboard;

/// Max world distance within which the cursor is considered aligned with the paused state.
const CURSOR_ALIGNMENT_RANGE: f32 = 0.1;

pub struct Game {
    context: Context,
    transition: Option<geng::state::Transition>,
    render: GameRender,
    post: PostRender,

    model: Model,
    debug_mode: bool,

    pause_player: Player,
    /// If currently paused, gets set to `true` when the cursor is aligned with the paused state.
    paused_waiting_cursor_alignment: bool,
    was_paused: bool,

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

            pause_player: {
                let mut player = model.player.clone();
                player.collider = Collider::new(player.collider.position, Shape::circle(r32(0.1)));
                player
            },
            paused_waiting_cursor_alignment: false,
            was_paused: false,

            model,
            debug_mode: false,

            transition: None,
            render: GameRender::new(context.clone()),
            post: PostRender::new(&context),
            context,
        }
    }

    fn enable_touch_mod(&mut self) {
        self.model.level.config.modifiers.touch = true;
    }

    fn is_paused(&self) -> bool {
        self.ui.pause.window.show.time.is_above_min() || self.paused_waiting_cursor_alignment
    }

    fn toggle_pause(&mut self) {
        if self.is_paused() {
            self.unpause();
        } else {
            self.pause();
        }
    }

    fn pause(&mut self) {
        self.ui.pause.window.request = Some(ctl_ui::WidgetRequest::Open);
        self.paused_waiting_cursor_alignment = true;
        self.context.music.stop();
    }

    fn unpause(&mut self) {
        self.ui.pause.window.request = Some(ctl_ui::WidgetRequest::Close);
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
        let is_paused = self.is_paused();

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

        if is_paused {
            let ui = &self.ui.pause;

            if ui.window.show.time.is_above_min() {
                self.render.ui.draw_quad(
                    Aabb2::ZERO.extend_positive(framebuffer.size().as_f32()),
                    crate::util::with_alpha(Rgba::BLACK, 0.25),
                    buffer,
                );
            }

            // Pause menu
            let width = self.ui_context.font_size * 0.2;
            self.render.ui.fill_quad_width(
                ui.state.position,
                width,
                crate::util::with_alpha(theme.dark, 0.9),
                buffer,
            );
            self.render.ui.draw_text(&ui.title, buffer);
            self.render.ui.draw_button(&ui.resume, theme, buffer);
            self.render.ui.draw_button(&ui.retry, theme, buffer);
            self.render.ui.draw_button(&ui.quit, theme, buffer);
            self.render
                .ui
                .draw_outline(ui.state.position, width, theme.light, buffer);

            // Pause cursor
            let mut dither_buffer = self.render.dither.start();
            self.render.util.draw_player(
                &self.pause_player,
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
            &options,
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
                geng::Key::Escape => {
                    self.toggle_pause();
                }
                geng::Key::F11 => self.context.geng.window().toggle_fullscreen(),
                #[cfg(debug_assertions)]
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
                self.enable_touch_mod();
                self.active_touch = Some(touch.id);
            }
            geng::Event::TouchMove(touch) if Some(touch.id) == self.active_touch => {
                self.enable_touch_mod();
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

        if self.is_paused() {
            self.paused_waiting_cursor_alignment = (self.model.player.collider.position
                - self.pause_player.collider.position)
                .len()
                .as_f32()
                > CURSOR_ALIGNMENT_RANGE;
        }

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
                        self.model
                            .level
                            .group
                            .music
                            .as_ref()
                            .map(|music| music.meta.clone())
                            .unwrap_or_default(),
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
        let is_paused = self.is_paused();

        if self.was_paused
            && !is_paused
            && let Some(music) = &self.model.level.group.music
        {
            // Resume from pause
            self.context
                .music
                .play_from_time(music, self.model.play_time_ms, false);
        }

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
        self.model.update(target_pos, delta_time, is_paused);
        self.model.cursor_clicked = false;

        self.pause_player.collider.position = target_pos;
        self.pause_player.update_tail(delta_time);

        self.was_paused = is_paused;
    }
}
