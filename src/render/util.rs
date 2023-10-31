use crate::ui::widget::*;

use super::*;

pub struct UtilRender {
    geng: Geng,
    assets: Rc<Assets>,
}

#[derive(Debug, Clone, Copy)]
pub struct TextRenderOptions {
    pub size: f32,
    pub align: vec2<f32>,
    pub color: Color,
}

#[derive(Debug, Clone, Copy)]
pub struct DashRenderOptions {
    pub width: f32,
    pub color: Color,
    pub dash_length: f32,
    pub space_length: f32,
}

impl TextRenderOptions {
    pub fn new(size: f32) -> Self {
        Self { size, ..default() }
    }

    pub fn size(self, size: f32) -> Self {
        Self { size, ..self }
    }

    pub fn align(self, align: vec2<f32>) -> Self {
        Self { align, ..self }
    }

    pub fn color(self, color: Color) -> Self {
        Self { color, ..self }
    }
}

impl Default for TextRenderOptions {
    fn default() -> Self {
        Self {
            size: 1.0,
            align: vec2::splat(0.5),
            color: Color::WHITE,
        }
    }
}

impl UtilRender {
    pub fn new(geng: &Geng, assets: &Rc<Assets>) -> Self {
        Self {
            geng: geng.clone(),
            assets: assets.clone(),
        }
    }

    pub fn draw_text(
        &self,
        text: impl AsRef<str>,
        position: vec2<impl Float>,
        options: TextRenderOptions,
        camera: &impl geng::AbstractCamera2d,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        let text = text.as_ref();
        let font = self.geng.default_font();
        let size = font
            .measure(text, options.align.map(geng::TextAlign))
            .map_or(vec2::splat(1.0), |aabb| aabb.size());
        let align = size * (options.align - vec2::splat(0.5)); // Centered by default
        font.draw(
            framebuffer,
            camera,
            text,
            vec2::splat(geng::TextAlign::CENTER),
            mat3::translate(position.map(Float::as_f32))
                * mat3::scale_uniform(options.size)
                * mat3::translate(vec2(0.0, -0.25) - align),
            options.color,
        );
    }

    pub fn draw_collider(
        &self,
        collider: &Collider,
        color: Rgba<f32>,
        camera: &impl geng::AbstractCamera2d,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        match collider.shape {
            Shape::Circle { radius } => {
                self.geng.draw2d().draw2d(
                    framebuffer,
                    camera,
                    &draw2d::TexturedQuad::colored(
                        Aabb2::ZERO.extend_symmetric(vec2(radius.as_f32(), radius.as_f32())),
                        &self.assets.sprites.radial_gradient,
                        color,
                    )
                    .translate(collider.position.as_f32()),
                );
            }
            Shape::Line { width } => {
                let inf = 999.0;
                self.geng.draw2d().draw2d(
                    framebuffer,
                    camera,
                    &draw2d::TexturedQuad::colored(
                        Aabb2::ZERO.extend_symmetric(vec2(inf, width.as_f32()) / 2.0),
                        &self.assets.sprites.linear_gradient,
                        color,
                    )
                    .rotate(collider.rotation.map(Coord::as_f32))
                    .translate(collider.position.as_f32()),
                );
            }
            Shape::Rectangle { width, height } => {
                self.geng.draw2d().draw2d(
                    framebuffer,
                    camera,
                    &draw2d::Quad::new(
                        Aabb2::ZERO.extend_symmetric(vec2(width.as_f32(), height.as_f32()) / 2.0),
                        color,
                    )
                    .rotate(collider.rotation.map(Coord::as_f32))
                    .translate(collider.position.as_f32()),
                );
            }
        }
    }

    pub fn draw_outline(
        &self,
        collider: &Collider,
        outline_width: f32,
        color: Rgba<f32>,
        camera: &impl geng::AbstractCamera2d,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        match collider.shape {
            Shape::Circle { radius } => {
                self.geng.draw2d().draw2d(
                    framebuffer,
                    camera,
                    &draw2d::Ellipse::circle_with_cut(
                        collider.position.as_f32(),
                        radius.as_f32() - outline_width,
                        radius.as_f32(),
                        color,
                    ),
                );
            }
            Shape::Line { width } => {
                let inf = 1e3; // camera.fov;
                self.geng.draw2d().draw2d(
                    framebuffer,
                    camera,
                    &draw2d::Segment::new(
                        Segment(
                            vec2(-inf * 2.0, (width.as_f32() - outline_width) / 2.0),
                            vec2(inf * 2.0, (width.as_f32() - outline_width) / 2.0),
                        ),
                        outline_width,
                        color,
                    )
                    .rotate(collider.rotation.map(Coord::as_f32))
                    .translate(collider.position.as_f32()),
                );
                self.geng.draw2d().draw2d(
                    framebuffer,
                    camera,
                    &draw2d::Segment::new(
                        Segment(
                            vec2(-inf * 2.0, -(width.as_f32() - outline_width) / 2.0),
                            vec2(inf * 2.0, -(width.as_f32() - outline_width) / 2.0),
                        ),
                        outline_width,
                        color,
                    )
                    .rotate(collider.rotation.map(Coord::as_f32))
                    .translate(collider.position.as_f32()),
                );
            }
            Shape::Rectangle { width, height } => {
                let [a, b, c, d] = Aabb2::ZERO
                    .extend_symmetric(vec2(width.as_f32(), height.as_f32()) / 2.0)
                    .extend_uniform(-outline_width / 2.0)
                    .corners();
                let m = (a + b) / 2.0;
                self.geng.draw2d().draw2d(
                    framebuffer,
                    camera,
                    &draw2d::Chain::new(
                        Chain::new(vec![m, b, c, d, a, m]),
                        outline_width,
                        color,
                        1,
                    )
                    .rotate(collider.rotation.map(Coord::as_f32))
                    .translate(collider.position.as_f32()),
                );
            }
        }
    }

    pub fn draw_button(
        &self,
        button: &HoverButton,
        text: impl AsRef<str>,
        theme: &LevelTheme,
        camera: &Camera2d,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        let frame = |time: f32, scale: f32| -> MoveFrame {
            MoveFrame {
                lerp_time: Time::new(time),
                transform: Transform {
                    scale: Coord::new(scale),
                    ..default()
                },
            }
        };
        let movement = Movement {
            initial: frame(0.0, 2.25).transform,
            key_frames: vec![frame(0.5, 5.0), frame(0.25, 75.0)].into(),
            fade_in: Time::ZERO,
            fade_out: r32(0.2),
        };

        let t = button.hover_time.get_ratio();
        let scale = movement.get(t).scale;
        let collider = button
            .collider
            .transformed(Transform { scale, ..default() });
        self.draw_collider(&collider, theme.light, camera, framebuffer);

        if t.as_f32() < 0.5 {
            self.geng.default_font().draw(
                framebuffer,
                camera,
                text.as_ref(),
                vec2::splat(geng::TextAlign::CENTER),
                mat3::translate(collider.position.as_f32())
                    * mat3::scale_uniform(1.0)
                    * mat3::translate(vec2(0.0, -0.25)),
                theme.dark,
            );
        }
    }

    pub fn draw_checkbox(
        &self,
        widget: &CheckboxWidget,
        options: TextRenderOptions,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        if !widget.check.visible {
            return;
        }

        let camera = &geng::PixelPerfectCamera;
        let options = options.align(vec2(0.0, 0.5)); // TODO

        let checkbox = widget.check.position;
        if widget.checked {
            let checkbox = checkbox.extend_uniform(-options.size * 0.05);
            for (a, b) in [
                (checkbox.bottom_left(), checkbox.top_right()),
                (checkbox.top_left(), checkbox.bottom_right()),
            ] {
                self.geng.draw2d().draw2d(
                    framebuffer,
                    camera,
                    &draw2d::Segment::new(Segment(a, b), options.size * 0.07, options.color),
                );
            }
        }
        self.draw_outline(
            &Collider::aabb(checkbox.map(r32)),
            options.size * 0.1,
            options.color,
            camera,
            framebuffer,
        );
        self.draw_text_widget(&widget.text, options, framebuffer);
    }

    pub fn draw_text_widget(
        &self,
        widget: &TextWidget,
        options: TextRenderOptions,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        if !widget.state.visible {
            return;
        }

        self.draw_text(
            &widget.text,
            geng_utils::layout::aabb_pos(widget.state.position, options.align),
            options.size(widget.font_size),
            &geng::PixelPerfectCamera,
            framebuffer,
        );
    }

    pub fn draw_dashed_chain(
        &self,
        chain: &Chain<f32>,
        options: &DashRenderOptions,
        camera: &impl geng::AbstractCamera2d,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        let mut dash_full_left = 0.0;
        for segment in chain.segments() {
            dash_full_left =
                self.draw_dashed_segment(segment, options, dash_full_left, camera, framebuffer);
        }
    }

    /// Draws a dashed segment.
    /// Returns the unrendered length of the last dash.
    pub fn draw_dashed_segment(
        &self,
        mut segment: Segment<f32>,
        options: &DashRenderOptions,
        dash_full_left: f32,
        camera: &impl geng::AbstractCamera2d,
        framebuffer: &mut ugli::Framebuffer,
    ) -> f32 {
        let delta = segment.1 - segment.0;
        let delta_len = delta.len();
        let direction_norm = if delta.len().approx_eq(&0.0) {
            return dash_full_left;
        } else {
            delta / delta_len
        };

        if dash_full_left > 0.0 {
            // Finish drawing the previous dash and offset current segment
            let dash_full_length = dash_full_left.min(delta_len);
            let dash_length = dash_full_left - options.space_length;
            if dash_length > 0.0 {
                // Finish dash
                let dash_length = dash_length.min(dash_full_length);
                let dash_end = segment.0 + direction_norm * dash_length;
                self.geng.draw2d().draw2d(
                    framebuffer,
                    camera,
                    &draw2d::Chain::new(
                        Chain::new(vec![segment.0, dash_end]),
                        options.width,
                        options.color,
                        1,
                    ),
                );
            }

            // Finish space
            let dash_left = dash_full_left - dash_full_length;
            if dash_left > 0.0 {
                return dash_left;
            }

            // Offset
            segment.0 += dash_full_length * direction_norm
        }

        let full_length = options.dash_length + options.space_length;

        // Recalculate delta
        let delta_len = (segment.1 - segment.0).len();
        let dashes = (delta_len / full_length).floor() as usize;
        for i in 0..dashes {
            let dash_start = segment.0 + direction_norm * i as f32 * full_length;
            self.geng.draw2d().draw2d(
                framebuffer,
                camera,
                &draw2d::Chain::new(
                    Chain::new(vec![
                        dash_start,
                        dash_start + direction_norm * options.dash_length,
                    ]),
                    options.width,
                    options.color,
                    1,
                ),
            );
        }

        let last_start = segment.0 + direction_norm * dashes as f32 * full_length;
        let last_len = (segment.1 - last_start).len();
        let dash_len = last_len.min(options.dash_length);
        let last_end = last_start + direction_norm * dash_len;
        self.geng.draw2d().draw2d(
            framebuffer,
            camera,
            &draw2d::Chain::new(
                Chain::new(vec![last_start, last_end]),
                options.width,
                options.color,
                1,
            ),
        );
        full_length - last_len
    }
}
