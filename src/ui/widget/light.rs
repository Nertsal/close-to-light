use super::*;

use crate::prelude::*;

#[derive(Debug)]
pub struct LightWidget {
    pub state: WidgetState,
    pub light: LightSerde,
}

impl LightWidget {
    pub fn new() -> Self {
        Self {
            state: WidgetState::default(),
            light: LightSerde {
                position: vec2::ZERO,
                danger: false,
                rotation: Coord::ZERO,
                shape: Shape::Circle { radius: r32(1.0) },
                movement: Movement::default(),
            },
        }
    }
}

impl Widget for LightWidget {
    fn update(&mut self, position: Aabb2<f32>, cursor_position: vec2<f32>, cursor_down: bool) {
        self.state.update(position, cursor_position, cursor_down);
    }

    fn walk_states_mut(&mut self, f: &dyn Fn(&mut WidgetState)) {
        self.state.walk_states_mut(f);
    }
}
