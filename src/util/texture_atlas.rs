use super::*;

use geng_utils::conversions::Vec2RealConversions;

pub struct TextureAtlas {
    texture: Rc<ugli::Texture>,
    uvs: Vec<Aabb2<f32>>,
}

#[derive(Clone)]
pub struct SubTexture {
    pub texture: Rc<ugli::Texture>,
    pub uv: Aabb2<f32>,
}

impl SubTexture {
    pub fn size(&self) -> vec2<usize> {
        (self.texture.size().as_f32() * self.uv.size()).map(|x| x.round() as usize)
    }
}

impl TextureAtlas {
    pub fn new(ugli: &Ugli, textures: &[&ugli::Texture], filter: ugli::Filter) -> Self {
        // TODO: smarter algorithm
        // for example <https://github.com/TeamHypersomnia/rectpack2D?tab=readme-ov-file#algorithm>
        // ^ same as <https://blackpawn.com/texts/lightmaps/>

        let mut width = 0;
        let mut height = 0;
        for texture in textures {
            width += texture.size().x;
            height = height.max(texture.size().y);
        }
        let mut atlas_texture = ugli::Texture::new_uninitialized(ugli, vec2(width, height));
        let mut uvs = Vec::with_capacity(textures.len());
        let mut x = 0;
        for texture in textures {
            let framebuffer = ugli::FramebufferRead::new(
                ugli,
                ugli::ColorAttachmentRead::Texture(texture),
                ugli::DepthAttachmentRead::None,
            );
            framebuffer.copy_to_texture(
                &mut atlas_texture,
                Aabb2::ZERO.extend_positive(texture.size()),
                vec2(x, 0),
            );
            uvs.push(
                Aabb2::point(vec2(x as f32 / width as f32, 0.0)).extend_positive(vec2(
                    texture.size().x as f32 / width as f32,
                    texture.size().y as f32 / height as f32,
                )),
            );
            x += texture.size().x;
        }
        atlas_texture.set_filter(filter);
        Self {
            texture: Rc::new(atlas_texture),
            uvs,
        }
    }
    pub fn get(&self, texture_index: usize) -> SubTexture {
        SubTexture {
            texture: Rc::clone(&self.texture),
            uv: self.uvs[texture_index],
        }
    }
    // pub fn uv(&self, texture_index: usize) -> Aabb2<f32> {
    //     self.uvs[texture_index]
    // }
    pub fn texture(&self) -> &ugli::Texture {
        &self.texture
    }
}
