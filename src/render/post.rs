use super::*;

/// Renderer responsible for common post-processing effects, such as crt.
pub struct PostRender {
    context: Context,
    unit_quad: ugli::VertexBuffer<draw2d::TexturedVertex>,
    texture: ugli::Texture,
}

impl PostRender {
    pub fn new(context: Context) -> Self {
        Self {
            unit_quad: geng_utils::geometry::unit_quad_geometry(context.geng.ugli()),
            texture: geng_utils::texture::new_texture(context.geng.ugli(), vec2(1, 1)),
            context,
        }
    }

    /// Get access to the internal texture to render into.
    pub fn begin(&mut self, screen_size: vec2<usize>) -> ugli::Framebuffer {
        geng_utils::texture::update_texture_size(
            &mut self.texture,
            screen_size,
            self.context.geng.ugli(),
        );
        let mut buffer =
            geng_utils::texture::attach_texture(&mut self.texture, self.context.geng.ugli());
        ugli::clear(
            &mut buffer,
            Some(self.context.get_options().theme.dark),
            None,
            None,
        );
        buffer
    }

    pub fn post_process(&mut self, framebuffer: &mut ugli::Framebuffer, time: FloatTime) {
        let options = self.context.get_options();
        if options.graphics.crt.enabled {
            ugli::draw(
                framebuffer,
                &self.context.assets.shaders.crt,
                ugli::DrawMode::TriangleFan,
                &self.unit_quad,
                ugli::uniforms! {
                    u_time: time.as_f32(),
                    u_texture: &self.texture,
                    u_curvature: options.graphics.crt.curvature,
                    u_vignette_multiplier: options.graphics.crt.vignette,
                    u_scanlines_multiplier: options.graphics.crt.scanlines,
                },
                ugli::DrawParameters::default(),
            );
        } else {
            self.context.geng.draw2d().textured_quad(
                framebuffer,
                &geng::PixelPerfectCamera,
                Aabb2::ZERO.extend_positive(framebuffer.size().as_f32()),
                &self.texture,
                Color::WHITE,
            );
        }
    }
}
