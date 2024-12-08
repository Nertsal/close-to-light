use super::*;

use crate::prelude::ThemeColor;

#[derive(Clone)]
pub struct IconWidget {
    pub state: WidgetState,
    pub texture: Rc<ugli::Texture>,
    pub color: ThemeColor,
    pub background: Option<IconBackground>,
}

#[derive(Debug, Clone)]
pub struct IconBackground {
    pub color: ThemeColor,
    pub kind: IconBackgroundKind,
}

#[derive(Debug, Clone, Copy)]
pub enum IconBackgroundKind {
    NineSlice,
    Circle,
}

impl IconWidget {
    pub fn new(texture: &Rc<ugli::Texture>) -> Self {
        Self {
            state: WidgetState::new(),
            texture: texture.clone(),
            color: ThemeColor::Light,
            background: None,
        }
    }
}

impl WidgetOld for IconWidget {
    fn state_mut(&mut self) -> &mut WidgetState {
        &mut self.state
    }

    fn update(&mut self, position: Aabb2<f32>, context: &mut UiContext) {
        self.state.update(position, context);
    }
}
