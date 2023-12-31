use super::*;

use crate::{
    prelude::{Options, VolumeOptions},
    ui::layout,
};

pub struct OptionsWidget {
    pub state: WidgetState,
    pub volume: VolumeWidget,
}

impl OptionsWidget {
    pub fn new(options: Options) -> Self {
        Self {
            state: WidgetState::new(),
            volume: VolumeWidget::new(options.volume),
        }
    }

    pub fn set_options(&mut self, options: Options) {
        self.volume.set_options(options.volume);
    }

    pub fn update_options(&self, options: &mut Options) {
        self.volume.update_options(&mut options.volume);
    }
}

impl Widget for OptionsWidget {
    fn update(&mut self, position: Aabb2<f32>, context: &UiContext) {
        self.state.update(position, context);

        let main = position.extend_symmetric(vec2(-5.0, -3.0) * context.font_size);
        let column = Aabb2::point(main.top_left())
            .extend_right(context.font_size * 10.0)
            .extend_down(main.height());
        let columns = layout::stack(
            column,
            vec2(column.width() + context.font_size * 5.0, 0.0),
            2,
        );

        self.volume.update(columns[0], context);
    }

    fn walk_states_mut(&mut self, f: &dyn Fn(&mut WidgetState)) {
        self.state.walk_states_mut(f);
    }
}

pub struct VolumeWidget {
    pub state: WidgetState,
    pub options: VolumeOptions,
    pub title: TextWidget,
    pub master: SliderWidget,
}

impl VolumeWidget {
    pub fn new(options: VolumeOptions) -> Self {
        Self {
            state: WidgetState::new(),
            title: TextWidget::new("Volume"),
            master: SliderWidget::new("", options.master),
            options,
        }
    }

    pub fn set_options(&mut self, options: VolumeOptions) {
        self.master.update_value(options.master, 2);
        self.options = options;
    }

    pub fn update_options(&self, options: &mut VolumeOptions) {
        options.master.set(self.master.bounds.value());
    }
}

impl Widget for VolumeWidget {
    fn update(&mut self, position: Aabb2<f32>, context: &UiContext) {
        self.state.update(position, context);
        let main = position;

        let (title, main) = layout::cut_top_down(main, context.font_size * 1.2);
        self.title.align(vec2(0.5, 0.5));
        self.title.update(title, context);

        let row = Aabb2::point(main.top_left())
            .extend_right(main.width())
            .extend_down(context.font_size * 1.0);
        let rows = layout::stack(row, vec2(0.0, -row.height()), 1);

        self.master.update(rows[0], context);
    }

    fn walk_states_mut(&mut self, f: &dyn Fn(&mut WidgetState)) {
        self.state.walk_states_mut(f);
        self.title.walk_states_mut(f);
        self.master.walk_states_mut(f);
    }
}
