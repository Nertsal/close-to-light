use super::*;

use crate::Secrets;

pub struct SplashScreen {
    geng: Geng,
    assets: Rc<Assets>,
    secrets: Option<Secrets>,
    options: Options,
    transition: Option<geng::state::Transition>,

    dither: DitherRender,
    util: UtilRender,

    time: Time,
}

impl SplashScreen {
    pub fn new(
        geng: &Geng,
        assets: &Rc<Assets>,
        secrets: Option<Secrets>,
        options: Options,
    ) -> Self {
        Self {
            geng: geng.clone(),
            assets: assets.clone(),
            secrets,
            options,
            transition: None,

            dither: DitherRender::new(geng, assets),
            util: UtilRender::new(geng, assets),

            time: Time::ZERO,
        }
    }
}

impl geng::State for SplashScreen {
    fn transition(&mut self) -> Option<geng::state::Transition> {
        self.transition.take()
    }

    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        ugli::clear(framebuffer, Some(self.options.theme.dark), None, None);

        let camera = &Camera2d {
            center: vec2::ZERO,
            rotation: Angle::ZERO,
            fov: 12.0,
        };

        self.dither.set_noise((self.time.as_f32() / 3.0).min(1.0));
        let mut dither = self.dither.start();

        self.util.draw_text(
            "PHOTOSENSITIVTY WARNING",
            vec2(0.0, 0.7),
            TextRenderOptions::new(1.2)
                .align(vec2(0.5, 0.0))
                .color(crate::render::THEME.light),
            camera,
            &mut dither,
        );
        let warning = "
This game contains flashing lights which might
trigger seizures for people with photosensitive epilepsy
            ";
        self.util.draw_text(
            warning,
            vec2(0.0, 0.0),
            TextRenderOptions::new(0.7)
                .align(vec2(0.5, 1.0))
                .color(crate::render::THEME.light),
            camera,
            &mut dither,
        );

        self.dither.finish(self.time, &self.options.theme);
        let alpha = (5.0 - self.time.as_f32()).clamp(0.0, 1.0);
        let alpha = crate::util::smoothstep(alpha);
        geng_utils::texture::DrawTexture::new(self.dither.get_buffer())
            .fit_screen(vec2(0.5, 0.5), framebuffer)
            .colored(crate::util::with_alpha(Color::WHITE, alpha))
            .draw(&geng::PixelPerfectCamera, &self.geng, framebuffer);
    }

    fn update(&mut self, delta_time: f64) {
        let delta_time = Time::new(delta_time as f32);
        self.time += delta_time;

        if self.time.as_f32() > 5.0 {
            self.transition = Some(geng::state::Transition::Switch(Box::new(MainMenu::new(
                &self.geng,
                &self.assets,
                self.secrets.take(),
                self.options.clone(),
            ))));
        }
    }
}
