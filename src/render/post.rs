use super::*;

/// Renderer responsible for common post-processing effects, such as crt.
pub struct PostRender {
    context: Context,
    unit_quad: ugli::VertexBuffer<draw2d::TexturedVertex>,
    swap_buffer: (ugli::Texture, ugli::Texture),
}

#[derive(Debug, Clone)]
pub struct PostVfx {
    pub time: FloatTime,
    pub crt: bool,
    pub rgb_split: f32,
}

fn init_buffers(ugli: &Ugli, size: vec2<usize>) -> (ugli::Texture, ugli::Texture) {
    let mut first = geng_utils::texture::new_texture(ugli, size);
    first.set_filter(ugli::Filter::Nearest);
    let mut second = geng_utils::texture::new_texture(ugli, size);
    second.set_filter(ugli::Filter::Nearest);
    (first, second)
}

impl PostRender {
    pub fn new(context: Context) -> Self {
        Self {
            unit_quad: geng_utils::geometry::unit_quad_geometry(context.geng.ugli()),
            swap_buffer: init_buffers(context.geng.ugli(), vec2(1, 1)),
            context,
        }
    }

    /// Get access to the internal texture to render into.
    pub fn begin(&'_ mut self, screen_size: vec2<usize>) -> ugli::Framebuffer<'_> {
        geng_utils::texture::update_texture_size(
            &mut self.swap_buffer.0,
            screen_size,
            self.context.geng.ugli(),
        );
        ugli::clear(
            &mut geng_utils::texture::attach_texture(
                &mut self.swap_buffer.0,
                self.context.geng.ugli(),
            ),
            Some(self.context.get_options().theme.dark),
            None,
            None,
        );

        geng_utils::texture::update_texture_size(
            &mut self.swap_buffer.1,
            screen_size,
            self.context.geng.ugli(),
        );
        let mut buffer =
            geng_utils::texture::attach_texture(&mut self.swap_buffer.1, self.context.geng.ugli());
        ugli::clear(
            &mut buffer,
            Some(self.context.get_options().theme.dark),
            None,
            None,
        );
        buffer
    }

    pub fn continue_render(&mut self) -> ugli::Framebuffer<'_> {
        geng_utils::texture::attach_texture(&mut self.swap_buffer.1, self.context.geng.ugli())
    }

    pub fn post_process(&mut self, vfx: PostVfx, framebuffer: &mut ugli::Framebuffer) {
        let options = self.context.get_options();

        macro_rules! swap {
            () => {{
                std::mem::swap(&mut self.swap_buffer.0, &mut self.swap_buffer.1);
                let buffer = geng_utils::texture::attach_texture(
                    &mut self.swap_buffer.1,
                    self.context.geng.ugli(),
                );
                (&self.swap_buffer.0, buffer)
            }};
        }

        // CRT
        if vfx.crt {
            let (texture, mut buffer) = swap!();
            ugli::draw(
                &mut buffer,
                &self.context.assets.shaders.crt,
                ugli::DrawMode::TriangleFan,
                &self.unit_quad,
                ugli::uniforms! {
                    u_time: vfx.time.as_f32(),
                    u_texture: texture,
                    u_curvature: options.graphics.crt.curvature,
                    u_vignette_multiplier: options.graphics.crt.vignette,
                    u_scanlines_multiplier: options.graphics.crt.scanlines,
                },
                ugli::DrawParameters::default(),
            );
        }

        // RGB split
        {
            let (texture, mut buffer) = swap!();
            ugli::draw(
                &mut buffer,
                &self.context.assets.shaders.rgb_split,
                ugli::DrawMode::TriangleFan,
                &self.unit_quad,
                ugli::uniforms! {
                    u_time: vfx.time.as_f32(),
                    u_texture: texture,
                    u_offset: 0.01 * vfx.rgb_split,
                },
                ugli::DrawParameters::default(),
            );
        }

        self.context.geng.draw2d().textured_quad(
            framebuffer,
            &geng::PixelPerfectCamera,
            Aabb2::ZERO.extend_positive(framebuffer.size().as_f32()),
            &self.swap_buffer.1,
            Color::WHITE,
        );
    }
}
