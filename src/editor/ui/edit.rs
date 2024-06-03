use super::*;

pub struct EditorEditWidget {
    pub state: WidgetState,

    pub warn_select_level: TextWidget,

    pub new_event: TextWidget,
    // pub new_palette: ButtonWidget, // TODO: reimplement
    pub new_circle: ButtonWidget,
    pub new_line: ButtonWidget,

    pub view: TextWidget,
    pub show_only_selected: CheckboxWidget,
    pub visualize_beat: CheckboxWidget,
    pub show_grid: CheckboxWidget,
    pub view_zoom: ValueWidget<f32>,

    pub placement: TextWidget,
    pub snap_grid: CheckboxWidget,
    pub grid_size: ValueWidget<f32>,

    pub light: TextWidget,
    pub light_delete: ButtonWidget,
    pub light_danger: CheckboxWidget,
    pub light_fade_in: ValueWidget<Time>,
    pub light_fade_out: ValueWidget<Time>,

    pub waypoint: ButtonWidget,
    pub prev_waypoint: IconButtonWidget,
    pub current_waypoint: TextWidget,
    pub next_waypoint: IconButtonWidget,
    pub waypoint_delete: ButtonWidget,
    pub waypoint_scale: ValueWidget<f32>,
    /// Angle in degrees.
    pub waypoint_angle: ValueWidget<f32>,

    pub current_beat: TextWidget,
    pub timeline: TimelineWidget,
}

impl EditorEditWidget {
    pub fn new(context: Context) -> Self {
        let assets = &context.assets;
        Self {
            state: WidgetState::new(),

            warn_select_level: TextWidget::new("Select or create a difficulty in the Config tab"),

            new_event: TextWidget::new("Event"),
            // new_palette: ButtonWidget::new("Palette Swap"),
            new_circle: ButtonWidget::new("Circle"),
            new_line: ButtonWidget::new("Line"),

            view: TextWidget::new("View"),
            show_only_selected: CheckboxWidget::new("Only selected"),
            visualize_beat: CheckboxWidget::new("Dynamic"),
            show_grid: CheckboxWidget::new("Grid"),
            view_zoom: ValueWidget::new("Zoom: ", 1.0, 0.5..=2.0, 0.25),

            placement: TextWidget::new("Placement"),
            snap_grid: CheckboxWidget::new("Grid snap"),
            grid_size: ValueWidget::new("Grid size", 16.0, 2.0..=32.0, 1.0),

            light: TextWidget::new("Light"),
            light_delete: ButtonWidget::new("delete"),
            light_danger: CheckboxWidget::new("Danger"),
            light_fade_in: ValueWidget::new("Fade in", r32(1.0), r32(0.25)..=r32(25.0), r32(0.25)),
            light_fade_out: ValueWidget::new(
                "Fade out",
                r32(1.0),
                r32(0.25)..=r32(25.0),
                r32(0.25),
            ),

            waypoint: ButtonWidget::new("Waypoints"),
            prev_waypoint: IconButtonWidget::new_normal(&assets.sprites.arrow_left),
            current_waypoint: TextWidget::new("0"),
            next_waypoint: IconButtonWidget::new_normal(&assets.sprites.arrow_right),
            waypoint_delete: ButtonWidget::new("delete"),
            waypoint_scale: ValueWidget::new("Scale", 1.0, 0.25..=2.0, 0.25),
            waypoint_angle: ValueWidget::new("Angle", 0.0, 0.0..=360.0, 15.0).wrapping(),

            current_beat: default(),
            timeline: TimelineWidget::new(context.clone()),
        }
    }
}

impl StatefulWidget for EditorEditWidget {
    type State = Editor;

    fn state_mut(&mut self) -> &mut WidgetState {
        &mut self.state
    }

    fn update(&mut self, position: Aabb2<f32>, context: &mut UiContext, state: &mut Self::State) {
        let editor = state;
        let Some(level_editor) = &mut editor.level_edit else {
            let size = vec2(15.0, 1.0) * context.font_size;
            let warn = position
                .align_aabb(size, vec2(0.5, 1.0))
                .translate(vec2(0.0, -3.0 * size.y));
            self.warn_select_level.show();
            self.warn_select_level.update(warn, context);

            return;
        };

        self.warn_select_level.hide();

        let mut main = position;
        let font_size = context.font_size;
        let layout_size = context.layout_size;

        macro_rules! update {
            ($widget:expr, $position:expr) => {{
                $widget.update($position, context);
            }};
            ($widget:expr, $position:expr, $state:expr) => {{
                $widget.update($position, context, $state);
            }};
        }

        let bottom_bar = main.cut_bottom(layout_size * 3.0);
        let mut bottom_bar = bottom_bar.extend_symmetric(-vec2(5.0, 0.0) * layout_size);

        let mut main = main
            .extend_symmetric(-vec2(1.0, 2.0) * layout_size)
            .extend_up(-layout_size);
        let mut left_bar = main.cut_left(font_size * 5.0);
        let mut right_bar = main.cut_right(font_size * 5.0);

        let spacing = layout_size * 0.25;
        let title_size = font_size * 1.3;
        let button_height = font_size * 1.2;

        {
            let mut bar = left_bar;

            let event = bar.cut_top(title_size);
            update!(self.new_event, event);
            self.new_event.options.size = title_size;

            // let palette = bar.cut_top(button_height);
            // bar.cut_top(spacing);
            // update!(self.new_palette, palette);
            // if self.new_palette.text.state.clicked {
            //     level_editor.palette_swap();
            // }

            let circle = bar.cut_top(button_height);
            bar.cut_top(spacing);
            update!(self.new_circle, circle);
            if self.new_circle.text.state.clicked {
                level_editor.new_light_circle();
            }

            let line = bar.cut_top(button_height);
            bar.cut_top(spacing);
            update!(self.new_line, line);
            if self.new_line.text.state.clicked {
                level_editor.new_light_line();
            }

            bar.cut_top(layout_size * 1.5);

            let view = bar.cut_top(title_size);
            bar.cut_top(spacing);
            update!(self.view, view);
            self.view.options.size = title_size;

            let selected = bar.cut_top(font_size);
            bar.cut_top(spacing);
            update!(self.show_only_selected, selected);
            if self.show_only_selected.state.clicked {
                editor.show_only_selected = !editor.show_only_selected;
            }
            self.show_only_selected.checked = editor.show_only_selected;

            let dynamic = bar.cut_top(font_size);
            bar.cut_top(spacing);
            update!(self.visualize_beat, dynamic);
            if self.visualize_beat.state.clicked {
                editor.visualize_beat = !editor.visualize_beat;
            }
            self.visualize_beat.checked = editor.visualize_beat;

            let grid = bar.cut_top(font_size);
            bar.cut_top(spacing);
            update!(self.show_grid, grid);
            if self.show_grid.state.clicked {
                editor.render_options.show_grid = !editor.render_options.show_grid;
            }
            self.show_grid.checked = editor.render_options.show_grid;

            // let waypoints = bar.cut_top(button_height);
            // bar.cut_top(spacing);
            // update!(self.view_waypoints, waypoints);
            // if self.view_waypoints.text.state.clicked {
            //     editor.view_waypoints();
            // }

            let zoom = bar.cut_top(font_size);
            bar.cut_top(spacing);
            update!(self.view_zoom, zoom, &mut editor.view_zoom);
            context.update_focus(self.view_zoom.state.hovered);

            bar.cut_top(layout_size * 1.5);
            left_bar = bar;
        }

        {
            // Spacing
            let mut bar = left_bar;

            let placement = bar.cut_top(title_size);
            update!(self.placement, placement);
            self.placement.options.size = title_size;

            let grid_snap = bar.cut_top(button_height);
            bar.cut_top(spacing);
            update!(self.snap_grid, grid_snap);
            if self.snap_grid.state.clicked {
                editor.snap_to_grid = !editor.snap_to_grid;
            }
            self.snap_grid.checked = editor.snap_to_grid;

            let grid_size = bar.cut_top(button_height);
            bar.cut_top(spacing);
            let mut value = 10.0 / editor.grid_size.as_f32();
            update!(self.grid_size, grid_size, &mut value);
            editor.grid_size = r32(10.0 / value);
            context.update_focus(self.grid_size.state.hovered);

            bar.cut_top(layout_size * 1.5);
            // left_bar = bar;
        }

        {
            // Light
            let selected = level_editor
                .selected_light
                .and_then(|i| level_editor.level.events.get_mut(i.event))
                .filter(|event| matches!(event.event, Event::Light(_)));

            match selected {
                None => {
                    self.light.hide();
                    self.light_delete.hide();
                    self.light_danger.hide();
                    self.light_fade_in.hide();
                    self.light_fade_out.hide();
                    self.waypoint.hide();
                }
                Some(event) => {
                    if let Event::Light(light) = &mut event.event {
                        let light = &mut light.light;

                        self.light.show();
                        self.light_delete.show();
                        self.light_danger.show();
                        self.light_fade_in.show();
                        self.light_fade_out.show();
                        self.waypoint.show();

                        let mut bar = right_bar;

                        let light_pos = bar.cut_top(title_size);
                        update!(self.light, light_pos);
                        self.light.options.size = title_size;

                        let delete = bar.cut_top(button_height);
                        self.light_delete.update(delete, context);
                        // NOTE: click action delayed because level_editor is borrowed

                        let danger_pos = bar.cut_top(button_height);
                        bar.cut_top(spacing);
                        update!(self.light_danger, danger_pos);
                        if self.light_danger.state.clicked {
                            light.danger = !light.danger;
                        }
                        self.light_danger.checked = light.danger;

                        let fade_in = bar.cut_top(button_height);
                        bar.cut_top(spacing);
                        let mut fade = light.movement.fade_in;
                        let from = fade;
                        update!(self.light_fade_in, fade_in, &mut fade);
                        light.movement.change_fade_in(fade);
                        context.update_focus(self.light_fade_in.state.hovered);
                        event.beat -= light.movement.fade_in - from;

                        let fade_out = bar.cut_top(button_height);
                        bar.cut_top(spacing);
                        let mut fade = light.movement.fade_out;
                        update!(self.light_fade_out, fade_out, &mut fade);
                        light.movement.change_fade_out(fade);
                        context.update_focus(self.light_fade_out.state.hovered);

                        bar.cut_top(layout_size * 1.5);

                        let waypoints = bar.cut_top(button_height);
                        update!(self.waypoint, waypoints);
                        if self.waypoint.text.state.clicked {
                            level_editor.view_waypoints();
                        }

                        bar.cut_top(spacing);
                        right_bar = bar;

                        // Delayed actions
                        if self.light_delete.text.state.clicked {
                            level_editor.delete_light_selected();
                        }
                    }
                }
            }
        }

        let mut waypoint = false;
        if let Some(waypoints) = &mut level_editor.level_state.waypoints {
            if let Some(selected) = waypoints.selected {
                if let Some(event) = level_editor.level.events.get_mut(waypoints.event) {
                    if let Event::Light(light) = &mut event.event {
                        let frames = light.light.movement.key_frames.len();
                        if let Some(frame) = light.light.movement.get_frame_mut(selected) {
                            // Waypoint
                            waypoint = true;
                            self.prev_waypoint.show();
                            self.next_waypoint.show();
                            self.current_waypoint.show();
                            self.waypoint_delete.show();
                            self.waypoint_scale.show();
                            self.waypoint_angle.show();

                            let mut bar = right_bar;

                            let mut current = bar.cut_top(button_height);

                            if let WaypointId::Initial = selected {
                                self.prev_waypoint.hide();
                            } else {
                                self.prev_waypoint.show();
                            }
                            let prev = current.cut_left(current.height());
                            self.prev_waypoint.update(prev, context);
                            if self.prev_waypoint.state.clicked {
                                if let Some(id) = selected.prev() {
                                    waypoints.selected = Some(id);
                                }
                            }

                            let i = match selected {
                                WaypointId::Initial => 0,
                                WaypointId::Frame(i) => i + 1,
                            };

                            if i >= frames {
                                self.next_waypoint.hide();
                            } else {
                                self.next_waypoint.show();
                            }
                            let next = current.cut_right(current.height());
                            self.next_waypoint.update(next, context);
                            if self.next_waypoint.state.clicked {
                                waypoints.selected = Some(selected.next());
                            }

                            self.current_waypoint.update(current, context);
                            self.current_waypoint.text = (i + 1).to_string().into();

                            let delete = bar.cut_top(button_height);
                            self.waypoint_delete.update(delete, context);
                            // NOTE: click action delayed because level_editor is borrowed

                            let scale = bar.cut_top(button_height);
                            bar.cut_top(spacing);
                            let mut value = frame.scale.as_f32();
                            update!(self.waypoint_scale, scale, &mut value);
                            frame.scale = r32(value);
                            context.update_focus(self.waypoint_scale.state.hovered);

                            let angle = bar.cut_top(button_height);
                            bar.cut_top(spacing);
                            let mut value = frame.rotation.as_degrees().as_f32();
                            update!(self.waypoint_angle, angle, &mut value);
                            frame.rotation = Angle::from_degrees(r32(value.round()));
                            context.update_focus(self.waypoint_angle.state.hovered);

                            // Delayed actions
                            if self.waypoint_delete.text.state.clicked {
                                level_editor.delete_waypoint_selected();
                            }
                        }
                    }
                }
            }
        }
        if !waypoint {
            self.prev_waypoint.hide();
            self.next_waypoint.hide();
            self.current_waypoint.hide();
            self.waypoint_delete.hide();
            self.waypoint_scale.hide();
            self.waypoint_angle.hide();
        }

        {
            let current_beat = bottom_bar.cut_top(font_size * 1.5);
            update!(self.current_beat, current_beat);
            self.current_beat.text = format!("Beat: {:.2}", level_editor.current_beat).into();

            let timeline = bottom_bar.cut_top(font_size * 1.0);
            let was_pressed = self.timeline.state.pressed;
            update!(self.timeline, timeline);

            if self.timeline.state.pressed {
                let time = self.timeline.get_cursor_time();
                level_editor.scroll_time(time - level_editor.current_beat);
            }
            let replay = level_editor
                .dynamic_segment
                .as_ref()
                .map(|replay| replay.current_beat);
            self.timeline.update_time(level_editor.current_beat, replay);

            let select = context.mods.ctrl;
            if select {
                if !was_pressed && self.timeline.state.pressed {
                    self.timeline.start_selection();
                } else if was_pressed && !self.timeline.state.pressed {
                    let (start_beat, end_beat) = self.timeline.end_selection();
                    if start_beat != end_beat {
                        level_editor.dynamic_segment = Some(Replay {
                            start_beat,
                            end_beat,
                            current_beat: start_beat,
                            speed: Time::ONE,
                        });
                    }
                }
            }

            // self.timeline.auto_scale(level_editor.level.last_beat());
        }
    }
}
