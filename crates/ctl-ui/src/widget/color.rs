use super::*;

use crate::layout::AreaOps;

use ctl_assets::{Theme, ThemeColor};
use ctl_core::types::Name;

pub struct PaletteChooseWidget {
    pub state: WidgetState,
    pub title: TextWidget,
    pub palettes: Vec<PaletteWidget>,
}

impl PaletteChooseWidget {
    pub fn new(options: Vec<PaletteWidget>) -> Self {
        Self {
            state: WidgetState::new(),
            title: TextWidget::new("Palette"),
            palettes: options,
        }
    }
}

impl StatefulWidget for PaletteChooseWidget {
    type State<'a> = Theme;

    fn state_mut(&mut self) -> &mut WidgetState {
        &mut self.state
    }

    fn update(
        &mut self,
        position: Aabb2<f32>,
        context: &mut UiContext,
        state: &mut Self::State<'_>,
    ) {
        let mut main = position;

        let title = main.cut_top(context.font_size * 1.5);
        self.title.update(title, context);

        let row = Aabb2::point(main.top_left())
            .extend_right(main.width())
            .extend_down(context.font_size * 1.2);
        let rows = row.stack(
            vec2(0.0, -row.height() - context.layout_size * 0.1),
            self.palettes.len(),
        );
        let min_y = rows.last().unwrap().min.y;
        for (palette, pos) in self.palettes.iter_mut().zip(rows) {
            palette.update(pos, context, state);
            if palette.state.mouse_left.clicked {
                *state = palette.palette;
            }
        }

        let mut position = position;
        position.min.y = min_y;
        self.state.update(position, context);
    }
}

pub struct PaletteWidget {
    pub state: WidgetState,
    pub visual: WidgetState,
    pub name: TextWidget,
    pub palette: Theme,
}

impl PaletteWidget {
    pub fn new(name: impl Into<Name>, palette: Theme) -> Self {
        Self {
            state: WidgetState::new().with_sfx(WidgetSfxConfig::hover_left()),
            visual: WidgetState::new(),
            name: TextWidget::new(name),
            palette,
        }
    }
}

impl StatefulWidget for PaletteWidget {
    type State<'a> = Theme;

    fn state_mut(&mut self) -> &mut WidgetState {
        &mut self.state
    }

    fn update(
        &mut self,
        position: Aabb2<f32>,
        context: &mut UiContext,
        state: &mut Self::State<'_>,
    ) {
        self.state.update(position, context);
        if self.state.mouse_left.clicked {
            *state = self.palette;
        }

        let main = position;

        let mut name = main;
        let visual = name.split_left(0.5);

        let height = main.height() * 0.5;
        let visual = visual.extend_left(height * 4.0 - visual.width());
        let visual = visual.extend_symmetric(vec2(0.0, height - visual.height()) / 2.0);
        self.visual.update(visual, context);

        let name = name.extend_left(-context.font_size * 0.2);
        self.name.align(vec2(0.0, 0.5));
        self.name.update(name, context);
    }
}

/// Select one of the palette colors.
pub struct ColorSelectWidget {
    pub state: WidgetState,
    pub title: TextWidget,
    pub colors: Vec<(WidgetState, ThemeColor)>,
    pub selected_color: ThemeColor,
}

impl ColorSelectWidget {
    pub fn new(title: impl Into<Name>, options: impl IntoIterator<Item = ThemeColor>) -> Self {
        let colors: Vec<_> = options
            .into_iter()
            .map(|color| {
                (
                    WidgetState::new().with_sfx(WidgetSfxConfig::hover_left()),
                    color,
                )
            })
            .collect();
        Self {
            state: WidgetState::new(),
            title: TextWidget::new(title).aligned(vec2(0.0, 0.5)),
            selected_color: colors.first().map_or(ThemeColor::Light, |(_, c)| *c),
            colors,
        }
    }
}

impl StatefulWidget for ColorSelectWidget {
    type State<'a> = ThemeColor;

    fn state_mut(&mut self) -> &mut WidgetState {
        &mut self.state
    }

    fn update(
        &mut self,
        position: Aabb2<f32>,
        context: &mut UiContext,
        state: &mut Self::State<'_>,
    ) {
        let mut main = position;

        let title = main.cut_left(context.font_size * 3.5);
        self.title.update(title, context);

        let alignment = 1.0;
        let color = main.align_aabb(vec2::splat(main.height() * 0.8), vec2(alignment, 0.5));
        let colors = color.stack_aligned(
            vec2(color.width() + context.layout_size * 0.25, 0.0),
            self.colors.len(),
            vec2(alignment, 0.5),
        );
        let min_y = colors.last().unwrap().min.y;
        for ((widget, color), pos) in self.colors.iter_mut().zip(colors) {
            widget.update(pos, context);
            if widget.mouse_left.clicked {
                *state = *color;
            }
        }

        let mut position = position;
        position.min.y = min_y;
        self.state.update(position, context);
        self.selected_color = *state;
    }
}
