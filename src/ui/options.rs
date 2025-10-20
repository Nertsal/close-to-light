use super::*;

use crate::{
    menu::MenuState,
    prelude::{Assets, Theme, VolumeOptions},
    ui::layout::AreaOps,
};

use ctl_assets::{CursorOptions, GameplayOptions, GraphicsOptions};
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
                    // TODO: custom palettes
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
    content_size: f32,
    /// Downward scroll.
    pub scroll: SecondOrderState<f32>,
    scroll_drag_from: f32,
    pub scrollbar: WidgetState,
    pub scrollbar_handle: WidgetState,
    pub profile: ProfileWidget,
    pub separator: WidgetState,
    pub volume: VolumeWidget,
    pub palette: PaletteChooseWidget,
    pub graphics: GraphicsWidget,
    pub cursor: CursorWidget,
    pub gameplay: GameplayWidget,
}

impl OptionsWidget {
    pub fn new(assets: &Rc<Assets>, palettes: Vec<PaletteWidget>) -> Self {
        Self {
            state: WidgetState::new(),
            window: UiWindow::new((), 0.3),
            content_size: 1.0,
            scroll: SecondOrderState::new(SecondOrderDynamics::new(5.0, 2.0, 0.0, 0.0)),
            scroll_drag_from: 0.0,
            scrollbar: WidgetState::new(),
            scrollbar_handle: WidgetState::new(),
            profile: ProfileWidget::new(assets),
            separator: WidgetState::new(),
            volume: VolumeWidget::new(),
            palette: PaletteChooseWidget::new(palettes),
            graphics: GraphicsWidget::new(),
            cursor: CursorWidget::new(),
            gameplay: GameplayWidget::new(),
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
        mut position: Aabb2<f32>,
        context: &mut UiContext,
        state: &mut Self::State<'_>,
    ) {
        self.state.update(position, context);
        self.window.update(context.delta_time);

        position.cut_right(context.layout_size * 0.25);
        let scrollbar = position
            .cut_right(context.layout_size * 0.75)
            .extend_symmetric(vec2(0.0, -context.layout_size));
        let handle_height = context.layout_size * 2.5;
        self.scrollbar.update(scrollbar, context);

        if self.scrollbar.mouse_left.pressed.is_some() {
            // Scroll bar
            let t = (context.cursor.position.y - self.scrollbar.position.min.y)
                / self.scrollbar.position.height();
            let max_scroll = self.content_size - position.height();
            let scroll = -max_scroll * (1.0 - t);
            self.scroll.target = scroll;
            self.scroll.update(context.delta_time);
        } else {
            // Scroll drag
            ctl_ui::util::scroll_drag(
                context,
                &self.state,
                &mut self.scroll,
                &mut self.scroll_drag_from,
            );
        }

        let handle_t = -self.scroll.current / (self.content_size - position.height());
        let handle = scrollbar.with_height(handle_height, 1.0 - handle_t.clamp(0.0, 1.0));
        self.scrollbar_handle.update(handle, context);

        let mut main = position
            .extend_symmetric(vec2(-1.5, -1.0) * context.layout_size)
            .extend_down(100.0 * context.layout_size) // Technically infinite because we can scroll
            .translate(vec2(0.0, -self.scroll.current));
        let main_top = main.max.y;

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
        let spacing = context.layout_size;

        let volume = main.cut_top(5.0 * context.layout_size);
        self.volume.update(volume, context, &mut options.volume);
        // main.cut_top(spacing);

        let palette = main.clone().cut_top(6.0 * context.layout_size);
        self.palette.update(palette, context, &mut options.theme);
        main.cut_top(self.palette.state.position.height());
        main.cut_top(spacing);

        let gameplay = main.clone().cut_top(6.0 * context.layout_size);
        self.gameplay
            .update(gameplay, context, &mut options.gameplay);
        main.cut_top(self.gameplay.state.position.height());
        main.cut_top(spacing);

        let graphics = main.clone().cut_top(5.0 * context.font_size);
        self.graphics
            .update(graphics, context, &mut options.graphics);
        main.cut_top(self.graphics.state.position.height());
        main.cut_top(spacing);

        let cursor = main.clone().cut_top(5.0 * context.font_size);
        self.cursor.update(cursor, context, &mut options.cursor);
        main.cut_top(self.graphics.state.position.height());
        main.cut_top(spacing);

        state.context.set_options(options);

        // Limit scroll to the contents
        self.content_size = main_top - main.max.y + context.font_size * 2.0;
        ctl_ui::util::overflow_scroll(
            context.delta_time,
            &mut self.scroll.target,
            self.content_size,
            position.height(),
        );
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

        self.master
            .update_value(rows[0], context, &mut state.master, 0.0..=100.0);
    }
}

pub struct GraphicsWidget {
    pub state: WidgetState,
    pub title: TextWidget,
    pub crt: ToggleWidget,
    pub crt_scanlines: SliderWidget,
    pub telegraph_color: ToggleWidget,
}

impl GraphicsWidget {
    pub fn new() -> Self {
        Self {
            state: WidgetState::new(),
            title: TextWidget::new("Graphics"),
            crt: ToggleWidget::new("CRT Shader"),
            crt_scanlines: SliderWidget::new("Scanlines").with_display_precision(0),
            telegraph_color: ToggleWidget::new("Telegraph highlight"),
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

        let context = &mut context.scale_font(0.8);

        let mut current_row = Aabb2::point(main.top_left())
            .extend_right(main.width())
            .extend_down(context.font_size * 1.1);
        let mut min_y = current_row.min.y;
        let mut next_row = || -> Aabb2<f32> {
            let row = current_row;
            current_row =
                current_row.translate(vec2(0.0, -row.height() - context.layout_size * 0.1));
            min_y = row.min.y;
            row
        };

        self.crt.checked = state.crt.enabled;
        self.crt.update(next_row(), context);
        if self.crt.state.mouse_left.clicked {
            state.crt.enabled = !state.crt.enabled;
        }

        if state.crt.enabled {
            self.crt_scanlines.state.show();
            let mut value = Bounded::new(state.crt.scanlines * 100.0, 0.0..=100.0);
            self.crt_scanlines.update(next_row(), context, &mut value);
            state.crt.scanlines = value.value() / 100.0;
        } else {
            self.crt_scanlines.state.hide();
        }

        self.telegraph_color.update(next_row(), context);
        if self.telegraph_color.state.mouse_left.clicked {
            state.lights.telegraph_color = if state.lights.telegraph_color == ThemeColor::Light {
                ThemeColor::Highlight
            } else {
                ThemeColor::Light
            };
        }
        self.telegraph_color.checked = state.lights.telegraph_color == ThemeColor::Highlight;

        let mut position = position;
        position.min.y = min_y;
        self.state.update(position, context);
    }
}

pub struct CursorWidget {
    pub state: WidgetState,
    pub title: TextWidget,
    pub show_perfect_radius: ToggleWidget,
    pub inner_radius: SliderWidget,
    pub outer_radius: SliderWidget,
}

impl CursorWidget {
    pub fn new() -> Self {
        Self {
            state: WidgetState::new(),
            title: TextWidget::new("Cursor"),
            show_perfect_radius: ToggleWidget::new("Show Outline"),
            inner_radius: SliderWidget::new("Size").with_display_precision(2),
            outer_radius: SliderWidget::new("Outline width").with_display_precision(2),
        }
    }
}

impl StatefulWidget for CursorWidget {
    type State<'a> = CursorOptions;

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

        let context = &mut context.scale_font(0.8);

        let mut current_row = Aabb2::point(main.top_left())
            .extend_right(main.width())
            .extend_down(context.font_size * 1.1);
        let mut min_y = current_row.min.y;
        let mut next_row = || -> Aabb2<f32> {
            let row = current_row;
            current_row =
                current_row.translate(vec2(0.0, -row.height() - context.layout_size * 0.1));
            min_y = row.min.y;
            row
        };

        self.show_perfect_radius.update(next_row(), context);
        if self.show_perfect_radius.state.mouse_left.clicked {
            state.show_perfect_radius = !state.show_perfect_radius;
        }
        self.show_perfect_radius.checked = state.show_perfect_radius;

        self.inner_radius
            .update_value(next_row(), context, &mut state.inner_radius, 0.1..=0.5);
        if state.show_perfect_radius {
            self.outer_radius.update_value(
                next_row(),
                context,
                &mut state.outer_radius,
                0.05..=0.5,
            );
        }

        let mut position = position;
        position.min.y = min_y;
        self.state.update(position, context);
    }
}

pub struct GameplayWidget {
    pub state: WidgetState,
    pub title: TextWidget,
    pub music_offset: SliderWidget,
}

impl GameplayWidget {
    pub fn new() -> Self {
        Self {
            state: WidgetState::new(),
            title: TextWidget::new("Gameplay"),
            music_offset: SliderWidget::new("Music offset").with_display_precision(0),
        }
    }
}

impl StatefulWidget for GameplayWidget {
    type State<'a> = GameplayOptions;

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

        let context = &mut context.scale_font(0.8);

        let mut current_row = Aabb2::point(main.top_left())
            .extend_right(main.width())
            .extend_down(context.font_size * 1.1);
        let mut min_y = current_row.min.y;
        let mut next_row = || -> Aabb2<f32> {
            let row = current_row;
            current_row =
                current_row.translate(vec2(0.0, -row.height() - context.layout_size * 0.1));
            min_y = row.min.y;
            row
        };

        self.music_offset
            .update_value(next_row(), context, &mut state.music_offset, -50.0..=50.0);

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
