use ctl_render_core::TextRenderOptions;
use geng::prelude::*;
use geng_utils::conversions::Vec2RealConversions;

#[derive(ugli::Vertex, Debug, Clone, Copy)]
struct Vertex {
    a_pos: vec2<f32>,
    a_vt: vec2<f32>,
}

pub struct Font {
    ugli: Ugli,
    font: geng::Font,
    program: ugli::Program,
}

impl Font {
    pub fn new(manager: &geng::asset::Manager, data: Vec<u8>) -> anyhow::Result<Font> {
        Ok(Self {
            ugli: manager.ugli().clone(),
            font: geng::Font::new(
                manager.ugli(),
                &data,
                &geng::font::Options {
                    pixel_size: 64.0,
                    max_distance: 0.25,
                    antialias: false,
                    distance_mode: geng::font::DistanceMode::Max,
                },
            )?,
            program: manager.shader_lib().compile(include_str!("font.glsl"))?,
        })
    }

    pub fn descent(&self) -> f32 {
        self.font.descender()
    }

    pub fn measure(&self, text: &str, size: f32) -> Aabb2<f32> {
        self.font
            .measure(text, vec2::splat(geng::TextAlign::CENTER))
            .map_or(Aabb2::ZERO.extend_positive(vec2(size, size)), |measure| {
                measure.map(|x| x * size)
            })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn draw_with(
        &self,
        framebuffer: &mut ugli::Framebuffer,
        camera: &impl AbstractCamera2d,
        text: impl AsRef<str>,
        z_index: f32,
        options: TextRenderOptions,
        transform: mat3<f32>,
        params: ugli::DrawParameters,
    ) {
        let text = text.as_ref();
        let transform = transform * mat3::scale_uniform(options.size);
        let outline_size = 0.0;
        let outline_color = Rgba::TRANSPARENT_BLACK;
        self.font.draw_with(
            text,
            options.align.map(geng::TextAlign),
            |glyphs, texture| {
                let framebuffer_size = framebuffer.size();
                ugli::draw(
                    framebuffer,
                    &self.program,
                    ugli::DrawMode::TriangleFan,
                    // TODO: don't create VBs each time
                    ugli::instanced(
                        &ugli::VertexBuffer::new_dynamic(
                            &self.ugli,
                            Aabb2::point(vec2::ZERO)
                                .extend_positive(vec2(1.0, 1.0))
                                .corners()
                                .into_iter()
                                .map(|v| Vertex { a_pos: v, a_vt: v })
                                .collect(),
                        ),
                        &ugli::VertexBuffer::new_dynamic(&self.ugli, glyphs.to_vec()),
                    ),
                    (
                        ugli::uniforms! {
                            u_z: z_index,
                            u_texture: texture,
                            u_model_matrix: transform,
                            u_color: options.color,
                            u_outline_dist: outline_size / self.font.max_distance(),
                            u_outline_color: outline_color,
                        },
                        camera.uniforms(framebuffer_size.map(|x| x as f32)),
                    ),
                    params,
                );
            },
        );
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
        let framebuffer_size = framebuffer.size().as_f32();

        let position = position.map(Float::as_f32);
        let position = ctl_util::world_to_screen(camera, framebuffer_size, position);

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

        // let mut position = position;
        // for line in text.lines() {
        //     let measure = self.measure(line, font_size);
        //     let size = measure.size();
        //     let align = size * (options.align - vec2::splat(0.5)); // Centered by default
        //     let descent = -self.descent() * font_size;
        //     let align = vec2(
        //         measure.center().x + align.x,
        //         descent + (measure.max.y - descent) * options.align.y,
        //     );

        //     let transform = mat3::translate(position)
        //         * mat3::rotate(options.rotation)
        //         * mat3::translate(-align);

        //     self.draw_with(
        //         framebuffer,
        //         line,
        //         0.0,
        //         font_size,
        //         options.color,
        //         transform,
        //         ugli::DrawParameters {
        //             blend_mode: Some(ugli::BlendMode::straight_alpha()),
        //             ..default()
        //         },
        //     );
        //     position.y -= options.size;
        // }
    }
}

impl geng::asset::Load for Font {
    type Options = ();

    fn load(
        manager: &geng::asset::Manager,
        path: &std::path::Path,
        &(): &Self::Options,
    ) -> geng::asset::Future<Self> {
        let manager = manager.clone();
        let path = path.to_owned();
        async move {
            let data = file::load_bytes(path).await?;
            let font = Self::new(&manager, data)?;
            Ok(font)
        }
        .boxed_local()
    }

    const DEFAULT_EXT: Option<&'static str> = Some("ttf");
}
