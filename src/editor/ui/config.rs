use super::*;

pub struct EditorConfigWidget {
    pub assets: Rc<Assets>,
    pub state: WidgetState,

    pub timing: TextWidget,
    pub bpm: ValueWidget<Time>,
    // pub tempo:
    pub offset: ValueWidget<Time>,

    pub music: TextWidget,
    pub level: TextWidget,
    pub level_name: InputWidget,
    pub level_delete: ButtonWidget,
    pub level_create: ButtonWidget,
    pub all_levels: TextWidget,
    pub all_level_names: Vec<(IconWidget, IconWidget, TextWidget)>,

    pub timeline: TextWidget,
    /// Normal time scroll.
    pub scroll_by: ValueWidget<Time>, // TODO: 1/4 instead of 0.25
    /// Slow time scroll.
    pub shift_scroll: ValueWidget<Time>,
    /// Fast time scroll.
    pub alt_scroll: ValueWidget<Time>,
    // pub snap_to: CheckboxWidget,
}

impl EditorConfigWidget {
    pub fn new(assets: &Rc<Assets>) -> Self {
        Self {
            assets: assets.clone(),
            state: WidgetState::new(),

            timing: TextWidget::new("Timing"),
            bpm: ValueWidget::new_range("BPM", r32(150.0), r32(60.0)..=r32(240.0), r32(1.0)), // TODO: different
            offset: ValueWidget::new_range("Offset", r32(0.0), r32(-10.0)..=r32(10.0), r32(0.1)),

            music: TextWidget::new("Music"),
            level: TextWidget::new("Difficulty"),
            level_name: InputWidget::new(""),
            level_delete: ButtonWidget::new("Delete"),
            level_create: ButtonWidget::new("Create"),
            all_levels: TextWidget::new("All Dificulties"),
            all_level_names: Vec::new(),

            timeline: TextWidget::new("Timeline"),
            scroll_by: ValueWidget::new_range(
                "Scroll by",
                r32(1.0),
                r32(0.25)..=r32(4.0),
                r32(0.25),
            ),
            shift_scroll: ValueWidget::new_range(
                "Shift scroll",
                r32(0.25),
                r32(0.125)..=r32(1.0),
                r32(0.125),
            ),
            alt_scroll: ValueWidget::new_range(
                "Alt scroll",
                r32(10.0),
                r32(1.0)..=r32(20.0),
                r32(0.5),
            ),
        }
    }
}

impl StatefulWidget for EditorConfigWidget {
    type State<'a> = (&'a Editor, Vec<EditorStateAction>);

    fn state_mut(&mut self) -> &mut WidgetState {
        &mut self.state
    }

    fn update(
        &mut self,
        position: Aabb2<f32>,
        context: &mut UiContext,
        (state, actions): &mut Self::State<'_>,
    ) {
        self.state.update(position, context);

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
        self.timing.update(timing, context);

        let bpm = bar.cut_top(context.font_size);
        let mut bpm_value = state.group.music.meta.bpm;
        self.bpm.update(bpm, context, &mut bpm_value); // TODO: remove

        // let (offset, bar) = layout::cut_top_down(bar, context.font_size);
        // self.offset.update(offset, context);

        let mut bar = columns[1];
        let music = bar.cut_top(context.font_size);
        self.music.text = format!("Music: {}", state.group.music.meta.name).into();
        self.music.update(music, context);

        bar.cut_top(context.layout_size);

        let level = bar.cut_top(context.font_size);
        self.level.update(level, context);

        let name = bar.cut_top(context.font_size);
        let delete = bar.cut_top(context.font_size);
        if let Some(level_editor) = &state.level_edit {
            self.level_name.sync(&level_editor.name, context);
            self.level_name.show();
            self.level_name.update(name, context);
            actions.push(LevelAction::SetName(self.level_name.raw.clone()).into());

            self.level_delete.show();
            self.level_delete.update(delete, context);
            if self.level_delete.text.state.clicked {
                let index = level_editor.static_level.level_index;
                actions.push(EditorAction::DeleteLevel(index).into());
            }
        } else {
            self.level_name.hide();
            self.level_delete.hide();
        }

        let create = bar.cut_top(context.font_size);
        self.level_create.update(create, context);
        if self.level_create.text.state.clicked {
            actions.push(EditorAction::NewLevel.into());
        }

        bar.cut_top(context.layout_size);
        let all = bar.cut_top(context.font_size);
        self.all_levels.update(all, context);

        let names: Vec<_> = state
            .group
            .cached
            .data
            .levels
            .iter()
            .map(|level| level.meta.name.clone())
            .collect();
        if self.all_level_names.len() != names.len() {
            self.all_level_names = names
                .iter()
                .map(|name| {
                    (
                        IconWidget::new(&self.assets.sprites.arrow_up),
                        IconWidget::new(&self.assets.sprites.arrow_down),
                        TextWidget::new(name.clone()),
                    )
                })
                .collect();
        }

        let max = names.len().saturating_sub(1);
        for (i, ((icon_up, icon_down, level), mut level_name)) in
            self.all_level_names.iter_mut().zip(names).enumerate()
        {
            let name = bar.cut_top(context.font_size);
            level.update(name, context);

            if let Some(level_editor) = &state.level_edit {
                if level_editor.static_level.level_index == i {
                    level_name = self.level_name.text.text.clone();
                }
            }
            level.text = level_name;

            if level.state.clicked {
                if state.is_changed() {
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

            if level.state.hovered || context.can_focus && icons.contains(context.cursor.position) {
                let icons = icons.split_rows(2);
                let up = icons[0];
                let up_hover = up.contains(context.cursor.position);
                let down = icons[1];
                let down_hover = down.contains(context.cursor.position);

                if i > 0 && (up_hover || !down_hover) {
                    icon_up.show();
                    icon_up.update(up, context);
                    if icon_up.state.clicked {
                        actions.push(EditorAction::MoveLevelLow(i).into());
                    }
                } else {
                    icon_up.hide();
                }

                if i < max && (down_hover || !up_hover) {
                    icon_down.show();
                    icon_down.update(down, context);
                    if icon_down.state.clicked {
                        actions.push(EditorAction::MoveLevelHigh(i).into());
                    }
                } else {
                    icon_down.hide();
                }
            } else {
                icon_up.hide();
                icon_down.hide();
            }
        }

        let mut bar = columns[2];
        let timeline = bar.cut_top(context.font_size);
        self.timeline.update(timeline, context);

        let mut config = state.config.clone();

        let scroll_by = bar.cut_top(context.font_size);
        self.scroll_by
            .update(scroll_by, context, &mut config.scroll_normal);

        let shift_scroll = bar.cut_top(context.font_size);
        self.shift_scroll
            .update(shift_scroll, context, &mut config.scroll_slow);

        let alt_scroll = bar.cut_top(context.font_size);
        self.alt_scroll
            .update(alt_scroll, context, &mut config.scroll_fast);

        actions.push(EditorAction::SetConfig(config).into());
    }
}
