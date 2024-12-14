use super::*;

use crate::{assets::LoadingAssets, task::Task, ui::layout::AreaOps};

pub struct LoadingScreen<T> {
    geng: Geng,
    assets: Rc<LoadingAssets>,
    unit_quad: ugli::VertexBuffer<draw2d::TexturedVertex>,
    options: Options,
    future: Option<Task<T>>,
    result: Option<T>,

    /// Fake load time so the screen doesnt flash
    min_load_time: f64,
    real_time: f64,
    texts: Vec<&'static str>,
    current_text: usize,
    text_timer: Bounded<f64>,
}

impl<T: 'static> LoadingScreen<T> {
    pub fn new(
        geng: &Geng,
        assets: Rc<LoadingAssets>,
        future: impl Future<Output = T> + 'static,
        insta_load: bool,
    ) -> Self {
        // let height = 360;
        // let size = vec2(height * 16 / 9, height);
        Self {
            geng: geng.clone(),
            assets,
            unit_quad: geng_utils::geometry::unit_quad_geometry(geng.ugli()),
            options: preferences::load(crate::OPTIONS_STORAGE).unwrap_or_default(),
            future: Some(Task::new(geng, future)),
            result: None,

            min_load_time: if insta_load { 0.0 } else { 4.6 },
            real_time: 0.0,
            texts: vec!["Loading assets...", "Please hold", "Loading evil >:3"],
            current_text: 0,
            text_timer: Bounded::new_max(2.0),
        }
    }

    fn check_result(&mut self) -> Option<T> {
        // Poll future
        if let Some(task) = self.future.take() {
            match task.poll() {
                Ok(result) => {
                    self.result = Some(result);
                }
                Err(task) => {
                    self.future = Some(task);
                }
            }
        }

        // Check completion and timer
        if self.real_time > self.min_load_time {
            if let Some(result) = self.result.take() {
                return Some(result);
            }
        }

        None
    }

    pub async fn run(mut self) -> Option<T> {
        let geng = self.geng.clone();
        let mut timer = Timer::new();

        let mut events = geng.window().events();
        while let Some(event) = events.next().await {
            use geng::State;
            match event {
                geng::Event::Draw => {
                    let delta_time = timer.tick().as_secs_f64();
                    let delta_time = delta_time.min(0.05);
                    self.update(delta_time);

                    let window_size = geng.window().real_size();
                    if window_size.x != 0 && window_size.y != 0 {
                        geng.window().with_framebuffer(|framebuffer| {
                            self.draw(framebuffer);
                        });
                    }

                    if let Some(result) = self.check_result() {
                        return Some(result);
                    }
                }
                _ => self.handle_event(event),
            }
        }

        None
    }

    fn draw_text(
        &self,
        framebuffer: &mut ugli::Framebuffer,
        camera: &impl geng::AbstractCamera2d,
        text: impl AsRef<str>,
        position: vec2<impl Float>,
        options: TextRenderOptions,
    ) {
        let font = &self.assets.font;
        let text = text.as_ref();

        let measure = font
            .measure(text, vec2::splat(geng::TextAlign::CENTER))
            .unwrap_or(Aabb2::ZERO.extend_positive(vec2::splat(1.0)));
        let size = measure.size();
        let align = size * (options.align - vec2::splat(0.5)); // Centered by default
        let align = vec2(measure.center().x, 0.0) + align;

        let position = position.map(Float::as_f32);
        let transform = mat3::translate(position.map(Float::as_f32))
            * mat3::scale_uniform(options.size * 0.6) // TODO: figure out what that 0.6 is lmao
            * mat3::translate(-align.rotate(options.rotation))
            * mat3::rotate_around(vec2(measure.center().x, 0.0), options.rotation);

        let framebuffer_size = framebuffer.size();

        font.draw_with(
            text,
            vec2::splat(geng::TextAlign::CENTER),
            |glyphs, texture| {
                ugli::draw(
                    framebuffer,
                    &self.assets.font_shader,
                    ugli::DrawMode::TriangleFan,
                    ugli::instanced(
                        &self.unit_quad,
                        &ugli::VertexBuffer::new_dynamic(self.geng.ugli(), glyphs.to_vec()),
                    ),
                    (
                        ugli::uniforms! {
                            u_texture: texture,
                            u_model_matrix: transform,
                            u_color: options.color,
                            u_smooth: 0.0,
                            u_outline_dist: 0.0 / font.max_distance(),
                            u_outline_color: Color::TRANSPARENT_BLACK,
                        },
                        camera.uniforms(framebuffer_size.map(|x| x as f32)),
                    ),
                    ugli::DrawParameters {
                        blend_mode: Some(ugli::BlendMode::straight_alpha()),
                        depth_func: None,
                        ..Default::default()
                    },
                );
            },
        );
    }
}

impl<T: 'static> geng::State for LoadingScreen<T> {
    fn update(&mut self, delta_time: f64) {
        self.real_time += delta_time;
        self.text_timer.change(-delta_time);
        if self.text_timer.is_min() {
            self.text_timer.set_ratio(1.0);
            self.current_text = (self.current_text + 1) % self.texts.len();
        }
    }

    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        ugli::clear(framebuffer, Some(self.options.theme.dark), None, None);

        let framebuffer_size = framebuffer.size().as_f32();
        let font_size = framebuffer_size.y * 0.07;
        let theme = self.options.theme;

        let screen = Aabb2::ZERO.extend_positive(framebuffer_size);
        let camera = &geng::PixelPerfectCamera;

        // Fake loading bar
        let size = vec2(10.0, 0.8) * font_size;
        let load_bar = Aabb2::point(screen.center() + vec2(0.0, -font_size * 2.0))
            .extend_symmetric(size / 2.0);
        let fill_bar = load_bar.extend_uniform(-font_size * 0.1);
        let t = (self.real_time / self.min_load_time.max(2.0)).min(1.0) as f32;
        let t = crate::util::smoothstep(t);
        let fill_bar = fill_bar.extend_right((t - 1.0) * fill_bar.width());
        self.geng
            .draw2d()
            .quad(framebuffer, camera, load_bar, theme.light);
        self.geng
            .draw2d()
            .quad(framebuffer, camera, fill_bar, theme.highlight);

        // Title
        let title = geng_utils::pixel::pixel_perfect_aabb(
            screen.align_pos(vec2(0.5, 0.8)),
            vec2(0.5, 0.5),
            self.assets.title.size() * (framebuffer.size().y / 360),
            camera,
            framebuffer_size,
        );
        self.geng.draw2d().textured_quad(
            framebuffer,
            camera,
            title,
            &self.assets.title,
            theme.light,
        );

        // Funny text
        if let Some(text) = self.texts.get(self.current_text) {
            let pos = screen.align_pos(vec2(0.5, 0.5));
            self.draw_text(
                framebuffer,
                camera,
                text,
                pos,
                TextRenderOptions::new(font_size).color(theme.light),
            );
        }

        // Smol funny text
        let pos = screen.align_pos(vec2(0.5, 0.0)) + vec2(0.0, 1.0) * font_size;
        self.draw_text(
            framebuffer,
            camera,
            "this progress bar is totally fake btw",
            pos,
            TextRenderOptions::new(font_size * 0.4).color(theme.light),
        );
    }
}
