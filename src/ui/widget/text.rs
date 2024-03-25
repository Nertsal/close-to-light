use super::*;

use crate::render::util::TextRenderOptions;

#[derive(Debug, Clone, Default)]
pub struct TextWidget {
    pub state: WidgetState,
    pub text: String,
    pub options: TextRenderOptions,
}

impl TextWidget {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            ..default()
        }
    }

    pub fn aligned(mut self, align: vec2<f32>) -> Self {
        self.align(align);
        self
    }

    pub fn align(&mut self, align: vec2<f32>) {
        self.options.align = align;
    }
}

impl Widget for TextWidget {
    fn state_mut(&mut self) -> &mut WidgetState {
        &mut self.state
    }

    fn update(&mut self, position: Aabb2<f32>, context: &mut UiContext) {
        self.state.update(position, context);
        self.options.update(context);
    }
}
