use super::*;

use crate::{
    menu::GameOptions,
    prelude::{Assets, Theme, VolumeOptions},
    ui::layout::AreaOps,
};

use ctl_assets::{CursorOptions, GameplayOptions, GraphicsOptions};
use ctl_ui::util::ScrollState;
use geng_utils::bounded::Bounded;

const RANGE_VOLUME: RangeInclusive<f32> = 0.0..=100.0;
const RANGE_MUSIC_OFFSET: RangeInclusive<f32> = -100.0..=100.0;
const RANGE_BLUE: RangeInclusive<f32> = 50.0..=100.0;
const RANGE_SATURATION: RangeInclusive<f32> = 0.0..=100.0;
const RANGE_INNER_RADIUS: RangeInclusive<f32> = 0.1..=0.5;
const RANGE_OUTER_RADIUS: RangeInclusive<f32> = 0.05..=0.5;

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
                    PaletteWidget::new("Frostlight", Theme::frostlight()),
                    PaletteWidget::new("Stargazer", Theme::stargazer()),
                    PaletteWidget::new("Corruption", Theme::corruption()),
                    PaletteWidget::new("Linksider", Theme::linksider()),
                ],
            ),
        }
    }
}

impl StatefulWidget for OptionsButtonWidget {
    type State<'a> = GameOptions;

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

        if self.button.state.hovered
            || self.options.state.hovered
            || self.options.state.mouse_left.pressed.is_some()
        {
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
    pub drag_state: WidgetState,
    pub window: UiWindow<()>,
    content_size: f32,
    /// Downward scroll.
    pub scroll: ScrollState,
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
            drag_state: WidgetState::new(),
            window: UiWindow::new((), 0.3),
            content_size: 1.0,
            scroll: ScrollState::new(),
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
    type State<'a> = GameOptions;

    fn state_mut(&mut self) -> &mut WidgetState {
        &mut self.state
    }

    fn update(
        &mut self,
        mut position: Aabb2<f32>,
        context: &mut UiContext,
        state: &mut Self::State<'_>,
    ) {
        self.window.update(context.delta_time);
        self.state.update(position, context);

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
            self.scroll.state.target = scroll;
            self.scroll.state.update(context.delta_time);
        } else {
            // Scroll drag
            self.scroll.drag(context, &self.drag_state);
        }

        let handle_t = -self.scroll.state.current / (self.content_size - position.height());
        let handle = scrollbar.with_height(handle_height, 1.0 - handle_t.clamp(0.0, 1.0));
        self.scrollbar_handle.update(handle, context);

        let mut main = position
            .extend_symmetric(vec2(-1.5, -1.0) * context.layout_size)
            .extend_down(100.0 * context.layout_size) // Technically infinite because we can scroll
            .translate(vec2(0.0, -self.scroll.state.current));
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
        let cursor_size =
            if self.cursor.inner_radius.state.hovered || self.cursor.outer_radius.state.hovered {
                r32(0.5)
            } else {
                r32(0.1)
            };
        state.player_size.target = cursor_size;

        state.context.set_options(options);

        // Limit scroll to the contents
        self.content_size = main_top - main.max.y + context.font_size * 2.0;
        self.scroll
            .overflow(context.delta_time, self.content_size, position.height());

        self.drag_state.update(position, context);
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
            master: SliderWidget::new("").with_precision(0),
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
            .update_value(rows[0], context, &mut state.master, RANGE_VOLUME);
    }
}

pub struct GraphicsWidget {
    pub state: WidgetState,
    pub title: TextWidget,
    pub fullscreen: ToggleWidget,
    pub crt: ToggleWidget,
    pub blue: SliderWidget,
    pub saturation: SliderWidget,
    pub telegraph_color: ColorSelectWidget,
    pub perfect_color: ColorSelectWidget,
}

impl GraphicsWidget {
    pub fn new() -> Self {
        Self {
            state: WidgetState::new(),
            title: TextWidget::new("Graphics"),
            fullscreen: ToggleWidget::new("Fullscreen"),
            crt: ToggleWidget::new("CRT Shader"),
            blue: SliderWidget::new("Blue light").with_precision(0),
            saturation: SliderWidget::new("Saturation").with_precision(0),
            telegraph_color: ColorSelectWidget::new(
                "Telegraph color",
                [ThemeColor::Light, ThemeColor::Highlight],
            ),
            perfect_color: ColorSelectWidget::new(
                "Perfect color",
                [ThemeColor::Light, ThemeColor::Highlight],
            ),
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
        let layout_size = context.layout_size;
        let mut next_row = || -> Aabb2<f32> {
            let row = current_row;
            current_row = current_row.translate(vec2(0.0, -row.height() - layout_size * 0.1));
            min_y = row.min.y;
            row
        };

        let window = context.context.geng.window();
        self.fullscreen.checked = window.is_fullscreen();
        self.fullscreen.update(next_row(), context);
        if self.fullscreen.state.mouse_left.clicked {
            window.toggle_fullscreen();
        }

        self.crt
            .update_state(next_row(), context, &mut state.crt.enabled);

        // TODO: fix dragging view while changing value (also for music offset)
        let mut blue = state.colors.blue * 100.0;
        self.blue
            .update_value(next_row(), context, &mut blue, RANGE_BLUE);
        state.colors.blue = blue / 100.0;

        // TODO: fix dragging view while changing value (also for music offset)
        let mut saturation = state.colors.saturation * 100.0;
        self.saturation
            .update_value(next_row(), context, &mut saturation, RANGE_SATURATION);
        state.colors.saturation = saturation / 100.0;

        self.telegraph_color
            .update(next_row(), context, &mut state.lights.telegraph_color);
        self.perfect_color
            .update(next_row(), context, &mut state.lights.perfect_color);

        let mut position = position;
        position.min.y = min_y;
        self.state.update(position, context);
    }
}

pub struct CursorWidget {
    pub state: WidgetState,
    pub title: TextWidget,
    pub show_trail: ToggleWidget,
    pub inner_radius: SliderWidget,
    pub show_perfect_radius: ToggleWidget,
    pub outer_radius: SliderWidget,
    pub outer_color: ColorSelectWidget,
    pub show_rhythm_circles: ToggleWidget,
    pub show_rhythm_only_miss: ToggleWidget,
}

impl CursorWidget {
    pub fn new() -> Self {
        Self {
            state: WidgetState::new(),
            title: TextWidget::new("Cursor"),
            show_trail: ToggleWidget::new("Show trail"),
            inner_radius: SliderWidget::new("Trail size").with_precision(2),
            show_perfect_radius: ToggleWidget::new("Show outline"),
            outer_radius: SliderWidget::new("Outline width").with_precision(2),
            outer_color: ColorSelectWidget::new(
                "Outline color",
                [
                    // TODO: currently does not render properly in the menu because of transparency
                    // ThemeColor::Dark,
                    ThemeColor::Light,
                    ThemeColor::Highlight,
                    ThemeColor::Danger,
                ],
            ),
            show_rhythm_circles: ToggleWidget::new("Rhythm circles"),
            show_rhythm_only_miss: ToggleWidget::new("Show only misses"),
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
        let layout_size = context.layout_size;
        let mut next_row = || -> Aabb2<f32> {
            let row = current_row;
            current_row = current_row.translate(vec2(0.0, -row.height() - layout_size * 0.1));
            min_y = row.min.y;
            row
        };

        self.show_trail
            .update_state(next_row(), context, &mut state.show_trail);

        self.inner_radius.update_value(
            next_row(),
            context,
            &mut state.inner_radius,
            RANGE_INNER_RADIUS,
        );

        self.show_perfect_radius
            .update_state(next_row(), context, &mut state.show_perfect_radius);
        if state.show_perfect_radius {
            self.outer_radius.state.show();
            self.outer_radius.update_value(
                next_row(),
                context,
                &mut state.outer_radius,
                RANGE_OUTER_RADIUS,
            );

            self.outer_color.state.show();
            self.outer_color
                .update(next_row(), context, &mut state.outer_color);
        } else {
            self.outer_radius.state.hide();
            self.outer_color.state.hide();
        }

        self.show_rhythm_circles
            .update_state(next_row(), context, &mut state.show_rhythm_circles);
        if state.show_rhythm_circles {
            self.show_rhythm_only_miss.state.show();
            self.show_rhythm_only_miss.update_state(
                next_row(),
                context,
                &mut state.show_rhythm_only_miss,
            );
        } else {
            self.show_rhythm_only_miss.state.hide();
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
            music_offset: SliderWidget::new("Music offset").with_precision(0),
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

        self.music_offset.update_value(
            next_row(),
            context,
            &mut state.music_offset,
            RANGE_MUSIC_OFFSET,
        );

        let mut position = position;
        position.min.y = min_y;
        self.state.update(position, context);
    }
}
