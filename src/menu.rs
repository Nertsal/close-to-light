use crate::{
    assets::Assets,
    model::*,
    render::{Render, UtilRender},
};

use geng::prelude::*;
use geng_utils::conversions::Vec2RealConversions;

pub struct MainMenu {
    geng: Geng,
    assets: Rc<Assets>,
    config: Config,
    transition: Option<geng::state::Transition>,
    render: Render,
    util_render: UtilRender,
    framebuffer_size: vec2<usize>,
    /// Cursor position in screen space.
    cursor_pos: vec2<f64>,
    active_touch: Option<u64>,
    cursor_world_pos: vec2<Coord>,
    camera: Camera2d,
    time: Time,
    play_button: HoverButton,
    player: Collider,
}

impl MainMenu {
    pub fn new(geng: &Geng, assets: &Rc<Assets>, config: Config) -> Self {
        Self {
            geng: geng.clone(),
            assets: assets.clone(),
            transition: None,
            render: Render::new(geng, assets),
            util_render: UtilRender::new(geng, assets),
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
            play_button: HoverButton {
                collider: Collider {
                    position: vec2(0.0, 0.0).as_r32(),
                    rotation: Angle::ZERO,
                    shape: Shape::Circle { radius: r32(1.0) },
                },
                hover_time: Lifetime::new(Time::ZERO, Time::ZERO..=r32(1.5)),
            },
            player: Collider::new(
                vec2::ZERO,
                Shape::Circle {
                    radius: r32(config.player.radius),
                },
            ),
            config,
        }
    }

    fn play(&mut self) {
        let future = {
            let geng = self.geng.clone();
            let assets = self.assets.clone();
            let config = self.config.clone();

            async move {
                let manager = geng.asset_manager();
                let assets_path = run_dir().join("assets");

                let level: Level =
                    geng::asset::Load::load(manager, &assets_path.join("level.json"), &())
                        .await
                        .expect("failed to load level");

                crate::game::Game::new(&geng, &assets, config, level, Time::ZERO)
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
}

impl geng::State for MainMenu {
    fn transition(&mut self) -> Option<geng::state::Transition> {
        self.transition.take()
    }

    fn update(&mut self, delta_time: f64) {
        let delta_time = Time::new(delta_time as f32);
        self.time += delta_time;

        let pos = self.cursor_pos.as_f32();
        let game_pos = geng_utils::layout::fit_aabb(
            self.render.get_render_size().as_f32(),
            Aabb2::ZERO.extend_positive(self.framebuffer_size.as_f32()),
            vec2(0.5, 0.5),
        );
        let pos = pos - game_pos.bottom_left();
        self.cursor_world_pos = self.camera.screen_to_world(game_pos.size(), pos).as_r32();

        self.player.position = self.cursor_world_pos;

        self.play_button
            .hover_time
            .change(if self.player.check(&self.play_button.collider) {
                delta_time
            } else {
                -delta_time
            });
        if self.play_button.hover_time.is_max() {
            self.play_button.hover_time.set_ratio(Time::ZERO);
            self.play();
        }
    }

    fn handle_event(&mut self, event: geng::Event) {
        match event {
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
        ugli::clear(screen_buffer, Some(crate::render::COLOR_DARK), None, None);

        let mut framebuffer = self.render.start();

        self.util_render
            .draw_button(&self.play_button, "START", &self.camera, &mut framebuffer);

        self.util_render.draw_outline(
            &self.player,
            0.05,
            crate::render::COLOR_PLAYER,
            &self.camera,
            &mut framebuffer,
        );

        self.render.dither(self.time, R32::ZERO); // TODO

        let aabb = Aabb2::ZERO.extend_positive(screen_buffer.size().as_f32());
        geng_utils::texture::draw_texture_fit(
            self.render.get_buffer(),
            aabb,
            vec2(0.5, 0.5),
            &geng::PixelPerfectCamera,
            &self.geng,
            screen_buffer,
        );
    }
}
