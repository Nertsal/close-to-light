use super::*;

use crate::ui::layout;

#[derive(Default)]
pub struct GroupWidget {
    pub state: WidgetState,
    pub logo: WidgetState,
    pub name: TextWidget,
    // pub author: TextWidget,
}

impl Widget for GroupWidget {
    fn update(&mut self, position: Aabb2<f32>, context: &UiContext) {
        self.state.update(position, context);

        let logo_size = position.height();
        let (logo, position) = layout::cut_left_right(position, logo_size);
        self.logo.update(logo, context);
        self.name.update(position, context);
    }

    fn walk_states_mut(&mut self, f: &dyn Fn(&mut WidgetState)) {
        self.state.walk_states_mut(f);
        self.logo.walk_states_mut(f);
        self.name.walk_states_mut(f);
        // self.author.walk_states_mut(f);
    }
}
