use super::*;

pub struct DitherRender {
    geng: Geng,
    assets: Rc<Assets>,
    unit_quad: ugli::VertexBuffer<draw2d::TexturedVertex>,
    noise: f32,

    lights_sdf: ugli::Texture,
    target: ugli::Texture,
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
            noise: 1.0,

            lights_sdf: init_buffer(geng.ugli(), size),
            target: init_buffer(geng.ugli(), size),
        }
    }

    fn swap_buffer(&mut self) {
        std::mem::swap(&mut self.lights_sdf, &mut self.target);
    }

    pub fn get_render_size(&self) -> vec2<usize> {
        self.target.size()
    }

    pub fn get_buffer(&self) -> &ugli::Texture {
        &self.target
    }

    pub fn get_lights_sdf(&self) -> &ugli::Texture {
        &self.lights_sdf
    }

    // pub fn update_render_size(&mut self, new_size: vec2<usize>) {
    //     if self.get_render_size() != new_size {
    //         self.double_buffer = init_buffers(self.geng.ugli(), new_size);
    //     }
    // }

    pub fn start(&'_ mut self) -> ugli::Framebuffer<'_> {
        self.swap_buffer();

        let mut framebuffer =
            geng_utils::texture::attach_texture(&mut self.target, self.geng.ugli());
        ugli::clear(&mut framebuffer, Some(Color::TRANSPARENT_BLACK), None, None);

        let mut framebuffer =
            geng_utils::texture::attach_texture(&mut self.lights_sdf, self.geng.ugli());
        ugli::clear(&mut framebuffer, Some(Color::TRANSPARENT_BLACK), None, None);
        framebuffer
    }

    pub fn set_noise(&mut self, noise: f32) {
        self.noise = noise;
    }

    pub fn finish(&'_ mut self, time: FloatTime, theme: &Theme) {
        let mut other_framebuffer =
            geng_utils::texture::attach_texture(&mut self.target, self.geng.ugli());

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
                u_framebuffer_size: self.lights_sdf.size().as_f32(),
                u_pattern_size: self.assets.dither.dither1.size().as_f32(),
                u_noise: self.noise,
                u_color_dark: theme.dark,
                u_color_light: theme.light,
                u_color_danger: theme.danger,
                u_color_highlight: theme.highlight,
                u_texture: &self.lights_sdf,
                u_dither1: &self.assets.dither.dither1,
                u_dither2: &self.assets.dither.dither2,
                u_dither3: &self.assets.dither.dither3,
            ),
            ugli::DrawParameters {
                blend_mode: Some(util::additive()),
                ..Default::default()
            },
        );
    }
}

fn init_buffer(ugli: &Ugli, size: vec2<usize>) -> ugli::Texture {
    let mut first = geng_utils::texture::new_texture(ugli, size);
    first.set_filter(ugli::Filter::Nearest);
    first
}
