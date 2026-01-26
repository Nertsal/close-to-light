use super::*;

use ctl_assets::GraphicsColorsOptions;

pub struct PostContext {
    pub geng: Geng,
    pub shader_crt: Rc<ugli::Program>,
    pub shader_rgb_split: Rc<ugli::Program>,
    pub shader_color_correction: Rc<ugli::Program>,
}

/// Renderer responsible for common post-processing effects, such as crt.
pub struct PostRender {
    context: PostContext,
    unit_quad: ugli::VertexBuffer<draw2d::TexturedVertex>,
    swap_buffer: (ugli::Texture, ugli::Texture),
}

#[derive(Debug, Clone)]
pub struct PostVfx {
    pub time: FloatTime,
    pub crt: bool,
    pub rgb_split: f32,
    pub colors: GraphicsColorsOptions,
}

fn init_buffers(ugli: &Ugli, size: vec2<usize>) -> (ugli::Texture, ugli::Texture) {
    let mut first = geng_utils::texture::new_texture(ugli, size);
    first.set_filter(ugli::Filter::Nearest);
    let mut second = geng_utils::texture::new_texture(ugli, size);
    second.set_filter(ugli::Filter::Nearest);
    (first, second)
}

impl PostRender {
    pub fn new(context: &Context) -> Self {
        Self::new_with(PostContext {
            geng: context.geng.clone(),
            shader_crt: context.assets.shaders.crt.clone(),
            shader_rgb_split: context.assets.shaders.rgb_split.clone(),
            shader_color_correction: context.assets.shaders.color_correction.clone(),
        })
    }

    pub fn new_with(context: PostContext) -> Self {
        Self {
            unit_quad: geng_utils::geometry::unit_quad_geometry(context.geng.ugli()),
            swap_buffer: init_buffers(context.geng.ugli(), vec2(1, 1)),
            context,
        }
    }

    /// Get access to the internal texture to render into.
    pub fn begin(&'_ mut self, screen_size: vec2<usize>, dark: Color) -> ugli::Framebuffer<'_> {
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
            Some(dark),
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
        ugli::clear(&mut buffer, Some(dark), None, None);
        buffer
    }

    pub fn post_process(
        &mut self,
        options: &Options,
        vfx: PostVfx,
        framebuffer: &mut ugli::Framebuffer,
    ) {
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
                &self.context.shader_crt,
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
                &self.context.shader_rgb_split,
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

        // Color correction
        {
            let (texture, mut buffer) = swap!();
            ugli::draw(
                &mut buffer,
                &self.context.shader_color_correction,
                ugli::DrawMode::TriangleFan,
                &self.unit_quad,
                ugli::uniforms! {
                    u_texture: texture,
                    u_saturation: vfx.colors.saturation,
                    u_blue: vfx.colors.blue,
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
