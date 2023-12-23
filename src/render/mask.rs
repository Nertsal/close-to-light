use super::*;

pub struct MaskedRender {
    geng: Geng,
    assets: Rc<Assets>,
    unit_quad: ugli::VertexBuffer<draw2d::TexturedVertex>,
    mask_texture: ugli::Texture,
    color_texture: ugli::Texture,
}

pub struct Masking<'a> {
    geng: &'a Geng,
    pub mask: ugli::Framebuffer<'a>,
    pub color: ugli::Framebuffer<'a>,
}

impl MaskedRender {
    pub fn new(geng: &Geng, assets: &Rc<Assets>, texture_size: vec2<usize>) -> Self {
        let mut mask_texture = geng_utils::texture::new_texture(geng.ugli(), texture_size);
        mask_texture.set_filter(ugli::Filter::Nearest);

        let mut color_texture = geng_utils::texture::new_texture(geng.ugli(), texture_size);
        color_texture.set_filter(ugli::Filter::Nearest);

        Self {
            geng: geng.clone(),
            assets: assets.clone(),
            unit_quad: geng_utils::geometry::unit_quad_geometry(geng.ugli()),
            mask_texture,
            color_texture,
        }
    }

    // pub fn mask_texture(&self) -> &ugli::Texture {
    //     &self.mask_texture
    // }

    // pub fn color_texture(&self) -> &ugli::Texture {
    //     &self.color_texture
    // }

    pub fn update_size(&mut self, texture_size: vec2<usize>) {
        if self.mask_texture.size() == texture_size {
            return;
        }

        let mut mask_texture = geng_utils::texture::new_texture(self.geng.ugli(), texture_size);
        mask_texture.set_filter(ugli::Filter::Nearest);

        let mut color_texture = geng_utils::texture::new_texture(self.geng.ugli(), texture_size);
        color_texture.set_filter(ugli::Filter::Nearest);

        self.mask_texture = mask_texture;
        self.color_texture = color_texture;
    }

    /// Clears and returns the masking utility buffer to draw into.
    pub fn start(&mut self) -> Masking {
        let mut mask =
            geng_utils::texture::attach_texture(&mut self.mask_texture, self.geng.ugli());
        let mut color =
            geng_utils::texture::attach_texture(&mut self.color_texture, self.geng.ugli());

        ugli::clear(&mut mask, Some(Rgba::TRANSPARENT_BLACK), None, None);
        ugli::clear(&mut color, Some(Rgba::TRANSPARENT_BLACK), None, None);

        Masking {
            geng: &self.geng,
            mask,
            color,
        }
    }

    pub fn draw(&self, parameters: ugli::DrawParameters, framebuffer: &mut ugli::Framebuffer) {
        ugli::draw(
            framebuffer,
            &self.assets.shaders.masked,
            ugli::DrawMode::TriangleFan,
            &self.unit_quad,
            ugli::uniforms! {
                u_mask_texture: &self.mask_texture,
                u_color_texture: &self.color_texture,
            },
            parameters,
        );
    }
}

impl<'a> Masking<'a> {
    pub fn mask_quad(&mut self, aabb: Aabb2<f32>) {
        self.geng.draw2d().draw2d(
            &mut self.mask,
            &geng::PixelPerfectCamera,
            &draw2d::Quad::new(aabb, Rgba::WHITE),
        );
    }
}
