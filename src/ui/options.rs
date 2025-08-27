use super::*;

use crate::{
    menu::MenuState,
    prelude::{Assets, Theme, VolumeOptions},
    ui::layout::AreaOps,
};

use ctl_assets::GraphicsOptions;
use ctl_core::types::Name;
use ctl_util::{SecondOrderDynamics, SecondOrderState};
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
            state: WidgetState::new().with_sfx(WidgetSfxConfig::hover()),
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
    /// Downward scroll.
    pub scroll: SecondOrderState<f32>,
    pub profile: ProfileWidget,
    pub separator: WidgetState,
    pub volume: VolumeWidget,
    pub palette: PaletteChooseWidget,
    pub graphics: GraphicsWidget,
}

impl OptionsWidget {
    pub fn new(assets: &Rc<Assets>, palettes: Vec<PaletteWidget>) -> Self {
        Self {
            state: WidgetState::new(),
            window: UiWindow::new((), 0.3),
            scroll: SecondOrderState::new(SecondOrderDynamics::new(5.0, 2.0, 0.0, 0.0)),
            profile: ProfileWidget::new(assets),
            separator: WidgetState::new(),
            volume: VolumeWidget::new(),
            palette: PaletteChooseWidget::new(palettes),
            graphics: GraphicsWidget::new(),
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

        // Scroll
        if self.state.hovered {
            let scroll_speed = 2.0;
            self.scroll.target += context.cursor.scroll * scroll_speed;
        }
        self.scroll.update(context.delta_time);

        let mut main = position
            .extend_symmetric(vec2(-1.0, -1.0) * context.layout_size)
            .extend_down(100.0 * context.layout_size) // Technically infinite because we can scroll
            .translate(vec2(0.0, -self.scroll.current));

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

        let palette = main.clone().cut_top(6.0 * context.layout_size);
        self.palette.update(palette, context, &mut options.theme);
        main.cut_top(self.palette.state.position.height());

        let graphics = main.clone().cut_top(5.0 * context.font_size);
        self.graphics
            .update(graphics, context, &mut options.graphics);
        main.cut_top(self.graphics.state.position.height());

        state.context.set_options(options);

        // Limit scroll to the contents
        let overflow_up = self.scroll.target;
        let height = position.max.y - main.max.y + context.font_size * 2.0 - self.scroll.current;
        let max_scroll = (height - position.height()).max(0.0);
        let overflow_down = -max_scroll - self.scroll.target;
        let overflow = if overflow_up > 0.0 {
            overflow_up
        } else if overflow_down > 0.0 {
            -overflow_down
        } else {
            0.0
        };
        self.scroll.target -= overflow * (context.delta_time / 0.1).min(1.0);
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
            state: WidgetState::new().with_sfx(WidgetSfxConfig::hover()),
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

pub struct GraphicsWidget {
    pub state: WidgetState,
    pub title: TextWidget,
    pub crt: ToggleWidget,
    pub crt_scanlines: SliderWidget,
}

impl GraphicsWidget {
    pub fn new() -> Self {
        Self {
            state: WidgetState::new(),
            title: TextWidget::new("Graphics"),
            crt: ToggleWidget::new("CRT Shader"),
            crt_scanlines: SliderWidget::new("CRT Scanlines").with_display_precision(0),
        }
    }
}

impl StatefulWidget for GraphicsWidget {
    type State<'a> = GraphicsOptions;

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

        let title = main.cut_top(context.font_size * 1.2);
        self.title.align(vec2(0.5, 0.5));
        self.title.update(title, context);

        let row = Aabb2::point(main.top_left())
            .extend_right(main.width())
            .extend_down(context.font_size * 1.1);
        let rows = row.stack(vec2(0.0, -row.height() - context.layout_size * 0.1), 2);
        let min_y = rows.last().unwrap().min.y;

        self.crt.checked = state.crt.enabled;
        self.crt.update(rows[0], context);
        if self.crt.state.mouse_left.clicked {
            state.crt.enabled = !state.crt.enabled;
        }

        if state.crt.enabled {
            self.crt_scanlines.state.show();
            let mut value = Bounded::new(state.crt.scanlines * 100.0, 0.0..=100.0);
            self.crt_scanlines.update(rows[1], context, &mut value);
            state.crt.scanlines = value.value() / 100.0;
        } else {
            self.crt_scanlines.state.hide();
        }

        let mut position = position;
        position.min.y = min_y;
        self.state.update(position, context);
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
