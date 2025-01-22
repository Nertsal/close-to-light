use super::*;

use crate::{prelude::ThemeColor, util::SubTexture};

#[derive(Clone)]
pub struct IconWidget {
    pub state: WidgetState,
    pub texture: SubTexture,
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
    pub fn new(texture: SubTexture) -> Self {
        Self {
            state: WidgetState::new(),
            texture: texture.clone(),
            color: ThemeColor::Light,
            background: None,
        }
    }

    pub fn update(&mut self, position: Aabb2<f32>, context: &UiContext) {
        self.state.update(position, context);
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

impl Widget for IconWidget {
    fn draw(&self, context: &UiContext) -> Geometry {
        let theme = context.theme();
        let mut geometry = context.geometry.texture_pp(
            self.state.position.center(),
            theme.get_color(self.color),
            0.5,
            &self.texture,
        );

        if let Some(bg) = &self.background {
            match bg.kind {
                IconBackgroundKind::NineSlice => {
                    let texture = //if width < 5.0 {
                        &context.context.assets.atlas.fill_thin();
                    // } else {
                    //     &self.assets.sprites.fill
                    // };
                    geometry.merge(context.geometry.nine_slice(
                        self.state.position,
                        theme.get_color(bg.color),
                        texture,
                    ));
                }
                IconBackgroundKind::Circle => {
                    geometry.merge(context.geometry.texture_pp(
                        self.state.position.center(),
                        theme.get_color(bg.color),
                        0.5,
                        &context.context.assets.atlas.circle(),
                    ));
                }
            }
        }

        geometry
    }
}
