use super::*;

use crate::render::util::TextRenderOptions;

use ctl_client::core::types::Name;

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
