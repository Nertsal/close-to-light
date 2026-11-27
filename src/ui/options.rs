use super::*;

use crate::{
    menu::GameOptions,
    prelude::{Assets, Theme, VolumeOptions},
    ui::layout::AreaOps,
};

use ctl_assets::{CursorOptions, GameplayOptions, GraphicsOptions};
use ctl_render_core::SubTexture;
use ctl_ui::util::ScrollState;
use geng_utils::bounded::Bounded;

const RANGE_VOLUME: RangeInclusive<f32> = 0.0..=100.0;
const RANGE_MUSIC_OFFSET: RangeInclusive<f32> = -100.0..=100.0;
const RANGE_BLUE: RangeInclusive<f32> = 0.5..=1.0;
const RANGE_BRIGHTNESS: RangeInclusive<f32> = 0.25..=1.0;
const RANGE_SATURATION: RangeInclusive<f32> = 0.0..=1.0;
const RANGE_TELEGRAPH_BRIGHTNESS: RangeInclusive<f32> = 0.25..=1.0;
const RANGE_TELEGRAPH_WIDTH: RangeInclusive<f32> = 0.02..=0.2;
const RANGE_INNER_RADIUS: RangeInclusive<f32> = 0.1..=0.5;
const RANGE_OUTER_RADIUS: RangeInclusive<f32> = 0.05..=0.5;

const FONT_SCALE: f32 = 0.8;
const FONT_FIT_SCALE: f32 = 0.8;

pub struct OptionsButtonWidget {
    pub state: WidgetState,
    pub open_time: Bounded<f32>,
    pub button: IconWidget,
    pub volume: IconWidget,
    pub options: OptionsWidget,
}

impl OptionsButtonWidget {
    pub fn new(assets: &Rc<Assets>, open_time: f32) -> Self {
        Self {
            state: WidgetState::new().with_sfx(WidgetSfxConfig::hover()),
            open_time: Bounded::new_zero(open_time),
            button: IconWidget::new(assets.atlas.settings()),
            volume: IconWidget::new(assets.atlas.volume_mute()).colored(ThemeColor::Danger),
            options: OptionsWidget::new(
                assets,
                vec![
                    // TODO: custom palettes
                    PaletteWidget::new("Classic", Theme::classic()),
                    PaletteWidget::new("Frostlight", Theme::frostlight()),
                    PaletteWidget::new("Stargazer", Theme::stargazer()),
                    PaletteWidget::new("Corruption", Theme::corruption()),
                    PaletteWidget::new("Linksider", Theme::linksider()),
                    PaletteWidget::new("Oxide", Theme::oxide()),
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
        self.volume.update(
            button.translate(vec2(-button_size.x - context.layout_size * 0.5, 0.0)),
            context,
        );
        self.volume
            .state
            .set_visibility(context.context.get_options().volume.music() == 0.0);

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
            let max_size = vec2(17.0, 27.0) * context.layout_size;
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
            graphics: GraphicsWidget::new(assets),
            cursor: CursorWidget::new(assets),
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

pub trait ResetWidget {
    type ValueType: Clone + PartialEq;
}

impl ResetWidget for ToggleWidget {
    type ValueType = bool;
}

impl ResetWidget for SliderWidget {
    type ValueType = f32;
}

impl ResetWidget for ColorSelectWidget {
    type ValueType = ThemeColor;
}

pub struct WithReset<T: ResetWidget> {
    pub reset_value: T::ValueType,
    pub state: WidgetState,
    pub inner: T,
    pub reset: IconButtonWidget,
}

impl<T: ResetWidget> WithReset<T> {
    pub fn new(inner: T, reset_value: T::ValueType, reset_icon: &SubTexture) -> Self {
        Self {
            reset_value,
            state: WidgetState::new(),
            inner,
            reset: IconButtonWidget::new_danger(reset_icon.clone()),
        }
    }

    pub fn update(
        &mut self,
        position: Aabb2<f32>,
        context: &UiContext,
        state: &mut T::ValueType,
    ) -> Aabb2<f32> {
        self.state.update(position, context);
        let size = (self.reset.icon.texture.size() + vec2(2, 2)).as_f32() * context.pixel_size;
        // let reset = position.cut_right(size.x).align_aabb(size, vec2(0.5, 0.5));
        let reset = position
            .align_aabb(size, vec2(1.0, 0.5))
            .translate(vec2(size.x, 0.0));
        self.reset
            .icon
            .state
            .set_visibility(self.reset_value != *state);
        self.reset.update(reset, context);
        if self.reset.icon.state.mouse_left.clicked {
            *state = self.reset_value.clone();
        }
        position
    }
}

pub struct GraphicsWidget {
    pub state: WidgetState,
    pub title: TextWidget,
    pub fullscreen: ToggleWidget,
    pub vsync: ToggleWidget,
    pub crt: WithReset<ToggleWidget>,
    pub brightness: WithReset<SliderWidget>,
    pub blue: WithReset<SliderWidget>,
    pub saturation: WithReset<SliderWidget>,
    pub telegraph_color: WithReset<ColorSelectWidget>,
    pub telegraph_brightness: WithReset<SliderWidget>,
    pub telegraph_width: WithReset<SliderWidget>,
    pub perfect_color: WithReset<ColorSelectWidget>,
}

impl GraphicsWidget {
    pub fn new(assets: &Assets) -> Self {
        let reset_icon = &assets.atlas.reset();
        let def = GraphicsOptions::default();
        Self {
            state: WidgetState::new(),
            title: TextWidget::new("Graphics"),
            fullscreen: ToggleWidget::new("Fullscreen"),
            vsync: ToggleWidget::new("Vsync"),
            crt: WithReset::new(ToggleWidget::new("CRT Shader"), def.crt.enabled, reset_icon),
            brightness: WithReset::new(
                SliderWidget::new("Brightness").with_precision(0),
                def.colors.brightness,
                reset_icon,
            ),
            blue: WithReset::new(
                SliderWidget::new("Blue light").with_precision(0),
                def.colors.blue,
                reset_icon,
            ),
            saturation: WithReset::new(
                SliderWidget::new("Saturation").with_precision(0),
                def.colors.saturation,
                reset_icon,
            ),
            telegraph_color: WithReset::new(
                ColorSelectWidget::new(
                    "Telegraph color",
                    [ThemeColor::Light, ThemeColor::Highlight],
                ),
                def.lights.telegraph_color,
                reset_icon,
            ),
            telegraph_brightness: WithReset::new(
                SliderWidget::new("Telegraph brightness").with_precision(0),
                def.lights.telegraph_brightness,
                reset_icon,
            ),
            telegraph_width: WithReset::new(
                SliderWidget::new("Telegraph width").with_precision(0),
                def.lights.telegraph_width,
                reset_icon,
            ),
            perfect_color: WithReset::new(
                ColorSelectWidget::new("Perfect color", [ThemeColor::Light, ThemeColor::Highlight]),
                def.lights.perfect_color,
                reset_icon,
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

        let context = &mut context.scale_font(FONT_SCALE);

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

        let context = &mut context.scale_font(FONT_FIT_SCALE);

        let window = context.context.geng.window();
        self.fullscreen.checked = window.is_fullscreen();
        self.fullscreen.update(next_row(), context);
        if self.fullscreen.state.mouse_left.clicked {
            window.toggle_fullscreen();
        }

        self.vsync.checked = window.is_vsync();
        self.vsync.update(next_row(), context);
        if self.vsync.state.mouse_left.clicked {
            let vsync = !self.vsync.checked;
            window.set_vsync(vsync);
            state.display.vsync = vsync;
        }

        let crt = self.crt.update(next_row(), context, &mut state.crt.enabled);
        self.crt
            .inner
            .update_state(crt, context, &mut state.crt.enabled);

        let brightness = self
            .brightness
            .update(next_row(), context, &mut state.colors.brightness);
        self.brightness.inner.update_value_percent(
            brightness,
            context,
            &mut state.colors.brightness,
            RANGE_BRIGHTNESS,
        );
        let blue = self
            .blue
            .update(next_row(), context, &mut state.colors.blue);
        self.blue
            .inner
            .update_value_percent(blue, context, &mut state.colors.blue, RANGE_BLUE);
        let saturation = self
            .saturation
            .update(next_row(), context, &mut state.colors.saturation);
        self.saturation.inner.update_value_percent(
            saturation,
            context,
            &mut state.colors.saturation,
            RANGE_SATURATION,
        );

        let t_color =
            self.telegraph_color
                .update(next_row(), context, &mut state.lights.telegraph_color);
        self.telegraph_color
            .inner
            .update(t_color, context, &mut state.lights.telegraph_color);
        let t_bright = self.telegraph_brightness.update(
            next_row(),
            context,
            &mut state.lights.telegraph_brightness,
        );
        self.telegraph_brightness.inner.update_value_percent(
            t_bright,
            context,
            &mut state.lights.telegraph_brightness,
            RANGE_TELEGRAPH_BRIGHTNESS,
        );
        let t_width =
            self.telegraph_width
                .update(next_row(), context, &mut state.lights.telegraph_width);
        self.telegraph_width.inner.update_value_percent(
            t_width,
            context,
            &mut state.lights.telegraph_width,
            RANGE_TELEGRAPH_WIDTH,
        );
        let p_color =
            self.perfect_color
                .update(next_row(), context, &mut state.lights.perfect_color);
        self.perfect_color
            .inner
            .update(p_color, context, &mut state.lights.perfect_color);

        let mut position = position;
        position.min.y = min_y;
        self.state.update(position, context);
    }
}

pub struct CursorWidget {
    pub state: WidgetState,
    pub title: TextWidget,
    pub show_trail: WithReset<ToggleWidget>,
    pub inner_radius: WithReset<SliderWidget>,
    pub show_perfect_radius: WithReset<ToggleWidget>,
    pub outer_radius: WithReset<SliderWidget>,
    pub outer_color: WithReset<ColorSelectWidget>,
    pub show_rhythm_circles: WithReset<ToggleWidget>,
    pub show_rhythm_only_miss: WithReset<ToggleWidget>,
}

impl CursorWidget {
    pub fn new(assets: &Assets) -> Self {
        let reset_icon = &assets.atlas.reset();
        let def = CursorOptions::default();
        Self {
            state: WidgetState::new(),
            title: TextWidget::new("Cursor"),
            show_trail: WithReset::new(ToggleWidget::new("Show trail"), def.show_trail, reset_icon),
            inner_radius: WithReset::new(
                SliderWidget::new("Trail size").with_precision(2),
                def.inner_radius,
                reset_icon,
            ),
            show_perfect_radius: WithReset::new(
                ToggleWidget::new("Show outline"),
                def.show_perfect_radius,
                reset_icon,
            ),
            outer_radius: WithReset::new(
                SliderWidget::new("Outline width").with_precision(2),
                def.outer_radius,
                reset_icon,
            ),
            outer_color: WithReset::new(
                ColorSelectWidget::new(
                    "Outline color",
                    [
                        // TODO: currently does not render properly in the menu because of transparency
                        // ThemeColor::Dark,
                        ThemeColor::Light,
                        ThemeColor::Highlight,
                        ThemeColor::Danger,
                    ],
                ),
                def.outer_color,
                reset_icon,
            ),
            show_rhythm_circles: WithReset::new(
                ToggleWidget::new("Rhythm circles"),
                def.show_rhythm_circles,
                reset_icon,
            ),
            show_rhythm_only_miss: WithReset::new(
                ToggleWidget::new("Show only misses"),
                def.show_rhythm_only_miss,
                reset_icon,
            ),
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

        let context = &mut context.scale_font(FONT_SCALE);

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

        let context = &mut context.scale_font(FONT_FIT_SCALE);

        let show_trail = self
            .show_trail
            .update(next_row(), context, &mut state.show_trail);
        self.show_trail
            .inner
            .update_state(show_trail, context, &mut state.show_trail);

        let inner_r = self
            .inner_radius
            .update(next_row(), context, &mut state.inner_radius);
        self.inner_radius.inner.update_value(
            inner_r,
            context,
            &mut state.inner_radius,
            RANGE_INNER_RADIUS,
        );

        let show_p =
            self.show_perfect_radius
                .update(next_row(), context, &mut state.show_perfect_radius);
        self.show_perfect_radius.inner.update_state(
            show_p,
            context,
            &mut state.show_perfect_radius,
        );
        if state.show_perfect_radius {
            self.outer_radius.state.show();
            let outer_r = self
                .outer_radius
                .update(next_row(), context, &mut state.outer_radius);
            self.outer_radius.inner.update_value(
                outer_r,
                context,
                &mut state.outer_radius,
                RANGE_OUTER_RADIUS,
            );

            self.outer_color.state.show();
            let outer_c = self
                .outer_color
                .update(next_row(), context, &mut state.outer_color);
            self.outer_color
                .inner
                .update(outer_c, context, &mut state.outer_color);
        } else {
            self.outer_radius.state.hide();
            self.outer_color.state.hide();
        }

        let show_r =
            self.show_rhythm_circles
                .update(next_row(), context, &mut state.show_rhythm_circles);
        self.show_rhythm_circles.inner.update_state(
            show_r,
            context,
            &mut state.show_rhythm_circles,
        );
        if state.show_rhythm_circles {
            self.show_rhythm_only_miss.state.show();
            let show_rom = self.show_rhythm_only_miss.update(
                next_row(),
                context,
                &mut state.show_rhythm_only_miss,
            );
            self.show_rhythm_only_miss.inner.update_state(
                show_rom,
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

        let context = &mut context.scale_font(FONT_SCALE);

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

        let context = &mut context.scale_font(FONT_FIT_SCALE);

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
