use super::*;

pub struct DitherRender {
    geng: Geng,
    assets: Rc<Assets>,
    unit_quad: ugli::VertexBuffer<draw2d::TexturedVertex>,
    double_buffer: (ugli::Texture, ugli::Texture),
    noise: f32,
}

impl DitherRender {
    pub fn new(geng: &Geng, assets: &Rc<Assets>) -> Self {
        let height = 360;
        let size = vec2(height * 16 / 9, height);
        Self::new_sized(geng, assets, size)
    }

    pub fn new_sized(geng: &Geng, assets: &Rc<Assets>, size: vec2<usize>) -> Self {
        Self {
            geng: geng.clone(),
            assets: assets.clone(),
            unit_quad: geng_utils::geometry::unit_quad_geometry(geng.ugli()),
            double_buffer: init_buffers(geng.ugli(), size),
            noise: 1.0,
        }
    }

    fn swap_buffer(&mut self) {
        std::mem::swap(&mut self.double_buffer.0, &mut self.double_buffer.1);
    }

    pub fn get_render_size(&self) -> vec2<usize> {
        self.double_buffer.0.size()
    }

    pub fn get_buffer(&self) -> &ugli::Texture {
        &self.double_buffer.0
    }

    // pub fn update_render_size(&mut self, new_size: vec2<usize>) {
    //     if self.get_render_size() != new_size {
    //         self.double_buffer = init_buffers(self.geng.ugli(), new_size);
    //     }
    // }

    pub fn start(&mut self) -> ugli::Framebuffer {
        let mut framebuffer =
            geng_utils::texture::attach_texture(&mut self.double_buffer.1, self.geng.ugli());
        ugli::clear(&mut framebuffer, Some(Color::TRANSPARENT_BLACK), None, None);

        let mut framebuffer =
            geng_utils::texture::attach_texture(&mut self.double_buffer.0, self.geng.ugli());
        ugli::clear(&mut framebuffer, Some(Color::TRANSPARENT_BLACK), None, None);
        framebuffer
    }

    pub fn set_noise(&mut self, noise: f32) {
        self.noise = noise;
    }

    pub fn finish(&mut self, time: Time, theme: &Theme) -> ugli::Framebuffer {
        let mut other_framebuffer =
            geng_utils::texture::attach_texture(&mut self.double_buffer.1, self.geng.ugli());

        let timespan = 32.0;
        let t = time.as_f32();
        let t = ((t / timespan / 2.0).fract() * timespan * 2.0 - timespan).abs();

        ugli::draw(
            &mut other_framebuffer,
            &self.assets.dither.dither_shader,
            ugli::DrawMode::TriangleFan,
            &self.unit_quad,
            ugli::uniforms!(
                u_time: t,
                u_framebuffer_size: self.double_buffer.0.size().as_f32(),
                u_pattern_size: self.assets.dither.dither1.size().as_f32(),
                u_noise: self.noise,
                u_color_dark: theme.dark,
                u_color_light: theme.light,
                u_color_danger: theme.danger,
                u_texture: &self.double_buffer.0,
                u_dither1: &self.assets.dither.dither1,
                u_dither2: &self.assets.dither.dither2,
                u_dither3: &self.assets.dither.dither3,
            ),
            ugli::DrawParameters {
                blend_mode: Some(util::additive()),
                ..Default::default()
            },
        );
        self.swap_buffer();
        geng_utils::texture::attach_texture(&mut self.double_buffer.0, self.geng.ugli())
    }
}

fn init_buffers(ugli: &Ugli, size: vec2<usize>) -> (ugli::Texture, ugli::Texture) {
    let mut first = geng_utils::texture::new_texture(ugli, size);
    first.set_filter(ugli::Filter::Nearest);
    let mut second = geng_utils::texture::new_texture(ugli, size);
    second.set_filter(ugli::Filter::Nearest);
    (first, second)
}
