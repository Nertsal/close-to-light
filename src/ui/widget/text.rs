use super::*;

use crate::render::util::TextRenderOptions;

use ctl_client::core::{prelude::Color, types::Name};

#[derive(Debug, Clone)]
pub struct TextWidget {
    pub state: WidgetState,
    pub text: Name,
    pub options: TextRenderOptions,
}

impl Default for TextWidget {
    fn default() -> Self {
        Self {
            state: default(),
            text: "<text>".into(),
            options: default(),
        }
    }
}

impl TextWidget {
    pub fn new(text: impl Into<Name>) -> Self {
        Self {
            text: text.into(),
            ..default()
        }
    }

    pub fn rotated(mut self, rotation: Angle) -> Self {
        self.options.rotation = rotation;
        self
    }

    pub fn aligned(mut self, align: vec2<f32>) -> Self {
        self.align(align);
        self
    }

    pub fn align(&mut self, align: vec2<f32>) {
        self.options.align = align;
    }

    pub fn update(&mut self, position: Aabb2<f32>, context: &UiContext) {
        self.state.update(position, context);
        self.options.update(context);
    }
}

impl WidgetOld for TextWidget {
    fn state_mut(&mut self) -> &mut WidgetState {
        &mut self.state
    }

    fn update(&mut self, position: Aabb2<f32>, context: &mut UiContext) {
        self.state.update(position, context);
        self.options.update(context);
    }
}

impl TextWidget {
    pub fn draw_colored(&self, context: &UiContext, color: Color) -> Geometry {
        let font = &context.context.assets.fonts.pixel;
        let measure = font
            .measure(&self.text, vec2::splat(geng::TextAlign::CENTER))
            .unwrap_or(Aabb2::ZERO.extend_positive(vec2(1.0, 1.0)));

        let size = self.state.position.size();
        let right = vec2(size.x, 0.0).rotate(self.options.rotation).x;
        let left = vec2(0.0, size.y).rotate(self.options.rotation).x;
        let width = if left.signum() != right.signum() {
            left.abs() + right.abs()
        } else {
            left.abs().max(right.abs())
        };

        let max_width = width * 0.9; // Leave some space TODO: move into a parameter or smth
        let max_size = max_width / measure.width() / 0.6; // Magic constant from the util renderer that scales everything by 0.6 idk why
        let size = self.options.size.min(max_size);

        let mut options = self.options;
        options.size = size;
        options.color = color;

        context.geometry.text(
            self.text.clone(),
            geng_utils::layout::aabb_pos(self.state.position, options.align),
            options,
        )
    }
}

impl Widget for TextWidget {
    fn draw(&self, context: &UiContext) -> Geometry {
        self.draw_colored(context, self.options.color)
    }
}
