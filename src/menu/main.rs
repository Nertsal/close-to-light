use super::*;

use crate::{
    render::{THEME, mask::MaskedRender, post::PostRender, ui::UiRender},
    ui::{UiContext, layout::AreaOps, widget::*},
};

use ctl_local::Leaderboard;

pub struct MainMenu {
    context: Context,
    leaderboard: Leaderboard,
    options: GameOptions,
    transition: Option<geng::state::Transition>,

    dither_preview: DitherRender,
    masked: MaskedRender,
    dither: DitherRender,
    util_render: UtilRender,
    ui_render: UiRender,
    post_render: PostRender,

    framebuffer_size: vec2<usize>,
    /// Cursor position in screen space.
    cursor_pos: vec2<f64>,
    /// Cursor clicked last frame.
    clicked: bool,
    active_touch: Option<u64>,
    cursor_world_pos: vec2<Coord>,
    camera: Camera2d,

    time: FloatTime,
    play_button: HoverButton,
    player: Player,

    ui: MainUI,
    ui_context: UiContext,
}

struct MainUI {
    screen: WidgetState,
    version: TextWidget,
    exit: ButtonWidget,
    exit_queued: bool,
    options: OptionsButtonWidget,
    join_community: TextWidget,
    join_discord: IconButtonWidget,
    profile: ProfileWidget,
}

impl MainMenu {
    pub fn new(context: Context, client: Option<&Arc<ctl_client::Nertboard>>) -> Self {
        let leaderboard = Leaderboard::new(
            &context.geng,
            client,
            &context.local.fs,
            &context.achievements,
            context.get_options().account.auto_login,
        );

        Self {
            dither_preview: DitherRender::new_sized(
                &context.geng,
                &context.assets,
                crate::render::PREVIEW_RESOLUTION,
            ),
            masked: MaskedRender::new(&context.geng, &context.assets, vec2(1, 1)),
            dither: DitherRender::new(&context.geng, &context.assets),
            util_render: UtilRender::new(context.clone()),
            ui_render: UiRender::new(context.clone()),
            post_render: PostRender::new(context.clone()),

            framebuffer_size: vec2(1, 1),
            cursor_pos: vec2::ZERO,
            active_touch: None,
            cursor_world_pos: vec2::ZERO,
            clicked: false,
            camera: Camera2d {
                center: vec2::ZERO,
                rotation: Angle::ZERO,
                fov: Camera2dFov::Vertical(10.0),
            },

            time: FloatTime::ZERO,
            play_button: HoverButton::new(
                Collider {
                    position: vec2(0.0, 0.0).as_r32(),
                    rotation: Angle::ZERO,
                    shape: Shape::Circle { radius: r32(1.0) },
                },
                1.5,
            ),
            player: Player::new(
                Collider::new(vec2::ZERO, Shape::Circle { radius: r32(0.1) }),
                r32(0.0),
            ),

            ui: MainUI::new(context.clone()),
            ui_context: UiContext::new(context.clone()),

            options: GameOptions::new(context.clone(), leaderboard.clone()),
            transition: None,
            context,
            leaderboard,
        }
    }

    fn play(&mut self) {
        let context = self.context.clone();
        let state = LevelMenu::new(
            context,
            self.leaderboard.clone(),
            Some(self.play_button.clone()),
        );
        self.play_button.reset();
        self.transition = Some(geng::state::Transition::Push(Box::new(state)));
    }
}

impl geng::State for MainMenu {
    fn transition(&mut self) -> Option<geng::state::Transition> {
        let mut transition = self.transition.take();

        if transition.is_none() && self.ui.exit_queued {
            transition = Some(geng::state::Transition::Pop);
        }

        if transition.is_some() {
            self.cursor_pos = vec2(0.0, 0.0);
        }
        transition
    }

    fn update(&mut self, delta_time: f64) {
        let delta_time = FloatTime::new(delta_time as f32);
        self.context.update(delta_time);
        self.time += delta_time;

        self.context
            .geng
            .window()
            .set_cursor_type(geng::CursorType::None);

        self.ui_context.update(delta_time.as_f32());

        self.context.music.stop(); // TODO: menu music

        self.leaderboard.get_mut().poll();

        let pos = self.cursor_pos.as_f32();
        let game_pos = geng_utils::layout::fit_aabb(
            self.dither.get_render_size().as_f32(),
            Aabb2::ZERO.extend_positive(self.framebuffer_size.as_f32()),
            vec2(0.5, 0.5),
        );
        let pos = pos - game_pos.bottom_left();
        self.cursor_world_pos = self.camera.screen_to_world(game_pos.size(), pos).as_r32();

        self.options.preview.update(delta_time);

        // Update player cursor size
        self.options.player_size.update(delta_time.as_f32());
        self.player.collider.shape = Shape::circle(self.options.player_size.current);

        self.player.collider.position = self.cursor_world_pos;
        self.player.reset_distance();

        let hovering = self.player.collider.check(&self.play_button.base_collider);
        if hovering && self.clicked {
            self.play_button.clicked = true;
        }
        self.play_button.update(hovering, delta_time);
        self.player
            .update_distance_simple(&self.play_button.base_collider);
        if self.play_button.is_fading() {
            self.play();
        }

        self.clicked = false;
    }

    fn fixed_update(&mut self, delta_time: f64) {
        let delta_time = FloatTime::new(delta_time as _);
        self.player.update_tail(delta_time);
    }

    fn handle_event(&mut self, event: geng::Event) {
        match event {
            geng::Event::KeyPress {
                key: geng::Key::F11,
            } => self.context.geng.window().toggle_fullscreen(),
            geng::Event::Wheel { delta } => {
                self.ui_context.cursor.scroll += delta as f32;
            }
            geng::Event::CursorMove { position } => {
                self.cursor_pos = position;
                self.ui_context.cursor.cursor_move(position.as_f32());
            }
            geng::Event::MousePress {
                button: geng::MouseButton::Left,
            } => self.clicked = true,
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
        let options = self.context.get_options();
        let theme = options.theme;
        ugli::clear(screen_buffer, Some(theme.dark), None, None);

        self.masked.update_size(screen_buffer.size());

        let mut framebuffer = self.dither.start();

        let button = crate::render::smooth_button(&self.play_button, self.time + r32(0.5));
        self.util_render
            .draw_button(&button, "START", &THEME, &self.camera, &mut framebuffer);

        self.util_render.draw_text(
            "made in rust btw",
            vec2(0.0, -4.0),
            TextRenderOptions::new(0.5).color(THEME.dark),
            &self.camera,
            &mut framebuffer,
        );

        let fading = self.play_button.is_fading();
        if !fading
            && let Ok(pos) = self
                .camera
                .world_to_screen(framebuffer.size().as_f32(), vec2(0.0, 3.5))
        {
            self.ui_render.draw_texture(
                Aabb2::point(pos),
                &self.context.assets.sprites.title,
                THEME.light,
                1.0,
                &mut framebuffer,
            );
        }

        self.dither.finish(self.time, &theme);

        let buffer = &mut self.post_render.begin(screen_buffer.size(), theme.dark);

        let aabb = Aabb2::ZERO.extend_positive(buffer.size().as_f32());
        geng_utils::texture::DrawTexture::new(self.dither.get_buffer())
            .fit(aabb, vec2(0.5, 0.5))
            .draw(&geng::PixelPerfectCamera, &self.context.geng, buffer);

        if !fading {
            self.ui.layout(
                Aabb2::ZERO.extend_positive(buffer.size().as_f32()),
                &mut self.ui_context,
                &mut self.leaderboard,
                &mut self.options,
            );

            // UI
            let theme = self.context.get_options().theme;
            let ui = &self.ui;

            self.ui_render.draw_text(&ui.version, buffer);
            self.ui_render.draw_button(&ui.exit, theme, buffer);

            // Options
            if ui.options.open_time.is_above_min() {
                self.ui_render.draw_options(
                    &mut self.masked,
                    &mut self.dither_preview,
                    &ui.options,
                    &self.options,
                    buffer,
                );
            } else {
                // Options button
                self.ui_render.draw_icon(&ui.options.button, theme, buffer);
                self.ui_render.draw_outline(
                    ui.options.button.state.position,
                    self.ui_context.font_size * 0.1,
                    theme.light,
                    buffer,
                );
            }

            self.ui_render.draw_text(&ui.join_community, buffer);
            self.ui_render
                .draw_icon_button(&ui.join_discord, theme, buffer);
            self.ui_render.draw_profile(&ui.profile, buffer);
        }

        let mut dither_buffer = self.dither.start();
        self.util_render
            .draw_player(&self.player, &self.camera, &mut dither_buffer);
        self.dither.finish(self.time, &theme.transparent());
        geng_utils::texture::DrawTexture::new(self.dither.get_buffer())
            .fit_screen(vec2(0.5, 0.5), buffer)
            .draw(&geng::PixelPerfectCamera, &self.context.geng, buffer);

        self.post_render.post_process(
            crate::render::post::PostVfx {
                time: self.time,
                crt: options.graphics.crt.enabled,
                rgb_split: 0.0,
                colors: options.graphics.colors,
            },
            screen_buffer,
        );

        self.ui_context.frame_end();
    }
}

impl MainUI {
    pub fn new(context: Context) -> Self {
        Self {
            exit_queued: false,
            version: TextWidget::new(ctl_constants::GAME_VERSION.to_string())
                .aligned(vec2(0.5, 0.5)),
            screen: WidgetState::new(),
            exit: ButtonWidget::new("Exit"),
            options: OptionsButtonWidget::new(&context.assets, 0.25),
            join_community: TextWidget::new("Join our community!"),
            join_discord: IconButtonWidget::new_normal(context.assets.atlas.discord()),
            profile: ProfileWidget::new(&context.assets),
        }
    }

    pub fn layout(
        &mut self,
        screen: Aabb2<f32>,
        context: &mut UiContext,
        leaderboard: &mut Leaderboard,
        state: &mut GameOptions,
    ) {
        // Fix aspect
        let screen = screen.fit_aabb(vec2(16.0, 9.0), vec2::splat(0.5));

        let layout_size = screen.height() * 0.03;
        let font_size = screen.height() * 0.06;

        context.screen = screen;
        context.layout_size = layout_size;
        context.font_size = font_size;

        self.screen.update(screen, context);

        let exit = screen
            .align_aabb(vec2(2.2, 1.0) * context.font_size, vec2(0.0, 1.0))
            .translate(vec2(1.5, -0.5) * context.layout_size);
        let options = screen.extend_positive(-vec2(2.0, 0.5) * layout_size);

        self.exit.update(exit, &context.scale_font(0.8));
        if self.exit.text.state.mouse_left.clicked {
            self.exit_queued = true;
        }

        let version = Aabb2::point(exit.align_pos(vec2(0.5, 0.0)))
            .extend_symmetric(vec2(1.5 * font_size, 0.0))
            .extend_down(font_size);
        self.version.update(version, &context.scale_font(0.7));

        self.options.update(options, context, state);
        context.update_focus(self.options.options.state.hovered);

        let join = vec2(6.0, 3.0) * font_size;
        let mut join = screen
            .align_aabb(join, vec2(0.0, 0.0))
            .translate(vec2(1.0, 1.0) * layout_size);
        let text = join.cut_top(font_size * 1.5);
        self.join_community.update(text, context);
        self.join_discord.update(join, context);
        if self.join_discord.icon.state.mouse_left.clicked {
            let _ = webbrowser::open(crate::DISCORD_SERVER_URL);
        }

        let profile = vec2(6.0, 3.0) * font_size;
        let profile = screen
            .align_aabb(profile, vec2(1.0, 0.0))
            .translate(vec2(-1.0, 1.0) * layout_size);
        self.profile.update(profile, context, leaderboard);
    }
}
