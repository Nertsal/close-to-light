use crate::ui::layout::AreaOps;

use super::*;

pub struct InputWidget {
    pub state: WidgetState,
    pub name: TextWidget,
    pub text: TextWidget,
    pub edit_id: Option<usize>,
    pub hide_input: bool,
    pub raw: String,
}

impl InputWidget {
    pub fn new(name: impl Into<String>, hide_input: bool) -> Self {
        Self {
            state: WidgetState::new(),
            name: TextWidget::new(name).aligned(vec2(0.0, 0.5)),
            text: TextWidget::new("").aligned(vec2(1.0, 0.5)),
            edit_id: None,
            hide_input,
            raw: String::new(),
        }
    }
}

impl Widget for InputWidget {
    fn update(&mut self, position: Aabb2<f32>, context: &mut UiContext) {
        self.state.update(position, context);

        if self.state.clicked {
            self.edit_id = Some(context.text_edit.edit(&self.text.text));
        }

        let color = if self
            .edit_id
            .map_or(false, |id| context.text_edit.is_active(id))
        {
            self.text.text = if self.hide_input {
                "*".repeat(context.text_edit.text.len())
            } else {
                context.text_edit.text.clone()
            };
            context.theme.highlight
        } else {
            context.theme.light
        };

        let mut main = position;

        let old_color = context.theme.light;
        context.theme.light = color;

        let name_width = context.layout_size * 5.0;
        let name = main.cut_left(name_width);
        self.name.update(name, context);
        self.text.update(main, context);

        context.theme.light = old_color;
    }

    fn walk_states_mut(&mut self, f: &dyn Fn(&mut WidgetState)) {
        self.state.walk_states_mut(f);
        self.name.walk_states_mut(f);
        self.text.walk_states_mut(f);
    }
}
