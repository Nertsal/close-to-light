use super::*;

pub struct EditorEditUi {
    event_mode: NewEventMode,
}

enum NewEventMode {
    Idle,
    Light,
    Vfx,
}

impl EditorEditUi {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            event_mode: NewEventMode::Idle,
        }
    }
}

struct LayoutHelper<'a> {
    editor: &'a Editor,
    level_editor: &'a LevelEditor,

    spacing: f32,
    title_size: f32,
    button_height: f32,
    delete_width: f32,
    value_height: f32,
}

impl EditorEditUi {
    pub fn layout(
        &mut self,
        position: Aabb2<f32>,
        game_position: Aabb2<f32>,
        context: &UiContext,
        editor: &Editor,
        actions: &mut Vec<EditorStateAction>,
    ) {
        let Some(level_editor) = &editor.level_edit else {
            let size = vec2(15.0, 1.0) * context.font_size;
            let warn = position
                .align_aabb(size, vec2(0.5, 1.0))
                .translate(vec2(0.0, -3.0 * size.y));

            let text = context
                .state
                .get_root_or(|| TextWidget::new("Select or create a difficulty in the Config tab"));
            text.update(warn, context);
            return;
        };

        let mut main = position;
        let font_size = context.font_size;
        let layout_size = context.layout_size;

        let mut bottom_bar = main.cut_bottom(game_position.min.y - 6.0 - main.min.y);

        let mut main = main
            .extend_symmetric(-vec2(1.0, 2.0) * layout_size)
            .extend_up(-layout_size);
        let mut left_bar = main.cut_left(layout_size * 7.0);
        let right_bar = main.cut_right(layout_size * 7.0);

        let spacing = layout_size * 0.25;
        let title_size = font_size * 1.3;
        let button_height = font_size * 1.3;
        let delete_width = font_size * 3.5;
        let value_height = font_size * 1.2;

        let tooltip = context.state.get_root_or(TooltipWidget::new);
        tooltip.visible = false;

        // Timeline
        {
            let timeline = bottom_bar.cut_top(font_size * 1.0);
            let linetime = context.state.get_root_or(TimelineWidget::new);
            linetime.update_time(
                level_editor.current_time.value,
                level_editor.current_time.target,
            );
            linetime.rescale(level_editor.timeline_zoom.current.as_f32());

            linetime.update(timeline, context, editor, level_editor, actions);

            // self.timeline.auto_scale(level_editor.level.last_beat());
        }

        let helper = LayoutHelper {
            editor,
            level_editor,

            spacing,
            title_size,
            button_height,
            delete_width,
            value_height,
        };

        // New Event
        let remaining = helper.layout_event(self, tooltip, left_bar, actions, context);
        left_bar.max.y = (left_bar.max.y - context.font_size * 7.0).min(remaining.max.y);

        // View
        left_bar = helper.layout_view(self, tooltip, left_bar, actions, context);

        // Placement
        left_bar = helper.layout_placement(self, tooltip, left_bar, actions, context);

        // Active selection
        helper.layout_selected(self, tooltip, right_bar, actions, context);

        let _ = left_bar;
        let _ = right_bar;
    }
}

impl LayoutHelper<'_> {
    /// New event - lights, vfx
    fn layout_event(
        &self,
        ui: &mut EditorEditUi,
        tooltip: &mut TooltipWidget,
        mut bar: Aabb2<f32>,
        actions: &mut Vec<EditorStateAction>,
        context: &UiContext,
    ) -> Aabb2<f32> {
        let event = bar
            .cut_top(self.title_size)
            .with_width(bar.width() / 2.0, 0.0);
        let text = context
            .state
            .get_root_or(|| TextWidget::new("Event").aligned(vec2(0.0, 0.5)));
        text.update(event, context);
        text.options.size = self.title_size;

        if !matches!(ui.event_mode, NewEventMode::Idle) {
            // Button to return to idle mode
            let back = event
                .with_width(event.height(), 1.0)
                .translate(vec2(event.height(), 0.0));
            let button = context.state.get_root_or(|| {
                IconButtonWidget::new_normal(context.context.assets.atlas.button_close())
            });
            button.update(back, context);
            if button.icon.state.mouse_left.clicked {
                ui.event_mode = NewEventMode::Idle;
            }
        }

        if self.level_editor.level_state.waypoints.is_some() {
            // Waypoints mode
            let waypoint = bar.cut_top(self.button_height);
            bar.cut_top(self.spacing);
            let button = context
                .state
                .get_root_or(|| ButtonWidget::new("Add waypoint"));
            button.update(waypoint, context);
            if button.text.state.mouse_left.clicked {
                actions.push(LevelAction::NewWaypoint.into());
            }

            tooltip.update(&button.text.state, "1", context);

            bar.cut_top(self.button_height);
            bar.cut_top(self.spacing);
        } else {
            match ui.event_mode {
                NewEventMode::Idle => {
                    self.layout_event_idle(ui, tooltip, &mut bar, actions, context)
                }
                NewEventMode::Light => {
                    self.layout_event_light(ui, tooltip, &mut bar, actions, context)
                }
                NewEventMode::Vfx => self.layout_event_vfx(ui, tooltip, &mut bar, actions, context),
            }
        }

        bar.cut_top(context.layout_size * 0.5);
        bar
    }

    /// Regular mode - new lights and effects
    fn layout_event_idle(
        &self,
        ui: &mut EditorEditUi,
        _tooltip: &mut TooltipWidget,
        bar: &mut Aabb2<f32>,
        actions: &mut Vec<EditorStateAction>,
        context: &UiContext,
    ) {
        let button_width = context.font_size * 4.5;

        let new_timing = bar
            .cut_top(self.button_height)
            .with_width(button_width, 0.0);
        let button = context
            .state
            .get_root_or(|| ButtonWidget::new("Timing (BPM)"));
        button.update(new_timing, context);
        if button.text.state.mouse_left.clicked {
            let time = self.level_editor.current_time.target;
            let beat_time = self.level_editor.level.timing.get_timing(time).beat_time;
            actions.push(LevelAction::TimingNew(time, beat_time).into());
        }

        let new_light = bar
            .cut_top(self.button_height)
            .with_width(button_width, 0.0);
        let button = context.state.get_root_or(|| ButtonWidget::new("Light"));
        button.update(new_light, context);
        if button.text.state.mouse_left.clicked {
            ui.event_mode = NewEventMode::Light;
        }

        let new_vfx = bar
            .cut_top(self.button_height)
            .with_width(button_width, 0.0);
        let button = context.state.get_root_or(|| ButtonWidget::new("VFX"));
        button.update(new_vfx, context);
        if button.text.state.mouse_left.clicked {
            ui.event_mode = NewEventMode::Vfx;
        }

        let new_shader = bar
            .cut_top(self.button_height)
            .with_width(button_width, 0.0);
        let button = context.state.get_root_or(|| ButtonWidget::new("Shader"));
        button.update(new_shader, context);
        if button.text.state.mouse_left.clicked {
            let time = self.level_editor.current_time.target;
            let beat_time = self.level_editor.level.timing.get_timing(time).beat_time;
            let shader = ShaderEvent {
                shader: "".into(),
                layer: ShaderLayer::Background,
                duration: seconds_to_time(beat_time),
            };
            actions.push(LevelAction::NewShader(time, shader).into());
        }
    }

    /// Light mode - select light shape
    fn layout_event_light(
        &self,
        _ui: &mut EditorEditUi,
        tooltip: &mut TooltipWidget,
        bar: &mut Aabb2<f32>,
        actions: &mut Vec<EditorStateAction>,
        context: &UiContext,
    ) {
        let new_light_width = context.font_size * 4.0;
        for (i, shape) in self.editor.config.shapes.iter().enumerate() {
            let new_shape = bar.cut_top(self.button_height).cut_left(new_light_width);
            bar.cut_top(self.spacing);
            let button = context.state.get_root_or(|| {
                ButtonWidget::new(match shape {
                    Shape::Circle { .. } => "Circle",
                    Shape::Line { .. } => "Line",
                    Shape::Rectangle { .. } => "Rectangle",
                })
            });
            button.update(new_shape, context);
            if button.text.state.mouse_left.clicked {
                actions.push(LevelAction::Shape(*shape).into());
            }
            tooltip.update(&button.text.state, format!("{}", i + 1), context);
        }
    }

    /// VFX mode - select vfx
    fn layout_event_vfx(
        &self,
        _ui: &mut EditorEditUi,
        _tooltip: &mut TooltipWidget,
        bar: &mut Aabb2<f32>,
        actions: &mut Vec<EditorStateAction>,
        context: &UiContext,
    ) {
        let button_width = context.font_size * 4.0;

        let new_rgb = bar.cut_top(self.button_height).cut_left(button_width);
        bar.cut_top(self.spacing);
        let button = context.state.get_root_or(|| ButtonWidget::new("RGB Split"));
        button.update(new_rgb, context);
        if button.text.state.mouse_left.clicked {
            actions.push(LevelAction::NewRgbSplit(TIME_IN_FLOAT_TIME).into());
        }

        let new_palette = bar.cut_top(self.button_height).cut_left(button_width);
        bar.cut_top(self.spacing);
        let button = context
            .state
            .get_root_or(|| ButtonWidget::new("Palette swap"));
        button.update(new_palette, context);
        if button.text.state.mouse_left.clicked {
            actions.push(LevelAction::NewPaletteSwap(TIME_IN_FLOAT_TIME / 2).into());
        }

        let new_palette = bar.cut_top(self.button_height).cut_left(button_width);
        bar.cut_top(self.spacing);
        let button = context
            .state
            .get_root_or(|| ButtonWidget::new("Camera shake"));
        button.update(new_palette, context);
        if button.text.state.mouse_left.clicked {
            actions.push(LevelAction::NewCameraShake(TIME_IN_FLOAT_TIME / 8).into());
        }
    }

    /// View configuration
    fn layout_view(
        &self,
        _ui: &mut EditorEditUi,
        tooltip: &mut TooltipWidget,
        mut bar: Aabb2<f32>,
        actions: &mut Vec<EditorStateAction>,
        context: &UiContext,
    ) -> Aabb2<f32> {
        let view = bar.cut_top(self.title_size);
        bar.cut_top(self.spacing);
        let text = context
            .state
            .get_root_or(|| TextWidget::new("View").aligned(vec2(0.0, 0.5)));
        text.update(view, context);
        text.options.size = self.title_size;

        let selected = bar.cut_top(context.font_size);
        bar.cut_top(self.spacing);
        let toggle = context
            .state
            .get_root_or(|| ToggleWidget::new("Only selected"));
        toggle.update(selected, context);
        if toggle.state.mouse_left.clicked {
            actions.push(EditorAction::ToggleShowOnlySelected.into());
        }
        toggle.checked = self.editor.show_only_selected;

        let dynamic = bar.cut_top(context.font_size);
        bar.cut_top(self.spacing);
        let toggle = context.state.get_root_or(|| ToggleWidget::new("Dynamic"));
        toggle.update(dynamic, context);
        if toggle.state.mouse_left.clicked {
            actions.push(EditorAction::ToggleDynamicVisual.into());
        }
        toggle.checked = self.editor.visualize_beat;
        tooltip.update(&toggle.state, "F", context);

        let grid = bar.cut_top(context.font_size);
        bar.cut_top(self.spacing);
        let toggle = context.state.get_root_or(|| ToggleWidget::new("Show grid"));
        toggle.update(grid, context);
        if toggle.state.mouse_left.clicked {
            actions.push(EditorAction::ToggleGrid.into());
        }
        toggle.checked = self.editor.render_options.show_grid;
        tooltip.update(&toggle.state, "C-~", context);

        // let waypoints = bar.cut_top(button_height);
        // bar.cut_top(spacing);
        // update!(self.view_waypoints, waypoints);
        // if self.view_waypoints.text.state.clicked {
        //     editor.view_waypoints();
        // }

        let zoom = bar.cut_top(self.value_height);
        bar.cut_top(self.spacing);
        let slider = context.state.get_root_or(|| {
            ValueWidget::new_range("Zoom", self.editor.view_zoom.target, 0.5..=2.0, 0.25)
        });
        {
            let mut view_zoom = self.editor.view_zoom.clone();
            slider.update_dynamic(zoom, context, &mut view_zoom);
            actions.push(EditorAction::SetViewZoom(Change::Set(view_zoom.target)).into());
        }
        context.update_focus(slider.state.hovered);

        bar.cut_top(context.layout_size * 1.5);
        bar
    }

    /// Placement configuration
    fn layout_placement(
        &self,
        _ui: &mut EditorEditUi,
        tooltip: &mut TooltipWidget,
        mut bar: Aabb2<f32>,
        actions: &mut Vec<EditorStateAction>,
        context: &UiContext,
    ) -> Aabb2<f32> {
        let placement = bar.cut_top(self.title_size);
        let text = context
            .state
            .get_root_or(|| TextWidget::new("Placement").aligned(vec2(0.0, 0.5)));
        text.update(placement, context);
        text.options.size = self.title_size;

        let grid_snap = bar.cut_top(context.font_size);
        bar.cut_top(self.spacing);
        let button = context
            .state
            .get_root_or(|| ToggleWidget::new("Snap to grid"));
        button.update(grid_snap, context);
        if button.state.mouse_left.clicked {
            actions.push(EditorAction::ToggleGridSnap.into());
        }
        button.checked = self.editor.snap_to_grid;
        tooltip.update(&button.state, "~", context);

        let grid_size = bar.cut_top(self.value_height);
        bar.cut_top(self.spacing);
        {
            let mut value = 10.0 / self.editor.grid.cell_size.as_f32();
            let slider = context
                .state
                .get_root_or(|| ValueWidget::new_range("Grid size", value, 2.0..=32.0, 1.0));
            slider.update(grid_size, context, &mut value);
            actions.push(EditorAction::SetGridSize(r32(10.0 / value)).into());
            context.update_focus(slider.state.hovered);
        }

        bar.cut_top(context.layout_size * 1.5);
        bar
    }

    /// View selected event
    fn layout_selected(
        &self,
        _ui: &mut EditorEditUi,
        tooltip: &mut TooltipWidget,
        mut bar: Aabb2<f32>,
        actions: &mut Vec<EditorStateAction>,
        context: &UiContext,
    ) {
        match &self.level_editor.selection {
            Selection::Empty => {}
            Selection::Lights(selected) => {
                self.layout_selected_lights(selected, tooltip, &mut bar, actions, context);
            }
            Selection::Waypoints(light_id, selected) => {
                self.layout_selected_lights(&[*light_id], tooltip, &mut bar, actions, context);
                self.layout_selected_waypoints(
                    *light_id, selected, tooltip, &mut bar, actions, context,
                );
            }
            &Selection::Event(event_i) => {
                self.layout_selected_event(event_i, tooltip, &mut bar, actions, context);
            }
            &Selection::Timing(idx) => {
                self.layout_selected_timing(idx, tooltip, &mut bar, actions, context);
            }
        }
    }

    fn layout_selected_lights(
        &self,
        selected: &[LightId],
        tooltip: &mut TooltipWidget,
        bar: &mut Aabb2<f32>,
        actions: &mut Vec<EditorStateAction>,
        context: &UiContext,
    ) {
        let light_pos = bar.cut_top(self.title_size);
        let text = context
            .state
            .get_root_or(|| TextWidget::new("Light").aligned(vec2(0.0, 0.5)));
        text.update(light_pos, context);
        text.options.size = self.title_size;
        let delete = bar.cut_top(self.button_height).cut_left(self.delete_width);
        let button = context
            .state
            .get_root_or(|| ButtonWidget::new("Delete").color(ThemeColor::Danger));
        button.update(delete, context);
        tooltip.update(&button.text.state, "X", context);
        if button.text.state.mouse_left.clicked {
            actions.push(
                LevelAction::list(selected.iter().copied().map(LevelAction::DeleteLight)).into(),
            );
        }
        match selected.len().cmp(&1) {
            std::cmp::Ordering::Greater => {
                // More than 1 selected light
                let danger_pos = bar.cut_top(self.button_height);
                bar.cut_top(self.spacing);
                let button = context
                    .state
                    .get_root_or(|| ButtonWidget::new("Toggle Danger").color(ThemeColor::Danger));
                button.update(danger_pos, context);
                if button.text.state.mouse_left.clicked {
                    actions.push(
                        LevelAction::list(selected.iter().copied().map(LevelAction::ToggleDanger))
                            .into(),
                    );
                }
                tooltip.update(&button.text.state, "D", context);
            }
            std::cmp::Ordering::Equal => {
                // Exactly 1 light selected
                let light_id = *selected.first().unwrap();
                if let Some(event) = self.level_editor.level.events.get(light_id.event)
                    && let Event::Light(light) = &event.event
                {
                    let danger_pos = bar.cut_top(self.button_height);
                    bar.cut_top(self.spacing);
                    let button = context
                        .state
                        .get_root_or(|| ToggleWidget::new("Danger").color(ThemeColor::Danger));
                    button.update(danger_pos, context);
                    if button.state.mouse_left.clicked {
                        actions.push(LevelAction::ToggleDanger(light_id).into());
                    }
                    button.checked = light.danger;
                    tooltip.update(&button.state, "D", context);

                    let timing = &self.level_editor.level.timing;

                    {
                        let timing_point = timing.get_timing(event.time);
                        let fade_in = bar.cut_top(self.value_height);
                        bar.cut_top(self.spacing);
                        let mut fade = BeatTime::from_beats_float(
                            time_to_seconds(light.movement.get_fade_in()) / timing_point.beat_time,
                        );
                        let slider = context.state.get_root_or(|| {
                            BeatValueWidget::new(
                                "Fade in",
                                fade,
                                BeatTime::ZERO..=BeatTime::WHOLE * 10,
                                self.level_editor.beat_snap,
                            )
                        });
                        slider.scroll_by = self.level_editor.beat_snap;
                        if slider.update(fade_in, context, &mut fade) {
                            actions.push(
                                LevelAction::ChangeFadeIn(
                                    light_id,
                                    Change::Set(fade.as_time(timing_point.beat_time)),
                                )
                                .into(),
                            );
                        }
                        if slider.control_state.mouse_left.just_released {
                            actions.push(
                                LevelAction::FlushChanges(Some(HistoryLabel::FadeIn(light_id)))
                                    .into(),
                            );
                        }
                        context.update_focus(slider.state.hovered);
                    }

                    {
                        let to_time =
                            event.time + light.movement.get_fade_in() + light.movement.duration()
                                - light.movement.get_fade_out();
                        let timing_point = timing.get_timing(to_time);
                        let fade_out = bar.cut_top(self.value_height);
                        bar.cut_top(self.spacing);
                        let mut fade = BeatTime::from_beats_float(
                            time_to_seconds(light.movement.get_fade_out()) / timing_point.beat_time,
                        );
                        let slider = context.state.get_root_or(|| {
                            BeatValueWidget::new(
                                "Fade out",
                                fade,
                                BeatTime::ZERO..=BeatTime::WHOLE * 10,
                                self.level_editor.beat_snap,
                            )
                        });
                        slider.scroll_by = self.level_editor.beat_snap;
                        if slider.update(fade_out, context, &mut fade) {
                            actions.push(
                                LevelAction::ChangeFadeOut(
                                    light_id,
                                    Change::Set(fade.as_time(timing_point.beat_time)),
                                )
                                .into(),
                            );
                        }
                        if slider.control_state.mouse_left.just_released {
                            actions.push(
                                LevelAction::FlushChanges(Some(HistoryLabel::FadeOut(light_id)))
                                    .into(),
                            );
                        }
                        context.update_focus(slider.state.hovered);
                    }

                    bar.cut_top(context.layout_size * 1.5);

                    let waypoints = bar.cut_top(self.title_size);
                    let button = context.state.get_root_or(|| ToggleWidget::new("Waypoints"));
                    button.update(waypoints, context);
                    button.text.options.size = self.title_size;
                    button.checked =
                        matches!(self.level_editor.state, EditingState::Waypoints { .. });
                    if button.state.mouse_left.clicked {
                        actions.push(LevelAction::ToggleWaypointsView.into());
                    }

                    bar.cut_top(self.spacing);
                }
            }
            std::cmp::Ordering::Less => {}
        }
    }

    fn layout_selected_waypoints(
        &self,
        light_id: LightId,
        selected: &[WaypointId],
        tooltip: &mut TooltipWidget,
        bar: &mut Aabb2<f32>,
        actions: &mut Vec<EditorStateAction>,
        context: &UiContext,
    ) {
        let Some(event) = self.level_editor.level.events.get(light_id.event) else {
            return;
        };
        let Event::Light(light) = &event.event else {
            return;
        };

        match selected.len().cmp(&1) {
            std::cmp::Ordering::Greater => {
                // More than 1 selected waypoint
                // TODO
            }
            std::cmp::Ordering::Equal => {
                // Exactly 1 waypoint selected
                let selected = *selected.first().unwrap();
                let frames = light.movement.waypoints.len();
                let Some(frame) = light.movement.get_frame(selected) else {
                    return;
                };
                // Waypoint
                let mut current = bar.cut_top(context.font_size);
                let mut current = current.cut_left(context.font_size * 3.0);

                // Previous waypoint
                let prev = current
                    .cut_left(current.height() * 0.6)
                    .zero_size(vec2(0.5, 0.5))
                    .extend_uniform(context.font_size * 0.7);
                if let WaypointId::Initial = selected {
                    let button = context.state.get_root_or(|| {
                        IconWidget::new(context.context.assets.atlas.button_prev_hollow())
                    });
                    button.update(prev, context);
                } else {
                    let button = context.state.get_root_or(|| {
                        IconButtonWidget::new_normal(context.context.assets.atlas.button_prev())
                    });
                    button.update(prev, context);
                    if button.icon.state.mouse_left.clicked
                        && let Some(id) = selected.prev(frames)
                    {
                        actions.push(
                            LevelAction::SelectWaypoint(SelectMode::Set, light_id, vec![id], true)
                                .into(),
                        );
                    }
                };

                let i = match selected {
                    WaypointId::Initial => 0,
                    WaypointId::Frame(i) => i + 1,
                    WaypointId::Last => frames + 1,
                };

                // Next waypoint
                let next = current
                    .cut_right(current.height() * 0.6)
                    .zero_size(vec2(0.5, 0.5))
                    .extend_uniform(context.font_size * 0.7);
                if i > frames {
                    let button = context.state.get_root_or(|| {
                        IconWidget::new(context.context.assets.atlas.button_next_hollow())
                    });
                    button.update(next, context);
                } else {
                    let button = context.state.get_root_or(|| {
                        IconButtonWidget::new_normal(context.context.assets.atlas.button_next())
                    });
                    button.update(next, context);
                    if button.icon.state.mouse_left.clicked
                        && let Some(next) = selected.next(frames)
                    {
                        actions.push(
                            LevelAction::SelectWaypoint(
                                SelectMode::Set,
                                light_id,
                                vec![next],
                                true,
                            )
                            .into(),
                        );
                    }
                }

                // Current waypoint
                let text = context.state.get_root_or(|| TextWidget::new("0"));
                text.update(current, context);
                text.text = (i + 1).to_string().into();

                // Delete
                let delete = bar.cut_top(self.button_height).cut_left(self.delete_width);
                let button = context
                    .state
                    .get_root_or(|| ButtonWidget::new("Delete").color(ThemeColor::Danger));
                button.update(delete, context);
                if button.text.state.mouse_left.clicked {
                    actions.push(LevelAction::DeleteWaypoint(light_id, selected).into());
                }
                tooltip.update(&button.text.state, "X", context);

                let hollow_pos = bar.cut_top(self.value_height);
                bar.cut_top(self.spacing);
                let mut hollow = frame.hollow;
                let value = context.state.get_root_or(|| {
                    ValueWidget::new(
                        "Hollow",
                        hollow,
                        ValueControl::Slider {
                            min: r32(-1.0),
                            max: r32(1.0),
                        },
                        r32(0.05),
                    )
                });
                value.update(hollow_pos, context, &mut hollow);
                actions.push(
                    LevelAction::ChangeHollow(light_id, selected, Change::Set(hollow)).into(),
                );

                let scale = bar.cut_top(self.value_height);
                bar.cut_top(self.spacing);
                let mut value = frame.scale.as_f32();
                let slider = context
                    .state
                    .get_root_or(|| ValueWidget::new_range("Scale", value, 0.0..=20.0, 0.25));
                if slider.update(scale, context, &mut value) {
                    actions.push(
                        LevelAction::ScaleWaypoint(light_id, selected, Change::Set(r32(value)))
                            .into(),
                    );
                }
                if slider.control_state.mouse_left.just_released {
                    actions.push(
                        LevelAction::FlushChanges(Some(HistoryLabel::Scale(light_id, selected)))
                            .into(),
                    );
                }
                context.update_focus(slider.state.hovered);

                let angle = bar.cut_top(self.value_height);
                bar.cut_top(self.spacing);
                let mut value = frame.rotation.as_degrees().as_f32();
                let slider = context
                    .state
                    .get_root_or(|| ValueWidget::new_circle("Angle", value, 360.0, 15.0));
                if slider.update(angle, context, &mut value) {
                    actions.push(
                        LevelAction::RotateWaypointAround(
                            light_id,
                            selected,
                            frame.translation,
                            Change::Set(Angle::from_degrees(r32(value.round()))),
                        )
                        .into(),
                    );
                }
                if slider.control_state.mouse_left.just_released {
                    actions.push(
                        LevelAction::FlushChanges(Some(HistoryLabel::Rotate(light_id, selected)))
                            .into(),
                    );
                }
                context.update_focus(slider.state.hovered);
                tooltip.update(&slider.state, "Q/E", context);

                // Interpolation
                let curve = bar.cut_top(self.button_height);
                bar.cut_top(self.spacing);
                let interpolation = bar.cut_top(self.button_height);
                bar.cut_top(self.spacing);

                if let Some((mut move_interpolation, mut curve_interpolation)) =
                    light.movement.get_interpolation(selected)
                {
                    let waypoint_curve = context.state.get_root_or(|| {
                        DropdownWidget::new(
                            "Curve",
                            0,
                            [
                                ("Continue", None),
                                ("Linear", Some(TrajectoryInterpolation::Linear)),
                                (
                                    "Spline",
                                    Some(TrajectoryInterpolation::Spline { tension: r32(0.1) }),
                                ),
                                ("Bezier", Some(TrajectoryInterpolation::Bezier)),
                            ],
                        )
                    });

                    let waypoint_interpolation = context.state.get_root_or(|| {
                        DropdownWidget::new(
                            "Interpolation",
                            0,
                            [
                                ("Linear", MoveInterpolation::Linear),
                                ("Smoothstep", MoveInterpolation::Smoothstep),
                                ("EaseIn", MoveInterpolation::EaseIn),
                                ("EaseOut", MoveInterpolation::EaseOut),
                            ],
                        )
                    });

                    waypoint_curve.update(curve, context, &mut curve_interpolation);
                    actions.push(
                        LevelAction::SetWaypointCurve(light_id, selected, curve_interpolation)
                            .into(),
                    );

                    waypoint_interpolation.update(interpolation, context, &mut move_interpolation);
                    actions.push(
                        LevelAction::SetWaypointInterpolation(
                            light_id,
                            selected,
                            move_interpolation,
                        )
                        .into(),
                    );
                }

                bar.cut_top(self.spacing);
            }
            std::cmp::Ordering::Less => {}
        }
    }

    fn event_title_delete(
        &self,
        event_idx: EditorEventIdx,
        bar: &mut Aabb2<f32>,
        title: &str,
        tooltip: &mut TooltipWidget,
        actions: &mut Vec<EditorStateAction>,
        context: &UiContext,
    ) {
        let light_pos = bar.cut_top(self.title_size);
        let text = context
            .state
            .get_root_or(|| TextWidget::new("").aligned(vec2(0.0, 0.5)));
        text.text = title.into();
        text.update(light_pos, context);
        text.options.size = self.title_size;

        let delete = bar.cut_top(self.button_height).cut_left(self.delete_width);
        let button = context
            .state
            .get_root_or(|| ButtonWidget::new("Delete").color(ThemeColor::Danger));
        button.update(delete, context);
        tooltip.update(&button.text.state, "X", context);
        if button.text.state.mouse_left.clicked {
            actions.push(LevelAction::DeleteEvent(event_idx).into());
        }
    }

    fn layout_selected_event(
        &self,
        event_i: usize,
        tooltip: &mut TooltipWidget,
        bar: &mut Aabb2<f32>,
        actions: &mut Vec<EditorStateAction>,
        context: &UiContext,
    ) {
        let Some(event) = self.level_editor.level.events.get(event_i) else {
            return;
        };

        let timing = &self.level_editor.level.timing;
        let timing_point = timing.get_timing(event.time);

        match &event.event {
            Event::Light(_) => {}
            Event::Effect(effect) => match *effect {
                EffectEvent::PaletteSwap(duration) => {
                    self.event_title_delete(
                        EditorEventIdx::Event(event_i),
                        bar,
                        "Palette Swap",
                        tooltip,
                        actions,
                        context,
                    );

                    let duration_pos = bar.cut_top(self.value_height);
                    let mut duration = BeatTime::from_beats_float(
                        time_to_seconds(duration) / timing_point.beat_time,
                    );
                    let slider = context.state.get_root_or(|| {
                        BeatValueWidget::new(
                            "Duration",
                            duration,
                            BeatTime::ZERO..=BeatTime::WHOLE * 10,
                            self.level_editor.beat_snap,
                        )
                    });
                    slider.scroll_by = self.level_editor.beat_snap;
                    if slider.update(duration_pos, context, &mut duration) {
                        actions.push(
                            LevelAction::ChangeEffectDuration(
                                event_i,
                                Change::Set(duration.as_time(timing_point.beat_time)),
                            )
                            .into(),
                        );
                    }
                    if slider.control_state.mouse_left.just_released {
                        actions.push(
                            LevelAction::FlushChanges(Some(HistoryLabel::EventDuration(event_i)))
                                .into(),
                        );
                    }
                }
                EffectEvent::RgbSplit(duration) => {
                    self.event_title_delete(
                        EditorEventIdx::Event(event_i),
                        bar,
                        "RGB Split",
                        tooltip,
                        actions,
                        context,
                    );

                    let duration_pos = bar.cut_top(self.value_height);
                    let mut duration = BeatTime::from_beats_float(
                        time_to_seconds(duration) / timing_point.beat_time,
                    );
                    let slider = context.state.get_root_or(|| {
                        BeatValueWidget::new(
                            "Duration",
                            duration,
                            BeatTime::ZERO..=BeatTime::WHOLE * 10,
                            self.level_editor.beat_snap,
                        )
                    });
                    slider.scroll_by = self.level_editor.beat_snap;
                    if slider.update(duration_pos, context, &mut duration) {
                        actions.push(
                            LevelAction::ChangeEffectDuration(
                                event_i,
                                Change::Set(duration.as_time(timing_point.beat_time)),
                            )
                            .into(),
                        );
                    }
                    if slider.control_state.mouse_left.just_released {
                        actions.push(
                            LevelAction::FlushChanges(Some(HistoryLabel::EventDuration(event_i)))
                                .into(),
                        );
                    }
                }
                EffectEvent::CameraShake(duration, intensity) => {
                    self.event_title_delete(
                        EditorEventIdx::Event(event_i),
                        bar,
                        "Camera Shake",
                        tooltip,
                        actions,
                        context,
                    );

                    let duration_pos = bar.cut_top(self.value_height);
                    let mut duration = BeatTime::from_beats_float(
                        time_to_seconds(duration) / timing_point.beat_time,
                    );
                    let slider = context.state.get_root_or(|| {
                        BeatValueWidget::new(
                            "Duration",
                            duration,
                            BeatTime::ZERO..=BeatTime::WHOLE * 10,
                            self.level_editor.beat_snap,
                        )
                    });
                    slider.scroll_by = self.level_editor.beat_snap;
                    if slider.update(duration_pos, context, &mut duration) {
                        actions.push(
                            LevelAction::ChangeEffectDuration(
                                event_i,
                                Change::Set(duration.as_time(timing_point.beat_time)),
                            )
                            .into(),
                        );
                    }
                    if slider.control_state.mouse_left.just_released {
                        actions.push(
                            LevelAction::FlushChanges(Some(HistoryLabel::EventDuration(event_i)))
                                .into(),
                        );
                    }

                    let intensity_pos = bar.cut_top(self.value_height);
                    let scale = r32(2.0);
                    let mut intensity = intensity * scale;
                    let slider = context.state.get_root_or(|| {
                        ValueWidget::new(
                            "Intensity",
                            intensity,
                            ValueControl::Slider {
                                min: R32::ZERO,
                                max: r32(1.0),
                            },
                            r32(0.05),
                        )
                    });
                    if slider.update(intensity_pos, context, &mut intensity) {
                        actions.push(
                            LevelAction::ChangeCameraShakeIntensity(
                                event_i,
                                Change::Set(intensity / scale),
                            )
                            .into(),
                        );
                    }
                    if slider.control_state.mouse_left.just_released {
                        actions.push(
                            LevelAction::FlushChanges(Some(HistoryLabel::CameraShakeIntensity(
                                event_i,
                            )))
                            .into(),
                        );
                    }
                }
            },
            Event::Shader(old_shader) => {
                let mut shader = old_shader.clone();
                self.event_title_delete(
                    EditorEventIdx::Event(event_i),
                    bar,
                    "Custom Shader",
                    tooltip,
                    actions,
                    context,
                );

                let name_pos = bar.cut_top(self.value_height);
                let dropdown = context.state.get_root_or(|| {
                    DropdownWidget::new("Shader", 0, [("<name>", Name::from("<none>"))])
                });
                dropdown.update_options(
                    self.editor
                        .level_assets
                        .shaders
                        .keys()
                        .map(|name| (name.clone(), name.clone())),
                );
                dropdown.update(name_pos, context, &mut shader.shader);

                let layer_pos = bar.cut_top(self.value_height);
                let dropdown = context.state.get_root_or(|| {
                    DropdownWidget::new(
                        "Layer",
                        0,
                        [
                            ("Background", ShaderLayer::Background),
                            ("Post (early)", ShaderLayer::PostProcessEarly),
                            ("Post (late)", ShaderLayer::PostProcessLate),
                        ],
                    )
                });
                dropdown.update(layer_pos, context, &mut shader.layer);

                if *old_shader != shader {
                    actions.push(LevelAction::UpdateShader(event_i, shader.clone()).into());
                }

                let duration_pos = bar.cut_top(self.value_height);
                let mut duration = BeatTime::from_beats_float(
                    time_to_seconds(shader.duration) / timing_point.beat_time,
                );
                let slider = context.state.get_root_or(|| {
                    BeatValueWidget::new(
                        "Duration",
                        duration,
                        BeatTime::ZERO..=BeatTime::WHOLE * 10,
                        self.level_editor.beat_snap,
                    )
                });
                slider.scroll_by = self.level_editor.beat_snap;
                if slider.update(duration_pos, context, &mut duration) {
                    actions.push(
                        LevelAction::ChangeShaderDuration(
                            event_i,
                            Change::Set(duration.as_time(timing_point.beat_time)),
                        )
                        .into(),
                    );
                }
                if slider.control_state.mouse_left.just_released {
                    actions.push(
                        LevelAction::FlushChanges(Some(HistoryLabel::EventDuration(event_i)))
                            .into(),
                    );
                }
            }
        }
    }

    fn layout_selected_timing(
        &self,
        timing_i: usize,
        tooltip: &mut TooltipWidget,
        bar: &mut Aabb2<f32>,
        actions: &mut Vec<EditorStateAction>,
        context: &UiContext,
    ) {
        let Some(timing) = self.level_editor.level.timing.points.get(timing_i) else {
            return;
        };

        self.event_title_delete(
            EditorEventIdx::Timing(timing_i),
            bar,
            "Timing Point",
            tooltip,
            actions,
            context,
        );

        let mut bpm_value = r32(60.0) / timing.beat_time;

        let bpm_pos = bar.cut_top(self.value_height);
        let bpm = context.state.get_root_or(|| {
            ValueWidget::new(
                "BPM",
                bpm_value,
                ValueControl::Slider {
                    min: r32(20.0),
                    max: r32(500.0),
                },
                r32(1.0),
            )
        });
        bpm.update(bpm_pos, context, &mut bpm_value);
        actions.push(LevelAction::TimingUpdate(timing_i, r32(60.0) / bpm_value).into());
    }
}
