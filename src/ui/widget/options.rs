use super::*;

use crate::{
    menu::MenuState,
    prelude::{Assets, Theme, VolumeOptions},
    ui::layout::AreaOps,
};

use ctl_client::core::types::Name;
use geng_utils::bounded::Bounded;

pub struct OptionsButtonWidget {
    pub state: WidgetState,
    pub open_time: Bounded<f32>,
    pub button: IconWidget,
    pub options: OptionsWidget,
}

impl OptionsButtonWidget {
    pub fn new(assets: &Rc<Assets>, time: f32) -> Self {
        Self {
            state: WidgetState::new(),
            open_time: Bounded::new_zero(time),
            button: IconWidget::new(assets.atlas.settings()),
            options: OptionsWidget::new(
                assets,
                vec![
                    // TODO: custom
                    PaletteWidget::new("Classic", Theme::classic()),
                    PaletteWidget::new("Stargazer", Theme::peach_mint()),
                    PaletteWidget::new("Corruption", Theme::corruption()),
                    PaletteWidget::new("Linksider", Theme::linksider()),
                ],
            ),
        }
    }
}

impl StatefulWidget for OptionsButtonWidget {
    type State<'a> = MenuState;

    fn state_mut(&mut self) -> &mut WidgetState {
        &mut self.state
    }

    fn update(
        &mut self,
        position: Aabb2<f32>,
        context: &mut UiContext,
        state: &mut Self::State<'_>,
    ) {
        let button_size = vec2::splat(1.0 * context.font_size);
        let button = position.align_aabb(button_size, vec2(1.0, 1.0));
        self.state.update(button, context);
        self.button.update(button, context);

        if self.button.state.hovered || self.options.state.hovered {
            self.open_time.change(context.delta_time);
            self.options.show();
        } else {
            self.open_time.change(-context.delta_time);
            if self.open_time.is_min() {
                self.options.hide();
            }
        }

        if self.options.state.visible {
            let max_size = vec2(15.0, 27.0) * context.layout_size;
            let min_size = button_size;
            let options_size = min_size + (max_size - min_size) * self.open_time.get_ratio();
            let options = position.align_aabb(options_size, vec2(1.0, 1.0));
            self.options.update(options, context, state);
        }
    }
}

pub struct OptionsWidget {
    pub state: WidgetState,
    pub window: UiWindow<()>,
    pub profile: ProfileWidget,
    pub separator: WidgetState,
    pub volume: VolumeWidget,
    pub palette: PaletteChooseWidget,
}

impl OptionsWidget {
    pub fn new(assets: &Rc<Assets>, palettes: Vec<PaletteWidget>) -> Self {
        Self {
            state: WidgetState::new(),
            window: UiWindow::new((), 0.3),
            profile: ProfileWidget::new(assets),
            separator: WidgetState::new(),
            volume: VolumeWidget::new(),
            palette: PaletteChooseWidget::new(palettes),
        }
    }
}

impl StatefulWidget for OptionsWidget {
    type State<'a> = MenuState;

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
        self.window.update(context.delta_time);

        let mut main = position.extend_symmetric(vec2(-1.0, -1.0) * context.layout_size);

        let profile = main.cut_top(3.0 * context.font_size);
        self.profile
            .update(profile, context, &mut state.leaderboard);

        let separator = main.cut_top(context.layout_size);
        let separator = separator.align_aabb(
            vec2(separator.width() * 0.9, context.layout_size * 0.1),
            vec2(0.5, 0.5),
        );
        self.separator.update(separator, context);

        let mut options = state.context.get_options();

        let volume = main.cut_top(5.0 * context.layout_size);
        self.volume.update(volume, context, &mut options.volume);
        let palette = main.cut_top(6.0 * context.layout_size);
        self.palette.update(palette, context, &mut options.theme);

        state.context.set_options(options);
    }
}

pub struct VolumeWidget {
    pub state: WidgetState,
    pub title: TextWidget,
    pub master: SliderWidget,
}

impl VolumeWidget {
    pub fn new() -> Self {
        Self {
            state: WidgetState::new(),
            title: TextWidget::new("Volume"),
            master: SliderWidget::new("").with_display_precision(0),
        }
    }
}

impl StatefulWidget for VolumeWidget {
    type State<'a> = VolumeOptions;

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
        let mut main = position;

        let title = main.cut_top(context.font_size * 1.2);
        self.title.align(vec2(0.5, 0.5));
        self.title.update(title, context);

        let row = Aabb2::point(main.top_left())
            .extend_right(main.width())
            .extend_down(context.font_size * 1.1);
        let rows = row.stack(vec2(0.0, -row.height() - context.layout_size * 0.1), 1);

        self.master.update(rows[0], context, &mut state.master);
    }
}

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
        self.state.update(position, context);
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
        for (palette, pos) in self.palettes.iter_mut().zip(rows) {
            palette.update(pos, context, state);
            if palette.state.clicked {
                *state = palette.palette;
            }
        }
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
            state: WidgetState::new(),
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
        if self.state.clicked {
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
