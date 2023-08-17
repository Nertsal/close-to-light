use super::*;

#[allow(dead_code)]
pub struct GameRender {
    geng: Geng,
    assets: Rc<Assets>,
    render: Render,
    util: UtilRender,
}

impl GameRender {
    pub fn new(geng: &Geng, assets: &Rc<Assets>) -> Self {
        Self {
            geng: geng.clone(),
            assets: assets.clone(),
            render: Render::new(geng, assets),
            util: UtilRender::new(geng, assets),
        }
    }

    pub fn get_render_size(&self) -> vec2<usize> {
        self.render.get_render_size()
    }

    pub fn draw_world(&mut self, model: &Model, old_framebuffer: &mut ugli::Framebuffer) {
        let mut framebuffer = self.render.start();

        let camera = &model.camera;

        // Telegraphs
        for tele in &model.telegraphs {
            self.util.draw_outline(
                &tele.light.collider,
                0.05,
                COLOR_LIGHT,
                camera,
                &mut framebuffer,
            );
        }

        // Lights
        for light in &model.lights {
            self.util
                .draw_collider(&light.collider, COLOR_LIGHT, camera, &mut framebuffer);
        }

        // Player
        let player = model.player.collider.clone();
        // player.position += model.player.shake;
        // self.util
        //     .draw_collider(&player, COLOR_LIGHT, camera, &mut framebuffer);
        self.util
            .draw_outline(&player, 0.05, COLOR_PLAYER, camera, &mut framebuffer);

        if let State::Lost | State::Finished = model.state {
            self.util
                .draw_button(&model.restart_button, "RESTART", camera, &mut framebuffer);

            self.util.draw_text(
                "made in rust btw",
                vec2(0.0, 4.0).as_r32(),
                0.5,
                vec2::splat(0.5),
                COLOR_DARK,
                camera,
                &mut framebuffer,
            );
        }

        match model.state {
            State::Playing => {}
            State::Lost => {
                self.util.draw_text(
                    "YOU FAILED TO CHASE THE LIGHT",
                    vec2(0.0, 3.0).as_r32(),
                    1.0,
                    vec2::splat(0.5),
                    COLOR_LIGHT,
                    camera,
                    &mut framebuffer,
                );
            }
            State::Finished => {
                self.util.draw_text(
                    "YOU CAUGHT THE LIGHT",
                    vec2(0.0, 3.0).as_r32(),
                    1.0,
                    vec2::splat(0.5),
                    COLOR_LIGHT,
                    camera,
                    &mut framebuffer,
                );
            }
        }

        // let t = R32::ONE - (model.player.fear_meter.get_ratio() / r32(0.5)).min(R32::ONE);
        let t = if model.player.is_in_light {
            R32::ONE
        } else {
            R32::ZERO
        };
        self.render.dither(model.real_time, t);

        let aabb = Aabb2::ZERO.extend_positive(old_framebuffer.size().as_f32());
        draw_texture_fit(
            self.render.get_buffer(),
            aabb,
            vec2(0.5, 0.5),
            &geng::PixelPerfectCamera,
            &self.geng,
            old_framebuffer,
        );
    }

    pub fn draw_ui(&mut self, model: &Model, framebuffer: &mut ugli::Framebuffer) {
        let camera = &geng::PixelPerfectCamera;
        let screen = Aabb2::ZERO.extend_positive(framebuffer.size().as_f32());

        let font_size = screen.height() * 0.05;

        // Fear meter
        let fear = Aabb2::point(
            geng_utils::layout::aabb_pos(screen, vec2(0.5, 0.0)) + vec2(0.0, 1.0) * font_size,
        )
        .extend_symmetric(vec2(14.0, 0.0) * font_size / 2.0)
        .extend_up(font_size);
        self.geng.draw2d().draw2d(
            framebuffer,
            camera,
            &draw2d::Quad::new(fear.extend_uniform(font_size * 0.1), COLOR_LIGHT),
        );
        self.geng
            .draw2d()
            .draw2d(framebuffer, camera, &draw2d::Quad::new(fear, COLOR_DARK));
        self.geng.draw2d().draw2d(
            framebuffer,
            camera,
            &draw2d::Quad::new(
                fear.extend_symmetric(
                    vec2(
                        -model.player.fear_meter.get_ratio().as_f32() * fear.width(),
                        0.0,
                    ) / 2.0,
                ),
                COLOR_LIGHT,
            ),
        );
    }
}
