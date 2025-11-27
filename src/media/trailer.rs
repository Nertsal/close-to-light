use crate::{
    prelude::*,
    render::{
        THEME,
        dither::DitherRender,
        game::GameRender,
        post::{PostRender, PostVfx},
        ui::UiRender,
        util::UtilRender,
    },
};

use ctl_render_core::TextRenderOptions;
use geng_utils::{conversions::AngleRealConversions, gif::GifFrame};

pub struct TrailerState {
    context: Context,

    game_render: GameRender,
    util_render: UtilRender,
    ui_render: UiRender,
    dither: DitherRender,
    post: PostRender,

    theme: Theme,
    time: FloatTime,
    camera: Camera2d,
    load_texts: Vec<&'static str>,
}

impl TrailerState {
    pub fn new(context: Context) -> Self {
        Self {
            game_render: GameRender::new(context.clone()),
            util_render: UtilRender::new(context.clone()),
            ui_render: UiRender::new(context.clone()),
            dither: DitherRender::new(&context.geng, &context.assets),
            post: PostRender::new(context.clone()),

            theme: Theme::linksider(),
            time: FloatTime::ZERO,
            camera: Camera2d {
                center: vec2::ZERO,
                rotation: Angle::ZERO,
                fov: Camera2dFov::Vertical(10.0),
            },
            load_texts: vec![
                "Loading assets...",
                "Turning the lights on...",
                "Initializing evil... >:3",
                "Why is this taking so long?",
            ],

            context,
        }
    }
}

impl geng::State for TrailerState {
    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        let theme = self.theme;
        ugli::clear(framebuffer, Some(theme.dark), None, None);

        // self.game_render.draw_world(&self.model, false, framebuffer);

        let mut dither_buffer = self.dither.start();
        let loading_lights = [
            Collider {
                position: vec2(-8.75, 4.5).as_r32(),
                rotation: Angle::from_degrees(15.0).as_r32(),
                shape: Shape::Line { width: r32(1.7) },
            },
            Collider {
                position: vec2(8.75, 0.5).as_r32(),
                rotation: Angle::from_degrees(75.0).as_r32(),
                shape: Shape::Line { width: r32(0.95) },
            },
            Collider {
                position: vec2(-8.75, -5.0).as_r32(),
                rotation: Angle::ZERO,
                shape: Shape::Circle { radius: r32(1.2) },
            },
            Collider {
                position: vec2(2.5, 5.0).as_r32(),
                rotation: Angle::ZERO,
                shape: Shape::Circle { radius: r32(0.9) },
            },
            Collider {
                position: vec2(3.5, -4.5).as_r32(),
                rotation: Angle::ZERO,
                shape: Shape::Circle { radius: r32(0.5) },
            },
            Collider {
                position: vec2(-7.5, -0.5).as_r32(),
                rotation: Angle::ZERO,
                shape: Shape::Circle { radius: r32(0.25) },
            },
        ];
        for collider in loading_lights {
            self.util_render.draw_light(
                &Light {
                    base_collider: collider.clone(),
                    collider,
                    lifetime: 0,
                    danger: false,
                    event_id: None,
                    closest_waypoint: (100, WaypointId::Initial),
                },
                THEME.light,
                THEME.dark,
                r32(0.01),
                &self.camera,
                &mut dither_buffer,
            );
        }
        self.dither.finish(self.time, &theme);

        let buffer = &mut self.post.begin(framebuffer.size(), theme.dark);
        geng_utils::texture::DrawTexture::new(self.dither.get_buffer())
            .fit_screen(vec2(0.5, 0.5), buffer)
            .draw(&geng::PixelPerfectCamera, &self.context.geng, buffer);

        {
            // Fake loading bar
            let font_size = 1.0;
            let size = vec2(10.0, 0.8) * font_size;
            let load_bar = Aabb2::point(vec2(0.0, -font_size * 2.0)).extend_symmetric(size / 2.0);
            let fill_bar = load_bar.extend_uniform(-font_size * 0.1);
            let t = (self.time.as_f32() / 1.5 / 3.0).min(1.0);
            let t = crate::util::smoothstep(t);
            let fill_bar = fill_bar.extend_right((t - 1.0) * fill_bar.width());
            self.context
                .geng
                .draw2d()
                .quad(buffer, &self.camera, load_bar, theme.light);
            self.context
                .geng
                .draw2d()
                .quad(buffer, &self.camera, fill_bar, theme.highlight);
        }

        if let Ok(pos) = self
            .camera
            .world_to_screen(framebuffer.size().as_f32(), vec2(0.0, 3.0))
        {
            self.ui_render.draw_texture(
                Aabb2::point(pos),
                &self.context.assets.sprites.title,
                theme.light,
                2.0,
                buffer,
            );
        }
        self.util_render.draw_text(
            self.load_texts[(self.time.as_f32() / 1.5).floor() as usize % self.load_texts.len()],
            vec2(0.0, -1.0),
            TextRenderOptions::new(0.8).color(theme.light),
            &self.camera,
            buffer,
        );
        self.util_render.draw_text(
            "by Nertsal",
            vec2(-5.5, 1.75),
            TextRenderOptions::new(0.7)
                .align(vec2(0.0, 0.5))
                .color(theme.light),
            &self.camera,
            buffer,
        );
        self.util_render.draw_text(
            "music by IcyLava",
            vec2(5.5, 1.75),
            TextRenderOptions::new(0.7)
                .align(vec2(1.0, 0.5))
                .color(theme.light),
            &self.camera,
            buffer,
        );
        // if let Some(picture) = &self.picture {
        //     let pixel_scale =
        //         (buffer.size().as_f32() / picture.size().as_f32()).map(|x| x.floor().max(1.0));
        //     let pixel_scale = pixel_scale.x.min(pixel_scale.y);
        //     geng_utils::texture::DrawTexture::new(picture)
        //         .pixel_perfect(
        //             buffer.size().as_f32() / 2.0,
        //             vec2(0.5, 0.5),
        //             pixel_scale,
        //             &geng::PixelPerfectCamera,
        //             buffer,
        //         )
        //         .draw(&geng::PixelPerfectCamera, &self.context.geng, buffer);
        // }
        self.post.post_process(
            PostVfx {
                time: self.time,
                crt: true,
                rgb_split: 0.0,
            },
            framebuffer,
        );
    }

    fn update(&mut self, delta_time: f64) {
        let delta_time = FloatTime::new(delta_time as f32);
        self.time += delta_time;
    }
}
