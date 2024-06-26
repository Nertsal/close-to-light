use super::*;

use crate::ui::layout::AreaOps;

use ctl_client::core::types::Name;

pub struct InputWidget {
    pub state: WidgetState,
    pub name: TextWidget,
    pub text: TextWidget,
    pub edit_id: Option<usize>,
    pub hide_input: bool,
    pub raw: String,
    pub editing: bool,
}

impl InputWidget {
    pub fn new(name: impl Into<Name>, hide_input: bool) -> Self {
        Self {
            state: WidgetState::new(),
            name: TextWidget::new(name).aligned(vec2(0.0, 0.5)),
            text: TextWidget::new("").aligned(vec2(1.0, 0.5)),
            edit_id: None,
            hide_input,
            raw: String::new(),
            editing: false,
        }
    }

    pub fn sync(&mut self, text: &str, context: &mut UiContext) {
        if self.raw == text {
            return;
        }

        text.clone_into(&mut self.raw);
        self.text.text = self.raw.clone().into();
        if self
            .edit_id
            .map_or(false, |id| context.text_edit.is_active(id))
        {
            self.edit_id = Some(context.text_edit.edit(&self.raw));
        }
    }
}

impl Widget for InputWidget {
    fn state_mut(&mut self) -> &mut WidgetState {
        &mut self.state
    }

    fn update(&mut self, position: Aabb2<f32>, context: &mut UiContext) {
        self.state.update(position, context);

        if self.state.clicked {
            self.edit_id = Some(context.text_edit.edit(&self.text.text));
        }

        self.editing = if self
            .edit_id
            .map_or(false, |id| context.text_edit.is_active(id))
        {
            self.raw.clone_from(&context.text_edit.text);
            self.text.text = if self.hide_input {
                "*".repeat(context.text_edit.text.len()).into()
            } else {
                context.text_edit.text.clone().into()
            };
            true
        } else {
            false
        };

        let mut main = position;

        let name_width = if self.name.text.is_empty() {
            self.text.align(vec2(0.5, 0.5));
            0.0
        } else {
            self.text.align(vec2(1.0, 0.5));
            (context.layout_size * 5.0).min(main.width() / 2.0)
        };
        let name = main.cut_left(name_width);
        self.name.update(name, context);
        self.text.update(main, context);
    }
}
