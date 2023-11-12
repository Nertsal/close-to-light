use super::*;

use crate::render::util::TextRenderOptions;

#[derive(Debug, Default)]
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

    pub fn align(&mut self, align: vec2<f32>) {
        self.options.align = align;
    }
}

impl Widget for TextWidget {
    fn update(&mut self, position: Aabb2<f32>, context: &UiContext) {
        self.state.update(position, context);
        self.options.size = context.font_size;
    }

    fn walk_states_mut(&mut self, f: &dyn Fn(&mut WidgetState)) {
        self.state.walk_states_mut(f);
    }
}
