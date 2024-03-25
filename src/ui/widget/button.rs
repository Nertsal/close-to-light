use super::*;

#[derive(Clone, Default)]
pub struct ButtonWidget {
    pub text: TextWidget,
    pub texture: Option<Rc<ugli::Texture>>,
}

impl ButtonWidget {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: TextWidget::new(text),
            texture: None,
        }
    }

    pub fn new_textured(text: impl Into<String>, texture: &Rc<ugli::Texture>) -> Self {
        Self {
            text: TextWidget::new(text),
            texture: Some(texture.clone()),
        }
    }
}

impl Widget for ButtonWidget {
    fn state_mut(&mut self) -> &mut WidgetState {
        &mut self.text.state
    }

    fn update(&mut self, position: Aabb2<f32>, context: &mut UiContext) {
        self.text.update(position, context);
    }
}
