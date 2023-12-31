use super::*;

use crate::{prelude::*, ui::layout};

#[derive(Debug)]
pub struct LightWidget {
    pub state: WidgetState,
    pub light: LightSerde,
}

#[derive(Debug)]
pub struct LightStateWidget {
    pub light: LightWidget,
    pub danger: CheckboxWidget,
    pub scale: TextWidget,
    pub fade_in: TextWidget,
    pub fade_out: TextWidget,
}

impl LightWidget {
    pub fn new() -> Self {
        Self::new_shape(Shape::Circle { radius: r32(1.0) })
    }

    pub fn new_shape(shape: Shape) -> Self {
        Self {
            state: WidgetState::default(),
            light: LightSerde {
                danger: false,
                shape,
                movement: Movement::default(),
            },
        }
    }
}

impl Widget for LightWidget {
    fn update(&mut self, position: Aabb2<f32>, context: &UiContext) {
        let size = position.width().min(position.height());
        let position = Aabb2::point(position.center()).extend_uniform(size / 2.0);

        self.state.update(position, context);
    }

    fn walk_states_mut(&mut self, f: &dyn Fn(&mut WidgetState)) {
        self.state.walk_states_mut(f);
    }
}

impl LightStateWidget {
    pub fn new() -> Self {
        Self {
            light: LightWidget::new(),
            danger: CheckboxWidget::new("Danger"),
            scale: TextWidget::new("Scale"),
            fade_in: TextWidget::new("Fade in time"),
            fade_out: TextWidget::new("Fade out time"),
        }
    }
}

impl Widget for LightStateWidget {
    fn update(&mut self, position: Aabb2<f32>, context: &UiContext) {
        let (light_position, position) = layout::split_top_down(position, 0.5);
        self.light.update(light_position, context);

        let props: [&mut dyn Widget; 4] = [
            &mut self.danger,
            &mut self.scale,
            &mut self.fade_in,
            &mut self.fade_out,
        ];
        for (pos, prop) in layout::split_rows(position, props.len())
            .into_iter()
            .zip(props)
        {
            prop.update(pos, context);
        }
    }

    fn walk_states_mut(&mut self, f: &dyn Fn(&mut WidgetState)) {
        self.light.walk_states_mut(f);
        self.danger.walk_states_mut(f);
        self.scale.walk_states_mut(f);
        self.fade_in.walk_states_mut(f);
        self.fade_out.walk_states_mut(f);
    }
}
