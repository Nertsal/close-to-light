use super::*;

pub struct MaskedStack {
    geng: Geng,
    assets: Rc<Assets>,
    unit_quad: Rc<ugli::VertexBuffer<draw2d::TexturedVertex>>,
    size: vec2<usize>,

    current_depth: usize,
    stack: VecDeque<MaskFrame>,
}

pub struct MaskFrame {
    geng: Geng,
    assets: Rc<Assets>,
    unit_quad: Rc<ugli::VertexBuffer<draw2d::TexturedVertex>>,
    mask_texture: ugli::Texture,
    color_texture: ugli::Texture,
    depth_buffer: ugli::Renderbuffer<ugli::DepthComponent>,
}

pub struct MaskingFrame<'a> {
    geng: &'a Geng,
    pub mask: ugli::Framebuffer<'a>,
    pub color: ugli::Framebuffer<'a>,
}

impl MaskedStack {
    pub fn new(geng: &Geng, assets: &Rc<Assets>) -> Self {
        Self {
            geng: geng.clone(),
            assets: assets.clone(),
            unit_quad: Rc::new(geng_utils::geometry::unit_quad_geometry(geng.ugli())),
            size: vec2(1, 1),

            current_depth: 0,
            stack: VecDeque::new(),
        }
    }

    fn new_frame(&self) -> MaskFrame {
        let mut mask_texture = geng_utils::texture::new_texture(self.geng.ugli(), self.size);
        mask_texture.set_filter(ugli::Filter::Nearest);

        let mut color_texture = geng_utils::texture::new_texture(self.geng.ugli(), self.size);
        color_texture.set_filter(ugli::Filter::Nearest);

        let depth_buffer = ugli::Renderbuffer::new(self.geng.ugli(), self.size);

        MaskFrame {
            geng: self.geng.clone(),
            assets: self.assets.clone(),
            unit_quad: self.unit_quad.clone(),
            mask_texture,
            color_texture,
            depth_buffer,
        }
    }

    pub fn update_size(&mut self, texture_size: vec2<usize>) {
        if self.size == texture_size {
            return;
        }
        self.size = texture_size;

        self.stack = (0..self.stack.len()).map(|_| self.new_frame()).collect();
    }

    pub fn pop_mask(&mut self) -> MaskFrame {
        if self.stack.is_empty() {
            self.stack.push_back(self.new_frame());
        }
        let frame = self
            .stack
            .pop_front()
            .expect("frames are created as needed");
        self.current_depth += 1;
        frame
    }

    pub fn return_mask(&mut self, mask: MaskFrame) {
        self.stack.push_front(mask);
    }

    // pub fn draw(&self, parameters: ugli::DrawParameters, framebuffer: &mut ugli::Framebuffer) {
    //     assert!(self.current_depth > 0, "some masks were not returned");
    //     if let Some(frame) = self.stack.front() {
    //         frame.draw(parameters, framebuffer);
    //     }
    // }
}

impl MaskFrame {
    pub fn start(&mut self) -> MaskingFrame<'_> {
        let mut mask =
            geng_utils::texture::attach_texture(&mut self.mask_texture, self.geng.ugli());
        let mut color = ugli::Framebuffer::new(
            self.geng.ugli(),
            ugli::ColorAttachment::Texture(&mut self.color_texture),
            ugli::DepthAttachment::Renderbuffer(&mut self.depth_buffer),
        );

        ugli::clear(&mut mask, Some(Rgba::TRANSPARENT_BLACK), None, None);
        ugli::clear(&mut color, Some(Rgba::TRANSPARENT_BLACK), Some(1.0), None);

        MaskingFrame {
            geng: &self.geng,
            mask,
            color,
        }
    }

    pub fn draw(
        &self,
        depth: f32,
        parameters: ugli::DrawParameters,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        ugli::draw(
            framebuffer,
            &self.assets.shaders.masked,
            ugli::DrawMode::TriangleFan,
            &*self.unit_quad,
            ugli::uniforms! {
                u_mask_texture: &self.mask_texture,
                u_color_texture: &self.color_texture,
                u_depth: depth,
            },
            parameters,
        );
    }
}

impl MaskingFrame<'_> {
    pub fn mask_quad(&mut self, aabb: Aabb2<f32>) {
        self.geng.draw2d().draw2d(
            &mut self.mask,
            &geng::PixelPerfectCamera,
            &draw2d::Quad::new(aabb, Rgba::WHITE),
        );
    }
}

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
    pub fn start(&'_ mut self) -> Masking<'_> {
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

impl Masking<'_> {
    pub fn mask_quad(&mut self, aabb: Aabb2<f32>) {
        self.geng.draw2d().draw2d(
            &mut self.mask,
            &geng::PixelPerfectCamera,
            &draw2d::Quad::new(aabb, Rgba::WHITE),
        );
    }
}
