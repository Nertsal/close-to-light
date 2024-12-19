use super::*;

pub struct EditorConfigUi {
    // pub assets: Rc<Assets>,
    // pub state: WidgetState,

    // pub timing: TextWidget,
    // pub bpm: ValueWidget<FloatTime>,
    // // pub tempo:
    // // pub offset: ValueWidget<Time>,
    // pub music: TextWidget,
    // pub level: TextWidget,
    // pub level_name: InputWidget,
    // pub level_delete: ButtonWidget,
    // pub level_create: ButtonWidget,
    // pub all_levels: TextWidget,
    // pub all_level_names: Vec<(IconWidget, IconWidget, TextWidget)>,

    // pub timeline: TextWidget,

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
        Self {
            // assets: assets.clone(),
            // state: WidgetState::new(),

            // timing: TextWidget::new("Timing"),
            // bpm: ValueWidget::new_range("BPM", r32(150.0), r32(60.0)..=r32(240.0), r32(1.0)), // TODO: different
            // // offset: ValueWidget::new_range("Offset", r32(0.0), r32(-10.0)..=r32(10.0), r32(0.1)),
            // music: TextWidget::new("Music"),
            // level: TextWidget::new("Difficulty"),
            // level_name: InputWidget::new(""),
            // level_delete: ButtonWidget::new("Delete"),
            // level_create: ButtonWidget::new("Create"),
            // all_levels: TextWidget::new("All Dificulties"),
            // all_level_names: Vec::new(),

            // timeline: TextWidget::new("Timeline"),

            // scroll_by: ValueWidget::new_range(
            //     "Scroll by",
            //     BeatTime::WHOLE,
            //     BeatTime::QUARTER..=BeatTime::WHOLE * 4,
            //     BeatTime::QUARTER,
            // ),
            // shift_scroll: ValueWidget::new_range(
            //     "Shift scroll",
            //     BeatTime::QUARTER,
            //     BeatTime::EIGHTH..=BeatTime::WHOLE,
            //     BeatTime::EIGHTH,
            // ),
            // alt_scroll: ValueWidget::new_range(
            //     "Alt scroll",
            //     BeatTime::WHOLE * 10,
            //     BeatTime::WHOLE..=BeatTime::WHOLE * 20,
            //     BeatTime::HALF,
            // ),
        }
    }

    pub fn layout(
        &mut self,
        position: Aabb2<f32>,
        context: &UiContext,
        editor: &Editor,
        actions: &mut Vec<EditorStateAction>,
    ) {
        // self.state.update(position, context);

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
    }
}

// let mut bar = columns[2];
// let timeline = bar.cut_top(context.font_size);
// self.timeline.update(timeline, context);

// // TODO: scroll time
// // let mut config = state.config.clone();

// // let scroll_by = bar.cut_top(context.font_size);
// // self.scroll_by
// //     .update(scroll_by, context, &mut config.scroll_normal);

// // let shift_scroll = bar.cut_top(context.font_size);
// // self.shift_scroll
// //     .update(shift_scroll, context, &mut config.scroll_slow);

// // let alt_scroll = bar.cut_top(context.font_size);
// // self.alt_scroll
// //     .update(alt_scroll, context, &mut config.scroll_fast);

// // actions.push(EditorAction::SetConfig(config).into());
//     }
// }
