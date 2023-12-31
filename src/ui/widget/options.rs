use super::*;

use crate::{
    prelude::{Options, Theme, VolumeOptions},
    ui::layout,
};

pub struct OptionsWidget {
    pub state: WidgetState,
    pub volume: VolumeWidget,
    pub palette: PaletteChooseWidget,
}

impl OptionsWidget {
    pub fn new(options: Options, palettes: Vec<PaletteWidget>) -> Self {
        Self {
            state: WidgetState::new(),
            volume: VolumeWidget::new(options.volume),
            palette: PaletteChooseWidget::new(options.theme, palettes),
        }
    }

    pub fn set_options(&mut self, options: Options) {
        self.volume.set_options(options.volume);
        self.palette.set_palette(options.theme);
    }

    pub fn update_options(&self, options: &mut Options) {
        self.volume.update_options(&mut options.volume);
        self.palette.update_palette(&mut options.theme);
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
        self.palette.update(columns[1], context);
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

pub struct PaletteChooseWidget {
    pub state: WidgetState,
    pub title: TextWidget,
    pub selected: Theme,
    pub palettes: Vec<PaletteWidget>,
}

impl PaletteChooseWidget {
    pub fn new(palette: Theme, options: Vec<PaletteWidget>) -> Self {
        Self {
            state: WidgetState::new(),
            title: TextWidget::new("Palette"),
            selected: palette,
            palettes: options,
        }
    }

    pub fn set_palette(&mut self, palette: Theme) {
        self.selected = palette;
    }

    pub fn update_palette(&self, palette: &mut Theme) {
        *palette = self.selected;
    }
}

impl Widget for PaletteChooseWidget {
    fn update(&mut self, position: Aabb2<f32>, context: &UiContext) {
        self.state.update(position, context);
        let main = position;

        let (title, main) = layout::cut_top_down(main, context.font_size * 1.5);
        self.title.update(title, context);

        let row = Aabb2::point(main.top_left())
            .extend_right(main.width())
            .extend_down(context.font_size * 1.0);
        let rows = layout::stack(
            row,
            vec2(0.0, -row.height() - context.font_size * 0.5),
            self.palettes.len(),
        );
        for (palette, pos) in self.palettes.iter_mut().zip(rows) {
            palette.update(pos, context);
            if palette.state.clicked {
                self.selected = palette.palette;
            }
        }
    }

    fn walk_states_mut(&mut self, f: &dyn Fn(&mut WidgetState)) {
        self.state.walk_states_mut(f);
    }
}

pub struct PaletteWidget {
    pub state: WidgetState,
    pub visual: WidgetState,
    pub name: TextWidget,
    pub palette: Theme,
}

impl PaletteWidget {
    pub fn new(name: impl Into<String>, palette: Theme) -> Self {
        Self {
            state: WidgetState::new(),
            visual: WidgetState::new(),
            name: TextWidget::new(name),
            palette,
        }
    }
}

impl Widget for PaletteWidget {
    fn update(&mut self, position: Aabb2<f32>, context: &UiContext) {
        self.state.update(position, context);
        let main = position;

        let (visual, name) = layout::split_left_right(main, 0.5);

        let height = main.height() * 0.5;
        let visual = visual.extend_left(height * 3.0 - visual.width());
        let visual = visual.extend_symmetric(vec2(0.0, height - visual.height()) / 2.0);
        self.visual.update(visual, context);

        let name = name.extend_left(-context.font_size * 0.2);
        self.name.align(vec2(0.0, 0.5));
        self.name.update(name, context);
    }

    fn walk_states_mut(&mut self, f: &dyn Fn(&mut WidgetState)) {
        self.state.walk_states_mut(f);
        self.visual.walk_states_mut(f);
        self.name.walk_states_mut(f);
    }
}
