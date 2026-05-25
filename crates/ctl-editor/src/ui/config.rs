use super::*;

pub struct EditorConfigUi {}

impl EditorConfigUi {
    #[allow(clippy::new_without_default)]
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

        let column_widths = [1.0, 1.5, 1.0].map(|x| x * width);
        let total_width =
            column_widths.iter().copied().sum::<f32>() + (column_widths.len() - 1) as f32 * spacing;
        let mut column = Aabb2::point(vec2(main.center().x - total_width / 2.0, main.max.y))
            .extend_right(width)
            .extend_down(main.height());

        let columns = column_widths.map(|width| {
            let c = column.with_width(width, 0.0);
            column = column.translate(vec2(width + spacing, 0.0));
            c
        });

        let mut bar = columns[0];
        let timing = bar.cut_top(context.font_size);
        let text = context.state.get_root_or(|| TextWidget::new("Timing"));
        text.update(timing, context);

        // let bpm_pos = bar.cut_top(context.font_size);
        // if let Some(level_editor) = &editor.level_edit
        //     && let Some(timing) = level_editor.level.timing.points.first()
        // {
        //     let mut bpm = 60.0 / timing.beat_time.as_f32();
        //     let slider = context.state.get_root_or(|| {
        //         ValueWidget::new(
        //             "BPM",
        //             bpm,
        //             ValueControl::Slider {
        //                 min: 1.0,
        //                 max: 500.0,
        //             },
        //             1.0,
        //         )
        //     });
        //     slider.update(bpm_pos, context, &mut bpm);
        //     actions.push(LevelAction::TimingUpdate(0, r32(60.0 / bpm)).into());
        // }

        // let (offset, bar) = layout::cut_top_down(bar, context.font_size);
        // self.offset.update(offset, context);

        let mut bar = columns[0];

        // Music
        let button_pos = bar.cut_top(context.font_size * 1.4);
        let button = context
            .state
            .get_root_or(|| ButtonWidget::new("Select Music"));
        button.text.text = if editor.group.music.is_some() {
            "Change Music"
        } else {
            "Select Music"
        }
        .into();
        button.update(button_pos, context);
        #[cfg(not(target_arch = "wasm32"))]
        if button.text.state.mouse_left.clicked
            && let Some(path) = rfd::FileDialog::new()
                .add_filter("music", &["mp3"])
                .set_can_create_directories(false)
                .pick_file()
        {
            actions.push(EditorStateAction::SelectMusicFile(path));
        }

        let music_pos = bar.cut_top(context.font_size);
        if let Some(music) = &editor.group.music {
            // Music name
            let input = context.state.get_root_or(|| InputWidget::new("Music name"));
            if !input.editing {
                input.sync(&music.meta.name, context);
            }
            input.update(music_pos, context);
            if !input.editing && *music.meta.name != input.raw {
                actions.push(EditorStateAction::SetGroupName(input.raw.clone()));
            }

            bar.cut_top(context.layout_size * 0.5);

            // Music authors
            let authors = bar.cut_top(context.font_size);
            let text = context.state.get_root_or(|| TextWidget::new("Authors"));
            text.update(authors, context);
            for (i, author) in music.meta.authors.iter().enumerate() {
                let mut author_pos = bar.cut_top(context.font_size);

                let delete_pos = author_pos.cut_left(author_pos.height());
                let delete = context.state.get_root_or(|| {
                    IconButtonWidget::new_danger(context.context.assets.atlas.discard())
                });
                delete.update(delete_pos, context);
                if delete.icon.state.mouse_left.clicked {
                    actions.push(EditorStateAction::RemoveMusicAuthor(i));
                }

                let input = context.state.get_root_or(|| InputWidget::new("Name"));
                if !input.editing {
                    input.sync(&author.name, context);
                }
                input.update(author_pos, context);
                if !input.editing && *author.name != input.raw {
                    actions.push(EditorStateAction::UpdateMusicAuthor(
                        i,
                        MusicianInfo {
                            name: input.raw.clone().into(),
                            romanized: input.raw.clone().into(), // TODO
                            ..author.clone()
                        },
                    ));
                }
            }
            let button_pos = bar
                .cut_top(context.font_size)
                .with_width(context.font_size * 2.0, 0.5);
            let button = context.state.get_root_or(|| ButtonWidget::new("+"));
            button.update(button_pos, context);
            if button.text.state.mouse_left.clicked {
                actions.push(EditorStateAction::AddMusicAuthor(MusicianInfo {
                    id: 0,
                    name: "".into(),
                    romanized: "".into(),
                }));
            }
        }

        let mut bar = columns[1];

        let all = bar.cut_top(context.font_size * 1.4);
        let text = context
            .state
            .get_root_or(|| TextWidget::new("Select Difficulty").max_sized());
        text.update(all, context);

        let names: Vec<_> = editor
            .group
            .cached
            .local
            .meta
            .levels
            .iter()
            .map(|level| level.name.clone())
            .collect();

        let max = names.len().saturating_sub(1);
        for (level_idx, level_name) in names.into_iter().enumerate() {
            let is_selected = editor
                .level_edit
                .as_ref()
                .is_some_and(|editor| editor.static_level.level_index == level_idx);

            let name = bar
                .cut_top(context.font_size * 1.2)
                .with_width(bar.width() * 0.8, 0.5);

            // Name
            let level = context
                .state
                .get_root_or(|| ButtonWidget::new("<diff name>"));
            level.text.text = level_name;
            level.bg_color = if is_selected {
                ThemeColor::Highlight
            } else {
                ThemeColor::Light
            };
            level.update(name, context);

            if level.text.state.mouse_left.clicked && !is_selected {
                if editor.is_changed() {
                    actions.push(
                        EditorAction::PopupConfirm(
                            ConfirmAction::ChangeLevelUnsaved(level_idx),
                            "unsaved changes will be lost".into(),
                            "change difficulty".into(),
                            "cancel".into(),
                        )
                        .into(),
                    );
                } else {
                    actions.push(EditorAction::ChangeLevel(level_idx).into());
                }
            }

            // Delete difficulty
            let button_delete = name
                .clone()
                .cut_left(name.height())
                .translate(vec2(-name.height(), 0.0));
            let button = context.state.get_root_or(|| {
                IconButtonWidget::new_danger(context.context.assets.atlas.discard())
            });
            button.update(button_delete, context);
            if button.icon.state.mouse_left.clicked {
                actions.push(EditorAction::DeleteLevel(level_idx).into());
            }

            // Icons to reorder the levels
            let icons_width = name.height();
            let icons = name
                .clone()
                .cut_right(icons_width)
                .translate(vec2(icons_width, 0.0));

            if level.text.state.hovered
                || context.can_focus() && icons.contains(context.cursor.position)
            {
                let icons = icons.split_rows(2);
                let up = icons[0];
                let up_hover = up.contains(context.cursor.position);
                let down = icons[1];
                let down_hover = down.contains(context.cursor.position);

                // Move up
                if level_idx > 0 && (up_hover || !down_hover) {
                    let icon_up = context
                        .state
                        .get_root_or(|| IconWidget::new(context.context.assets.atlas.arrow_up()));
                    icon_up.update(up, context);
                    if icon_up.state.mouse_left.clicked {
                        actions.push(EditorAction::MoveLevelLow(level_idx).into());
                    }
                }

                // Move down
                if level_idx < max && (down_hover || !up_hover) {
                    let icon_down = context
                        .state
                        .get_root_or(|| IconWidget::new(context.context.assets.atlas.arrow_down()));
                    icon_down.update(down, context);
                    if icon_down.state.mouse_left.clicked {
                        actions.push(EditorAction::MoveLevelHigh(level_idx).into());
                    }
                }
            }
        }

        // Create difficulty
        let create = bar
            .cut_top(context.font_size * 1.0)
            .with_width(context.font_size * 2.0, 0.5);
        let button = context.state.get_root_or(|| ButtonWidget::new("+"));
        button.update(create, context);
        if button.text.state.mouse_left.clicked {
            actions.push(EditorAction::NewLevel.into());
        }

        let mut bar = columns[2];

        // Difficulties - Detailed view
        let all = bar.cut_top(context.font_size * 1.4);
        let text = context
            .state
            .get_root_or(|| TextWidget::new("All Difficulties").max_sized());
        text.update(all, context);

        for (level_idx, level_info) in editor.group.cached.local.meta.levels.iter().enumerate() {
            let mut level_bounds = bar;

            // Name
            let name = bar
                .cut_top(context.font_size * 1.2)
                .with_width(bar.width() * 0.9, 0.5);
            let level = context.state.get_root_or(|| InputWidget::new("Name"));
            if !level.editing {
                level.sync(&level_info.name, context);
            }
            level.update(name, context);
            if !level.editing && *level_info.name != level.raw {
                actions.push(EditorStateAction::SetLevelName(
                    level_idx,
                    level.raw.clone().into(),
                ));
            }

            // Authors
            let authors = bar.cut_top(context.font_size * 0.8);
            let text = context.state.get_root_or(|| TextWidget::new("Authors"));
            text.update(authors, context);
            for (author_idx, author) in level_info.authors.iter().enumerate() {
                let mut author_pos = bar
                    .cut_top(context.font_size)
                    .with_width(bar.width() * 0.8, 0.5);

                // Delete
                let delete_pos = author_pos.cut_left(author_pos.height());
                let delete = context.state.get_root_or(|| {
                    IconButtonWidget::new_danger(context.context.assets.atlas.discard())
                });
                delete.update(delete_pos, context);
                if delete.icon.state.mouse_left.clicked {
                    actions.push(EditorStateAction::RemoveLevelAuthor(level_idx, author_idx));
                }

                // Name
                let input = context.state.get_root_or(|| InputWidget::new("Name"));
                if !input.editing {
                    input.sync(&author.name, context);
                }
                input.update(author_pos, context);
                if !input.editing && *author.name != input.raw {
                    actions.push(EditorStateAction::UpdateLevelAuthor(
                        level_idx,
                        author_idx,
                        MapperInfo {
                            name: input.raw.clone().into(),
                            romanized: input.raw.clone().into(), // TODO
                            ..author.clone()
                        },
                    ));
                }
            }
            // Add author
            let button_pos = bar
                .cut_top(context.font_size)
                .with_width(context.font_size * 2.0, 0.5);
            let button = context.state.get_root_or(|| ButtonWidget::new("+"));
            button.update(button_pos, context);
            if button.text.state.mouse_left.clicked {
                actions.push(EditorStateAction::AddLevelAuthor(
                    level_idx,
                    MapperInfo {
                        id: 0,
                        name: "".into(),
                        romanized: "".into(),
                    },
                ));
            }

            level_bounds.min.y = bar.max.y;
            let bounds = context.state.get_root_or(|| {
                GeometryWidget::new(|state, context| {
                    let mut geometry = ctl_ui::geometry::Geometry::new();
                    let width = 5.0;
                    geometry.merge(context.geometry.quad_outline(
                        state.position.extend_uniform(width * 1.25),
                        width,
                        context.theme().light,
                    ));
                    geometry
                })
            });
            bounds.update(level_bounds, context);

            bar.cut_top(context.layout_size * 0.5);
        }

        // Timeline
        // {
        //     let mut bar = columns[2];
        //     let timeline = bar.cut_top(context.font_size);
        //     let title = context.state.get_root_or(|| TextWidget::new("Timeline"));
        //     title.update(timeline, context);

        //     let mut config = editor.config.clone();
        //     let value_height = context.font_size * 1.2;
        //     let spacing = context.font_size * 0.3;

        //     let shift_scroll = bar.cut_top(value_height);
        //     bar.cut_top(spacing);
        //     let value = context
        //         .state
        //         .get_root_or(|| ToggleWidget::new("Shift Precision"));
        //     value.update_state(
        //         shift_scroll,
        //         context,
        //         &mut config.timeline.hold_to_scroll_slow,
        //     );

        //     let alt_scroll = bar.cut_top(value_height);
        //     bar.cut_top(spacing);
        //     let value = context.state.get_root_or(|| {
        //         BeatValueWidget::new(
        //             "Alt scroll",
        //             BeatTime::WHOLE * 16,
        //             BeatTime::WHOLE * 4..=BeatTime::WHOLE * 64,
        //             BeatTime::WHOLE,
        //         )
        //     });
        //     value.update(alt_scroll, context, &mut config.timeline.fast_speed);

        //     actions.push(EditorAction::SetConfig(config).into());
        // }
    }
}
