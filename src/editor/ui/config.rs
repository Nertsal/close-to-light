use crate::ui::geometry::Geometry;

use super::*;

pub struct EditorConfigUi {
    // TODO scroll time
    // /// Normal time scroll.
    // pub scroll_by: ValueWidget<BeatTime>,
    // /// Slow time scroll.
    // pub shift_scroll: ValueWidget<BeatTime>,
    // /// Fast time scroll.
    // pub alt_scroll: ValueWidget<BeatTime>,
    // pub snap_to: CheckboxWidget,
}

impl EditorConfigUi {
    pub fn new() -> Self {
        Self {}
    }

    pub fn layout(
        &mut self,
        position: Aabb2<f32>,
        context: &UiContext,
        editor: &Editor,
        actions: &mut Vec<EditorStateAction>,
    ) {
        let main = position;

        let width = context.layout_size * 7.0;
        let spacing = context.layout_size * 5.0;

        let columns = 3;
        let total_width = columns as f32 * width + (columns - 1) as f32 * spacing;
        let column = Aabb2::point(vec2(main.center().x - total_width / 2.0, main.max.y))
            .extend_right(width)
            .extend_down(main.height());

        let columns = column.stack(vec2(width + spacing, 0.0), columns);

        let mut bar = columns[0];
        let timing = bar.cut_top(context.font_size);
        let text = context.state.get_or(|| TextWidget::new("Timing"));
        text.update(timing, context);

        let bpm = bar.cut_top(context.font_size);
        let slider = context
            .state
            .get_or(|| TextWidget::new(format!("BPM: {:.1}", editor.group.music.meta.bpm)));
        slider.update(bpm, context);

        // let (offset, bar) = layout::cut_top_down(bar, context.font_size);
        // self.offset.update(offset, context);

        let mut bar = columns[1];
        let music = bar.cut_top(context.font_size);
        let text = context
            .state
            .get_or(|| TextWidget::new(format!("Music: {}", editor.group.music.meta.name)));
        text.update(music, context);

        bar.cut_top(context.layout_size);

        let level = bar.cut_top(context.font_size);
        let text = context.state.get_or(|| TextWidget::new("Difficulty"));
        text.update(level, context);

        let name = bar.cut_top(context.font_size);
        let delete = bar.cut_top(context.font_size);
        if let Some(level_editor) = &editor.level_edit {
            let input = context.state.get_or(|| InputWidget::new(""));
            input.sync(&level_editor.name, context);
            input.update(name, context);
            actions.push(LevelAction::SetName(input.raw.clone()).into());

            let button = context
                .state
                .get_or(|| ButtonWidget::new("Delete").color(ThemeColor::Danger));
            button.update(delete, context);
            if button.text.state.clicked {
                let index = level_editor.static_level.level_index;
                actions.push(EditorAction::DeleteLevel(index).into());
            }
        }

        let create = bar.cut_top(context.font_size);
        let button = context.state.get_or(|| ButtonWidget::new("Create"));
        button.update(create, context);
        if button.text.state.clicked {
            actions.push(EditorAction::NewLevel.into());
        }

        bar.cut_top(context.layout_size);
        let all = bar.cut_top(context.font_size);
        let text = context.state.get_or(|| TextWidget::new("All Difficulties"));
        text.update(all, context);

        let names: Vec<_> = editor
            .group
            .cached
            .data
            .levels
            .iter()
            .map(|level| level.meta.name.clone())
            .collect();

        let max = names.len().saturating_sub(1);
        for (i, mut level_name) in names.into_iter().enumerate() {
            let level = context.state.get_or_default::<TextWidget>();

            let name = bar.cut_top(context.font_size);
            level.update(name, context);

            if let Some(level_editor) = &editor.level_edit {
                if level_editor.static_level.level_index == i {
                    level_name = level_editor.name.clone().into();
                }
            }
            level.text = level_name;

            if level.state.clicked {
                if editor.is_changed() {
                    actions.push(
                        EditorAction::PopupConfirm(
                            ConfirmAction::ChangeLevelUnsaved(i),
                            "there are unsaved changes".into(),
                        )
                        .into(),
                    );
                } else {
                    actions.push(EditorAction::ChangeLevel(i).into());
                }
            }

            let width = name.height();
            let mut icons = name;
            let icons = icons.cut_left(width).translate(vec2(-width, 0.0));

            if level.state.hovered || context.can_focus() && icons.contains(context.cursor.position)
            {
                let icons = icons.split_rows(2);
                let up = icons[0];
                let up_hover = up.contains(context.cursor.position);
                let down = icons[1];
                let down_hover = down.contains(context.cursor.position);

                if i > 0 && (up_hover || !down_hover) {
                    let icon_up = context
                        .state
                        .get_or(|| IconWidget::new(&context.context.assets.sprites.arrow_up));
                    icon_up.update(up, context);
                    if icon_up.state.clicked {
                        actions.push(EditorAction::MoveLevelLow(i).into());
                    }
                }

                if i < max && (down_hover || !up_hover) {
                    let icon_down = context
                        .state
                        .get_or(|| IconWidget::new(&context.context.assets.sprites.arrow_down));
                    icon_down.update(down, context);
                    if icon_down.state.clicked {
                        actions.push(EditorAction::MoveLevelHigh(i).into());
                    }
                }
            }
        }

        // Timeline
        {
            let mut bar = columns[2];
            let timeline = bar.cut_top(context.font_size);
            let title = context.state.get_or(|| TextWidget::new("Timeline"));
            title.update(timeline, context);

            let mut config = editor.config.clone();
            let value_height = context.font_size * 1.2;
            let spacing = context.font_size * 0.3;

            let scroll_by = bar.cut_top(value_height);
            bar.cut_top(spacing);
            let value = context.state.get_or(|| {
                BeatValueWidget::new(
                    "Scroll by",
                    BeatTime::WHOLE,
                    BeatTime::QUARTER..=BeatTime::WHOLE * 4,
                    BeatTime::QUARTER,
                )
            });
            value.update(scroll_by, context, &mut config.scroll_normal);

            let shift_scroll = bar.cut_top(value_height);
            bar.cut_top(spacing);
            let value = context.state.get_or(|| {
                BeatValueWidget::new(
                    "Shift scroll",
                    BeatTime::QUARTER,
                    BeatTime::EIGHTH..=BeatTime::WHOLE,
                    BeatTime::EIGHTH,
                )
            });
            value.update(shift_scroll, context, &mut config.scroll_slow);

            let alt_scroll = bar.cut_top(value_height);
            bar.cut_top(spacing);
            let value = context.state.get_or(|| {
                BeatValueWidget::new(
                    "Alt scroll",
                    BeatTime::WHOLE * 10,
                    BeatTime::WHOLE..=BeatTime::WHOLE * 20,
                    BeatTime::HALF,
                )
            });
            value.update(alt_scroll, context, &mut config.scroll_fast);

            actions.push(EditorAction::SetConfig(config).into());
        }
    }
}

pub struct BeatValueWidget {
    pub state: WidgetState,
    pub value_text: InputWidget,
    pub control_state: WidgetState,
    pub range: RangeInclusive<BeatTime>,
    pub value: BeatTime,
    pub scroll_by: BeatTime,
}

impl BeatValueWidget {
    pub fn new(
        text: impl Into<Name>,
        value: BeatTime,
        range: RangeInclusive<BeatTime>,
        scroll_by: BeatTime,
    ) -> Self {
        Self {
            state: WidgetState::new(),
            value_text: InputWidget::new(text).format(InputFormat::Ratio),
            control_state: WidgetState::new(),
            range,
            value,
            scroll_by,
        }
    }

    pub fn update(&mut self, position: Aabb2<f32>, context: &UiContext, state: &mut BeatTime) {
        self.value = *state;
        let mut target = *state;
        self.state.update(position, context);
        let mut main = position;

        let control_height = context.font_size * 0.2;
        let control = main.cut_bottom(control_height);
        self.control_state.update(control, context);
        let (min, max) = (*self.range.start(), *self.range.end());

        // Drag value
        if self.control_state.pressed {
            // (0,0) in the center, range -0.5..=0.5
            let convert = |pos| {
                (pos - self.control_state.position.center()) / self.control_state.position.size()
            };
            let pos = convert(context.cursor.position);
            let t = (pos.x + 0.5).clamp(0.0, 1.0);
            let steps = (max - min).units() as f32 / self.scroll_by.units() as f32 * t;
            target = min + self.scroll_by * steps.round() as Time;
        } else if self.control_state.hovered && context.cursor.scroll != 0.0 {
            // Scroll value
            let delta = self.scroll_by * context.cursor.scroll.signum() as Time;
            target += delta;
        }

        self.value_text.update(main, context);
        if self.value_text.editing {
            // TODO: handle errors
            if let Some((num, den)) = self.value_text.raw.split_once('/') {
                if let Ok(num) = num.parse::<Time>() {
                    if let Ok(den) = den.parse::<Time>() {
                        if 16 % den == 0 {
                            let units = num * (16 / den);
                            target = BeatTime::from_units(units);
                        }
                    }
                }
            }
        }

        // Check bounds
        target = target.clamp_range(min..=max);

        if !self.value_text.editing {
            let value = Ratio::new_raw(target.units(), BeatTime::UNITS_PER_BEAT).reduced();
            self.value_text
                .sync(&format!("{}/{}", value.numer(), value.denom()), context);
        }

        *state = target;
    }
}

impl Widget for BeatValueWidget {
    fn draw(&self, context: &UiContext) -> Geometry {
        let theme = context.theme();
        let mut geometry = self.value_text.draw(context);

        let quad = self.control_state.position;
        let width = quad.height() * 0.05;
        let quad = quad.extend_uniform(-width);

        {
            let (min, max) = (*self.range.start(), *self.range.end());
            let t = (self.value - min).units() as f32 / (max - min).units() as f32;
            let mut fill = quad;
            let fill = fill.cut_left(fill.width() * t);

            let tick = |t: f32| quad.align_pos(vec2(t, 0.5));

            geometry.merge(context.geometry.texture_pp(
                tick(0.0),
                theme.highlight,
                0.5,
                &context.context.assets.sprites.timeline.tick_smol,
            ));
            geometry.merge(context.geometry.texture_pp(
                tick(t),
                theme.highlight,
                0.5,
                &context.context.assets.sprites.timeline.tick_tiny,
            ));
            geometry.merge(context.geometry.texture_pp(
                tick(1.0),
                theme.light,
                0.5,
                &context.context.assets.sprites.timeline.tick_smol,
            ));

            geometry.merge(context.geometry.quad(fill, theme.highlight));
            geometry.merge(context.geometry.quad(quad, theme.light));
        }

        geometry
    }
}
