use super::*;

pub struct GeometryWidget {
    pub state: WidgetState,
    #[allow(clippy::type_complexity)]
    pub geometry: Box<dyn Fn(&WidgetState, &UiContext) -> Geometry>,
}

impl GeometryWidget {
    pub fn new(geometry: impl Fn(&WidgetState, &UiContext) -> Geometry + 'static) -> Self {
        Self {
            state: WidgetState::new(),
            geometry: Box::new(geometry),
        }
    }

    pub fn update(&mut self, position: Aabb2<f32>, context: &UiContext) {
        self.state.update(position, context);
    }
}

impl Widget for GeometryWidget {
    fn state_mut(&mut self) -> &mut WidgetState {
        &mut self.state
    }

    fn draw(&self, context: &UiContext) -> Geometry {
        (self.geometry)(&self.state, context)
    }
}
