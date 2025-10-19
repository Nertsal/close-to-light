use ctl_render_core::TextRenderOptions;
use geng::prelude::*;

#[derive(ugli::Vertex)]
struct QuadVertex {
    a_pos: vec2<f32>,
}

struct Glyph {
    texture_pos: Aabb2<usize>,
}

#[derive(Deserialize)]
struct GlyphRangeConfig {
    start: char,
    end: char,
    at: vec2<usize>,
}

#[derive(Deserialize)]
struct Config {
    space_size: f32,
    tile_size: vec2<usize>,
    kerning: f32,
    error: vec2<usize>,
    chars: HashMap<char, vec2<usize>>,
    ranges: Vec<GlyphRangeConfig>,
}

pub struct Font {
    ugli: Ugli,
    program: ugli::Program,
    quad: ugli::VertexBuffer<QuadVertex>,
    base_size: f32,
    config: Config,
    texture: ugli::Texture,
    error: Glyph,
    glyphs: HashMap<char, Glyph>,
}

impl Font {
    pub fn base_size(&self) -> f32 {
        self.base_size
    }

    pub fn measure(&self, text: &str, size: f32) -> Aabb2<f32> {
        let mut measure = Aabb2::ZERO;
        self.draw_with_impl(text, geng::TextAlign::LEFT, |glyphs, _texture| {
            for glyph in glyphs {
                measure = Aabb2 {
                    min: vec2(
                        partial_min(measure.min.x, glyph.i_pos.x),
                        partial_min(measure.min.y, glyph.i_pos.y),
                    ),
                    max: vec2(
                        partial_max(measure.max.x, glyph.i_pos.x + glyph.i_size.x),
                        partial_max(measure.max.y, glyph.i_pos.y + glyph.i_size.y),
                    ),
                };
            }
        });
        measure.map(|x| x * size)
    }

    pub fn draw(
        &self,
        framebuffer: &mut ugli::Framebuffer,
        camera: &impl geng::AbstractCamera2d,
        text: impl AsRef<str>,
        position: vec2<impl Float>,
        options: TextRenderOptions,
    ) {
        let text = text.as_ref();
        let position = position.map(Float::as_f32);
        self.draw_with(
            framebuffer,
            camera,
            text,
            0.0,
            options,
            mat3::translate(position),
            ugli::DrawParameters {
                depth_func: None,
                blend_mode: Some(ugli::BlendMode::straight_alpha()),
                ..Default::default()
            },
        );
    }

    #[allow(clippy::too_many_arguments)]
    pub fn draw_with(
        &self,
        framebuffer: &mut ugli::Framebuffer,
        camera: &impl AbstractCamera2d,
        text: impl AsRef<str>,
        z_index: f32,
        options: TextRenderOptions,
        mut transform: mat3<f32>,
        params: ugli::DrawParameters,
    ) {
        let text = text.as_ref();

        // let mut position = position;
        for line in text.lines() {
            let measure = self.measure(line, 1.0);
            let size = measure.size();
            let align = options.align - vec2(0.5, 0.0); // Account for default alignment
            // let descent = -self.descent() * font_size;
            let align = vec2(
                size.x * align.x,
                // descent + (measure.max.y - descent) * align.y,
                size.y * align.y,
            );

            let line_transform = transform
                * mat3::rotate(options.rotation)
                * mat3::scale_uniform(options.size)
                * mat3::translate(-align);

            self.draw_with_impl(line, geng::TextAlign::CENTER, |glyphs, texture| {
                let glyphs = ugli::VertexBuffer::new_dynamic(&self.ugli, glyphs.to_vec());
                let framebuffer_size = framebuffer.size();
                ugli::draw(
                    framebuffer,
                    &self.program,
                    ugli::DrawMode::TriangleFan,
                    ugli::instanced(&self.quad, &glyphs),
                    (
                        ugli::uniforms! {
                            u_z: z_index,
                            u_texture: texture,
                            u_texture_size: texture.size(),
                            u_model_matrix: line_transform,
                            u_color: options.color,
                        },
                        camera.uniforms(framebuffer_size.map(|x| x as f32)),
                    ),
                    params.clone(),
                );
            });
            transform *= mat3::translate(vec2(0.0, -options.size));
        }
    }

    fn draw_with_impl(
        &self,
        text: &str,
        align: geng::TextAlign,
        f: impl FnOnce(&[geng::font::GlyphInstance], &ugli::Texture),
    ) {
        let mut pos = 0.0;
        let mut glyphs = Vec::new();
        for (i, c) in text.chars().enumerate() {
            if c == ' ' {
                pos += self.config.space_size / self.base_size;
                continue;
            }
            if i != 0 {
                pos += self.config.kerning / self.base_size;
            }
            let glyph = self.glyphs.get(&c).unwrap_or(&self.error);
            glyphs.push(geng::font::GlyphInstance {
                i_pos: vec2(pos, 0.0),
                i_size: glyph.texture_pos.size().map(|x| x as f32) / self.base_size,
                i_uv_pos: glyph.texture_pos.bottom_left().map(|x| x as f32)
                    / self.texture.size().map(|x| x as f32),
                i_uv_size: glyph.texture_pos.size().map(|x| x as f32)
                    / self.texture.size().map(|x| x as f32),
            });
            pos += glyph.texture_pos.width() as f32 / self.base_size;
        }
        for glyph in &mut glyphs {
            glyph.i_pos.x -= pos * align.0;
        }
        f(&glyphs, &self.texture);
    }
}

impl geng::asset::Load for Font {
    type Options = ();
    fn load(
        manager: &geng::asset::Manager,
        path: &std::path::Path,
        _options: &Self::Options,
    ) -> geng::asset::Future<Self> {
        let manager = manager.clone();
        let path = path.to_owned();
        async move {
            let config: Config = manager.load_serde(path.join("config.toml")).await?;
            let texture: ugli::Texture = manager.load(path.join("texture.png")).await?;
            let program: ugli::Program = manager.load(path.join("shader.glsl")).await?;
            assert_eq!(
                texture.size().x % config.tile_size.x,
                0,
                "texture has to fit an exact integer number of tiles"
            );
            assert_eq!(
                texture.size().y % config.tile_size.y,
                0,
                "texture has to fit an exact integer number of tiles"
            );
            let texture_tile_size = texture.size() / config.tile_size;
            let mut glyphs = HashMap::new();
            let framebuffer = ugli::FramebufferRead::new_color(
                manager.ugli(),
                ugli::ColorAttachmentRead::Texture(&texture),
            );
            let color_data = framebuffer.read_color();
            let glyph_at = |pos: vec2<usize>| -> Glyph {
                let mut texture_pos =
                    Aabb2::point(vec2(pos.x, texture_tile_size.y - 1 - pos.y) * config.tile_size)
                        .extend_positive(config.tile_size);
                while texture_pos.width() != 0 {
                    if (texture_pos.min.y..texture_pos.max.y)
                        .any(|y| color_data.get(texture_pos.min.x, y).a != 0)
                    {
                        break;
                    }
                    texture_pos.min.x += 1;
                }
                while texture_pos.width() != 0 {
                    if (texture_pos.min.y..texture_pos.max.y)
                        .any(|y| color_data.get(texture_pos.max.x - 1, y).a != 0)
                    {
                        break;
                    }
                    texture_pos.max.x -= 1;
                }
                Glyph { texture_pos }
            };
            for range in &config.ranges {
                let mut pos = range.at;
                for c in range.start..=range.end {
                    assert!(pos.x < texture_tile_size.x);
                    assert!(pos.y < texture_tile_size.y);

                    glyphs.insert(c, glyph_at(pos));

                    pos.x += 1;
                    if pos.x >= texture_tile_size.x {
                        pos.y += 1;
                        pos.x = 0;
                    }
                }
            }
            for (&c, &pos) in &config.chars {
                glyphs.insert(c, glyph_at(pos));
            }
            Ok(Self {
                ugli: manager.ugli().clone(),
                program,
                quad: ugli::VertexBuffer::new_static(
                    manager.ugli(),
                    [[0, 0], [0, 1], [1, 1], [1, 0]]
                        .into_iter()
                        .map(|[x, y]| QuadVertex {
                            a_pos: vec2(x as f32, y as f32),
                        })
                        .collect(),
                ),
                base_size: config.tile_size.y as f32,
                error: glyph_at(config.error),
                config,
                texture,
                glyphs,
            })
        }
        .boxed_local()
    }
    const DEFAULT_EXT: Option<&'static str> = None;
}
