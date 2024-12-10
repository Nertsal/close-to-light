use super::*;

use crate::ui::geometry::Geometry;

#[derive(Debug, Clone)]
pub struct TooltipWidget {
    pub state: WidgetState,
    pub title: TextWidget,
    pub text: TextWidget,
}

impl TooltipWidget {
    pub fn new() -> Self {
        let mut state = WidgetState::new();
        state.hide();
        Self {
            state,
            title: TextWidget::new("shortcut"),
            text: TextWidget::new("tip").aligned(vec2(0.5, 0.0)),
        }
    }

    pub fn update(&mut self, anchor: &WidgetState, tip: impl Into<Name>, context: &UiContext) {
        if !anchor.hovered {
            return;
        }
        self.state.show();
        let mut position = Aabb2::point(anchor.position.top_right())
            .extend_positive(vec2::splat(context.font_size * 1.5));
        if position.max.x >= context.screen.max.x {
            position = position.translate(vec2(-anchor.position.width() - position.width(), 0.0));
        }
        self.state.update(position, context);

        let position = position.extend_uniform(-context.font_size * 0.2);

        let title = position.clone().cut_top(context.font_size * 0.3);
        self.title.update(title, &context.scale_font(0.7));

        self.text.text = tip.into();
        self.text.update(position, &context.scale_font(0.9));
    }
}

impl Widget for TooltipWidget {
    fn draw(&self, context: &UiContext) -> Geometry {
        if self.state.visible {
            return Geometry::new();
        }

        let position = self.state.position;
        let theme = context.theme();
        let width = context.font_size * 0.1;
        let mut geometry = context
            .geometry
            .quad_fill(position.extend_uniform(width / 2.0), theme.dark);
        geometry.merge(self.title.draw(context));
        geometry.merge(self.text.draw(context));
        geometry.merge(context.geometry.quad_outline(position, width, theme.light));
        geometry
    }
}

pub struct EditorEditUi {
    //     pub state: WidgetState,
    //     pub tooltip: TooltipWidget,

    //     pub warn_select_level: TextWidget,

    //     pub new_event: TextWidget,
    //     // pub new_palette: ButtonWidget, // TODO: reimplement
    //     pub new_circle: ButtonWidget,
    //     pub new_line: ButtonWidget,
    //     pub new_waypoint: ButtonWidget,

    //     pub view: TextWidget,
    //     pub show_only_selected: CheckboxWidget,
    //     pub visualize_beat: CheckboxWidget,
    //     pub show_grid: CheckboxWidget,
    //     pub view_zoom: ValueWidget<f32>,

    //     pub placement: TextWidget,
    //     pub snap_grid: CheckboxWidget,
    //     pub grid_size: ValueWidget<f32>,

    //     pub light: TextWidget,
    //     pub light_delete: ButtonWidget,
    //     pub light_danger: CheckboxWidget,
    //     pub light_fade_in: ValueWidget<FloatTime>,
    //     pub light_fade_out: ValueWidget<FloatTime>,

    //     pub waypoint: ButtonWidget,
    //     pub prev_waypoint: IconButtonWidget,
    //     pub current_waypoint: TextWidget,
    //     pub next_waypoint: IconButtonWidget,
    //     pub waypoint_delete: ButtonWidget,
    //     pub waypoint_scale: ValueWidget<f32>,
    //     /// Angle in degrees.
    //     pub waypoint_angle: ValueWidget<f32>,
    //     pub waypoint_curve: DropdownWidget<Option<TrajectoryInterpolation>>,
    //     pub waypoint_interpolation: DropdownWidget<MoveInterpolation>,

    //     pub timeline: TimelineWidget,
}

impl EditorEditUi {
    pub fn new(context: Context) -> Self {
        let assets = &context.assets;
        Self {
        //     state: WidgetState::new(),
        //     tooltip: TooltipWidget::new(),

        //     warn_select_level: TextWidget::new("Select or create a difficulty in the Config tab"),

        //     new_event: TextWidget::new("Event"),
        //     // new_palette: ButtonWidget::new("Palette Swap"),
        //     new_circle: ButtonWidget::new("Circle"),
        //     new_line: ButtonWidget::new("Line"),
        //     new_waypoint: ButtonWidget::new("Add waypoint"),

        //     view: TextWidget::new("View"),
        //     show_only_selected: CheckboxWidget::new("Only selected"),
        //     visualize_beat: CheckboxWidget::new("Dynamic"),
        //     show_grid: CheckboxWidget::new("Grid"),
        //     view_zoom: ValueWidget::new_range("Zoom: ", 1.0, 0.5..=2.0, 0.25),

        //     placement: TextWidget::new("Placement"),
        //     snap_grid: CheckboxWidget::new("Grid snap"),
        //     grid_size: ValueWidget::new_range("Grid size", 16.0, 2.0..=32.0, 1.0),

        //     light: TextWidget::new("Light"),
        //     light_delete: ButtonWidget::new("delete"),
        //     light_danger: CheckboxWidget::new("Danger"),
        //     light_fade_in: ValueWidget::new_range(
        //         "Fade in",
        //         r32(0.5),
        //         r32(0.25)..=r32(10.0),
        //         r32(0.1),
        //     ),
        //     light_fade_out: ValueWidget::new_range(
        //         "Fade out",
        //         r32(0.5),
        //         r32(0.25)..=r32(10.0),
        //         r32(0.1),
        //     ),

        //     waypoint: ButtonWidget::new("Waypoints"),
        //     prev_waypoint: IconButtonWidget::new_normal(&assets.sprites.arrow_left.texture),
        //     current_waypoint: TextWidget::new("0"),
        //     next_waypoint: IconButtonWidget::new_normal(&assets.sprites.arrow_right.texture),
        //     waypoint_delete: ButtonWidget::new("delete"),
        //     waypoint_scale: ValueWidget::new_range("Scale", 1.0, 0.25..=10.0, 0.25),
        //     waypoint_angle: ValueWidget::new_circle("Angle", 0.0, 360.0, 15.0),
        //     waypoint_curve: DropdownWidget::new(
        //         "Curve",
        //         0,
        //         [
        //             ("Continue", None),
        //             ("Linear", Some(TrajectoryInterpolation::Linear)),
        //             (
        //                 "Spline",
        //                 Some(TrajectoryInterpolation::Spline { tension: r32(0.1) }),
        //             ),
        //             ("Bezier", Some(TrajectoryInterpolation::Bezier)),
        //         ],
        //     ),
        //     waypoint_interpolation: DropdownWidget::new(
        //         "Interpolation",
        //         0,
        //         [
        //             ("Linear", MoveInterpolation::Linear),
        //             ("Smoothstep", MoveInterpolation::Smoothstep),
        //             ("EaseIn", MoveInterpolation::EaseIn),
        //             ("EaseOut", MoveInterpolation::EaseOut),
        //         ],
        //     ),

        //     timeline: TimelineWidget::new(context.clone()),
        }
    }
}

impl EditorEditUi {
    pub fn layout(
        &mut self,
        position: Aabb2<f32>,
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
                .get_or(|| TextWidget::new("Select or create a difficulty in the Config tab"));
            text.update(warn, context);
            return;
        };

        let mut main = position;
        let font_size = context.font_size;
        let layout_size = context.layout_size;

        let bottom_bar = main.cut_bottom(layout_size * 3.0);
        let mut bottom_bar = bottom_bar.extend_symmetric(-vec2(5.0, 0.0) * layout_size);

        let mut main = main
            .extend_symmetric(-vec2(1.0, 2.0) * layout_size)
            .extend_up(-layout_size);
        let mut left_bar = main.cut_left(layout_size * 7.0);
        let mut right_bar = main.cut_right(layout_size * 7.0);

        let spacing = layout_size * 0.25;
        let title_size = font_size * 1.3;
        let button_height = font_size * 1.2;
        let delete_width = font_size * 3.5;
        let value_height = font_size * 1.2;

        let tooltip = context.state.get_or(TooltipWidget::new);

        // Event
        {
            let mut bar = left_bar;

            let event = bar.cut_top(title_size);
            let text = context
                .state
                .get_or(|| TextWidget::new("Event").aligned(vec2(0.0, 0.5)));
            text.update(event, context);
            text.options.size = title_size;

            if level_editor.level_state.waypoints.is_some() {
                let waypoint = bar.cut_top(button_height);
                bar.cut_top(spacing);
                let button = context.state.get_or(|| ButtonWidget::new("Add waypoint"));
                button.update(waypoint, context);
                if button.text.state.clicked {
                    actions.push(LevelAction::NewWaypoint.into());
                }

                tooltip.update(&button.text.state, "1", context);

                bar.cut_top(button_height);
                bar.cut_top(spacing);
            } else {
                // let palette = bar.cut_top(button_height);
                // bar.cut_top(spacing);
                // update!(self.new_palette, palette);
                // if self.new_palette.text.state.clicked {
                //     level_editor.palette_swap();
                // }

                let new_light_width = font_size * 3.0;

                let circle = bar.cut_top(button_height).cut_left(new_light_width);
                bar.cut_top(spacing);
                let button = context.state.get_or(|| ButtonWidget::new("Circle"));
                button.update(circle, context);
                if button.text.state.clicked {
                    actions.push(LevelAction::NewLight(Shape::circle(r32(1.0))).into());
                }
                tooltip.update(&button.text.state, "1", context);

                let line = bar.cut_top(button_height).cut_left(new_light_width);
                bar.cut_top(spacing);
                let button = context.state.get_or(|| ButtonWidget::new("Line"));
                button.update(line, context);
                if button.text.state.clicked {
                    actions.push(LevelAction::NewLight(Shape::line(r32(1.0))).into());
                }
                tooltip.update(&button.text.state, "2", context);
            }

            bar.cut_top(layout_size * 1.5);
            left_bar = bar;
        }

        // View
        {
            let mut bar = left_bar;

            let view = bar.cut_top(title_size);
            bar.cut_top(spacing);
            let text = context
                .state
                .get_or(|| TextWidget::new("View").aligned(vec2(0.0, 0.5)));
            text.update(view, context);
            text.options.size = title_size;

            let selected = bar.cut_top(font_size);
            bar.cut_top(spacing);
            let toggle = context.state.get_or(|| ToggleWidget::new("Only selected"));
            toggle.update(selected, context);
            if toggle.state.clicked {
                actions.push(EditorAction::ToggleShowOnlySelected.into());
            }
            toggle.checked = editor.show_only_selected;

            let dynamic = bar.cut_top(font_size);
            bar.cut_top(spacing);
            let toggle = context.state.get_or(|| ToggleWidget::new("Dynamic"));
            toggle.update(dynamic, context);
            if toggle.state.clicked {
                actions.push(EditorAction::ToggleDynamicVisual.into());
            }
            toggle.checked = editor.visualize_beat;
            tooltip.update(&toggle.state, "F", context);

            let grid = bar.cut_top(font_size);
            bar.cut_top(spacing);
            let toggle = context.state.get_or(|| ToggleWidget::new("Show grid"));
            toggle.update(grid, context);
            if toggle.state.clicked {
                actions.push(EditorAction::ToggleGrid.into());
            }
            toggle.checked = editor.render_options.show_grid;
            tooltip.update(&toggle.state, "C-~", context);

            // let waypoints = bar.cut_top(button_height);
            // bar.cut_top(spacing);
            // update!(self.view_waypoints, waypoints);
            // if self.view_waypoints.text.state.clicked {
            //     editor.view_waypoints();
            // }

            let zoom = bar.cut_top(value_height);
            bar.cut_top(spacing);
            let slider = context
                .state
                .get_or(|| ValueWidget::new_range("Zoom", editor.view_zoom, 0.5..=2.0, 0.25));
            {
                let mut view_zoom = editor.view_zoom;
                slider.update(zoom, context, &mut view_zoom);
                actions.push(EditorAction::SetViewZoom(view_zoom).into());
            }
            context.update_focus(slider.state.hovered);

            bar.cut_top(layout_size * 1.5);
            left_bar = bar;
        }

        // Placement
        {
            let mut bar = left_bar;

            let placement = bar.cut_top(title_size);
            let text = context
                .state
                .get_or(|| TextWidget::new("Placement").aligned(vec2(0.0, 0.5)));
            text.update(placement, context);
            text.options.size = title_size;

            let grid_snap = bar.cut_top(font_size);
            bar.cut_top(spacing);
            let button = context.state.get_or(|| ToggleWidget::new("Snap to grid"));
            button.update(grid_snap, context);
            if button.state.clicked {
                actions.push(EditorAction::ToggleGridSnap.into());
            }
            button.checked = editor.snap_to_grid;
            tooltip.update(&button.state, "~", context);

            let grid_size = bar.cut_top(value_height);
            bar.cut_top(spacing);
            {
                let mut value = 10.0 / editor.grid_size.as_f32();
                let slider = context
                    .state
                    .get_or(|| ValueWidget::new_range("Grid size", value, 2.0..=32.0, 1.0));
                slider.update(grid_size, context, &mut value);
                actions.push(EditorAction::SetGridSize(r32(10.0 / value)).into());
                context.update_focus(slider.state.hovered);
            }

            bar.cut_top(layout_size * 1.5);
            left_bar = bar;
        }

        // Light
        {
            let selected = level_editor
                .selected_light
                .and_then(|i| level_editor.level.events.get(i.event))
                .filter(|event| matches!(event.event, Event::Light(_)));

            if let Some(event) = selected {
                let light_id = level_editor
                    .selected_light
                    .expect("light selected without id 0_0");
                if let Event::Light(light) = &event.event {
                    let mut bar = right_bar;

                    let light_pos = bar.cut_top(title_size);
                    let text = context
                        .state
                        .get_or(|| TextWidget::new("Light").aligned(vec2(0.0, 0.5)));
                    text.update(light_pos, context);
                    text.options.size = title_size;

                    let delete = bar.cut_top(button_height).cut_left(delete_width);
                    let button = context
                        .state
                        .get_or(|| ButtonWidget::new("Delete").color(ThemeColor::Danger));
                    button.update(delete, context);
                    tooltip.update(&button.text.state, "X", context);
                    if button.text.state.clicked {
                        actions.push(LevelAction::DeleteLight(light_id).into());
                    }

                    let danger_pos = bar.cut_top(button_height);
                    bar.cut_top(spacing);
                    let button = context
                        .state
                        .get_or(|| ToggleWidget::new("Danger").color(ThemeColor::Danger));
                    button.update(danger_pos, context);
                    if button.state.clicked {
                        actions.push(LevelAction::ToggleDanger(light_id).into());
                    }
                    button.checked = light.danger;
                    tooltip.update(&button.state, "D", context);

                    {
                        let fade_in = bar.cut_top(value_height);
                        bar.cut_top(spacing);
                        let mut fade = time_to_seconds(light.movement.fade_in);
                        let slider = context.state.get_or(|| {
                            ValueWidget::new_range("Fade in", fade, r32(0.25)..=r32(10.0), r32(0.1))
                        });
                        slider.update(fade_in, context, &mut fade);
                        context.update_focus(slider.state.hovered);
                        actions.push(
                            LevelAction::ChangeFadeIn(light_id, Change::Set(seconds_to_time(fade)))
                                .into(),
                        );
                    }

                    {
                        let fade_out = bar.cut_top(value_height);
                        bar.cut_top(spacing);
                        let mut fade = time_to_seconds(light.movement.fade_out);
                        let slider = context.state.get_or(|| {
                            ValueWidget::new_range(
                                "Fade out",
                                fade,
                                r32(0.25)..=r32(10.0),
                                r32(0.1),
                            )
                        });
                        slider.update(fade_out, context, &mut fade);
                        context.update_focus(slider.state.hovered);
                        actions.push(
                            LevelAction::ChangeFadeOut(
                                light_id,
                                Change::Set(seconds_to_time(fade)),
                            )
                            .into(),
                        );
                    }

                    bar.cut_top(layout_size * 1.5);

                    let waypoints = bar.cut_top(title_size);
                    let button = context.state.get_or(|| ToggleWidget::new("Waypoints"));
                    button.update(waypoints, context);
                    button.text.options.size = title_size;
                    button.checked = matches!(level_editor.state, State::Waypoints { .. });
                    if button.state.clicked {
                        actions.push(LevelAction::ToggleWaypointsView.into());
                    }

                    bar.cut_top(spacing);
                    right_bar = bar;
                }
            }
        }

        if let Some(waypoints) = &level_editor.level_state.waypoints {
            if let Some(selected) = waypoints.selected {
                if let Some(event) = level_editor.level.events.get(waypoints.light.event) {
                    if let Event::Light(light) = &event.event {
                        let frames = light.movement.key_frames.len();
                        if let Some(frame) = light.movement.get_frame(selected) {
                            // Waypoint
                            let mut bar = right_bar;

                            let mut current = bar.cut_top(font_size);
                            let mut current = current.cut_left(font_size * 3.0);

                            // Previous waypoint
                            let prev = current
                                .cut_left(current.height() * 0.6)
                                .zero_size(vec2(0.5, 0.5))
                                .extend_uniform(font_size * 0.7);
                            if let WaypointId::Initial = selected {
                                let button = context.state.get_or(|| {
                                    IconWidget::new(
                                        &context.context.assets.sprites.button_prev_hollow,
                                    )
                                });
                                button.update(prev, context);
                            } else {
                                let button = context.state.get_or(|| {
                                    IconButtonWidget::new_normal(
                                        &context.context.assets.sprites.button_prev,
                                    )
                                });
                                button.update(prev, context);
                                if button.state.clicked {
                                    if let Some(id) = selected.prev() {
                                        actions.push(LevelAction::SelectWaypoint(id).into());
                                    }
                                }
                            };

                            let i = match selected {
                                WaypointId::Initial => 0,
                                WaypointId::Frame(i) => i + 1,
                            };

                            // Next waypoint
                            let next = current
                                .cut_right(current.height() * 0.6)
                                .zero_size(vec2(0.5, 0.5))
                                .extend_uniform(font_size * 0.7);
                            if i >= frames {
                                let button = context.state.get_or(|| {
                                    IconWidget::new(
                                        &context.context.assets.sprites.button_next_hollow,
                                    )
                                });
                                button.update(next, context);
                            } else {
                                let button = context.state.get_or(|| {
                                    IconButtonWidget::new_normal(
                                        &context.context.assets.sprites.button_next,
                                    )
                                });
                                button.update(next, context);
                                if button.state.clicked {
                                    actions
                                        .push(LevelAction::SelectWaypoint(selected.next()).into());
                                }
                            }

                            // Current waypoint
                            let text = context.state.get_or(|| TextWidget::new("0"));
                            text.update(current, context);
                            text.text = (i + 1).to_string().into();

                            // Delete
                            let delete = bar.cut_top(button_height).cut_left(delete_width);
                            let button = context
                                .state
                                .get_or(|| ButtonWidget::new("Delete").color(ThemeColor::Danger));
                            button.update(delete, context);
                            if button.text.state.clicked {
                                actions.push(
                                    LevelAction::DeleteWaypoint(waypoints.light, selected).into(),
                                );
                            }
                            tooltip.update(&button.text.state, "X", context);

                            let mut new_frame = frame;

                            let scale = bar.cut_top(value_height);
                            bar.cut_top(spacing);
                            let mut value = frame.scale.as_f32();
                            let slider = context.state.get_or(|| {
                                ValueWidget::new_range("Scale", value, 0.25..=10.0, 0.25)
                            });
                            slider.update(scale, context, &mut value);
                            new_frame.scale = r32(value);
                            context.update_focus(slider.state.hovered);

                            let angle = bar.cut_top(value_height);
                            bar.cut_top(spacing);
                            let mut value = frame.rotation.as_degrees().as_f32();
                            let slider = context
                                .state
                                .get_or(|| ValueWidget::new_circle("Angle", value, 360.0, 1.0));
                            slider.update(angle, context, &mut value);
                            new_frame.rotation = Angle::from_degrees(r32(value.round()));
                            context.update_focus(slider.state.hovered);
                            tooltip.update(&slider.state, "Q/E", context);

                            actions.push(
                                LevelAction::SetWaypointFrame(waypoints.light, selected, new_frame)
                                    .into(),
                            );

                            // let curve = bar.cut_top(button_height);
                            // bar.cut_top(spacing);
                            // let interpolation = bar.cut_top(button_height);
                            // bar.cut_top(spacing);
                            // if let Some((mut move_interpolation, mut curve_interpolation)) =
                            //     light.movement.get_interpolation(selected)
                            // {
                            //     self.waypoint_curve.show();
                            //     self.waypoint_curve.update(
                            //         curve,
                            //         context,
                            //         &mut curve_interpolation,
                            //     );
                            //     actions.push(
                            //         LevelAction::SetWaypointCurve(
                            //             waypoints.light,
                            //             selected,
                            //             curve_interpolation,
                            //         )
                            //         .into(),
                            //     );

                            //     self.waypoint_interpolation.show();
                            //     self.waypoint_interpolation.update(
                            //         interpolation,
                            //         context,
                            //         &mut move_interpolation,
                            //     );
                            //     actions.push(
                            //         LevelAction::SetWaypointInterpolation(
                            //             waypoints.light,
                            //             selected,
                            //             move_interpolation,
                            //         )
                            //         .into(),
                            //     );
                            // } else {
                            //     self.waypoint_curve.hide();
                            //     self.waypoint_interpolation.hide();
                            // }

                            bar.cut_top(spacing);
                            right_bar = bar;
                        }
                    }
                }
            }
        }
    }
}

// impl StatefulWidget for EditorEditUi {
//     type State<'a> = (&'a Editor, Vec<EditorStateAction>);

//     fn state_mut(&mut self) -> &mut WidgetState {
//         &mut self.state
//     }

//     fn update(
//         &mut self,
//         position: Aabb2<f32>,
//         context: &mut UiContext,
//         (state, actions): &mut Self::State<'_>,
//     ) {
//         let editor = state;
//         let Some(level_editor) = &editor.level_edit else {
//             let size = vec2(15.0, 1.0) * context.font_size;
//             let warn = position
//                 .align_aabb(size, vec2(0.5, 1.0))
//                 .translate(vec2(0.0, -3.0 * size.y));
//             self.warn_select_level.show();
//             self.warn_select_level.update(warn, context);

//             return;
//         };

//         self.tooltip.state.hide();
//         self.warn_select_level.hide();

//         let mut main = position;
//         let font_size = context.font_size;
//         let layout_size = context.layout_size;

//         macro_rules! update {
//             ($widget:expr, $position:expr) => {{
//                 $widget.update($position, context);
//             }};
//             ($widget:expr, $position:expr, $state:expr) => {{
//                 $widget.update($position, context, $state);
//             }};
//         }

//         let bottom_bar = main.cut_bottom(layout_size * 3.0);
//         let mut bottom_bar = bottom_bar.extend_symmetric(-vec2(5.0, 0.0) * layout_size);

//         let mut main = main
//             .extend_symmetric(-vec2(1.0, 2.0) * layout_size)
//             .extend_up(-layout_size);
//         let mut left_bar = main.cut_left(layout_size * 7.0);
//         let mut right_bar = main.cut_right(layout_size * 7.0);

//         let spacing = layout_size * 0.25;
//         let title_size = font_size * 1.3;
//         let button_height = font_size * 1.2;
//         let value_height = font_size * 2.0;

//         {
//             let mut bar = left_bar;

//             let event = bar.cut_top(title_size);
//             update!(self.new_event, event);
//             self.new_event.options.size = title_size;

//             if level_editor.level_state.waypoints.is_some() {
//                 self.new_circle.hide();
//                 self.new_line.hide();
//                 self.new_waypoint.show();

//                 let waypoint = bar.cut_top(button_height);
//                 bar.cut_top(spacing);
//                 self.new_waypoint.update(waypoint, context);
//                 if self.new_waypoint.text.state.clicked {
//                     actions.push(LevelAction::NewWaypoint.into());
//                 }
//                 self.tooltip
//                     .update(&self.new_waypoint.text.state, "1", context);

//                 bar.cut_top(button_height);
//                 bar.cut_top(spacing);
//             } else {
//                 self.new_circle.show();
//                 self.new_line.show();
//                 self.new_waypoint.hide();

//                 // let palette = bar.cut_top(button_height);
//                 // bar.cut_top(spacing);
//                 // update!(self.new_palette, palette);
//                 // if self.new_palette.text.state.clicked {
//                 //     level_editor.palette_swap();
//                 // }

//                 let circle = bar.cut_top(button_height);
//                 bar.cut_top(spacing);
//                 update!(self.new_circle, circle);
//                 if self.new_circle.text.state.clicked {
//                     actions.push(LevelAction::NewLight(Shape::circle(r32(1.0))).into());
//                 }
//                 self.tooltip
//                     .update(&self.new_circle.text.state, "1", context);

//                 let line = bar.cut_top(button_height);
//                 bar.cut_top(spacing);
//                 update!(self.new_line, line);
//                 if self.new_line.text.state.clicked {
//                     actions.push(LevelAction::NewLight(Shape::line(r32(1.0))).into());
//                 }
//                 self.tooltip.update(&self.new_line.text.state, "2", context);
//             }

//             bar.cut_top(layout_size * 1.5);

//             let view = bar.cut_top(title_size);
//             bar.cut_top(spacing);
//             update!(self.view, view);
//             self.view.options.size = title_size;

//             let selected = bar.cut_top(font_size);
//             bar.cut_top(spacing);
//             update!(self.show_only_selected, selected);
//             if self.show_only_selected.state.clicked {
//                 actions.push(EditorAction::ToggleShowOnlySelected.into());
//             }
//             self.show_only_selected.checked = editor.show_only_selected;

//             let dynamic = bar.cut_top(font_size);
//             bar.cut_top(spacing);
//             update!(self.visualize_beat, dynamic);
//             if self.visualize_beat.state.clicked {
//                 actions.push(EditorAction::ToggleDynamicVisual.into());
//             }
//             self.visualize_beat.checked = editor.visualize_beat;
//             self.tooltip
//                 .update(&self.visualize_beat.state, "F", context);

//             let grid = bar.cut_top(font_size);
//             bar.cut_top(spacing);
//             update!(self.show_grid, grid);
//             if self.show_grid.state.clicked {
//                 actions.push(EditorAction::ToggleGrid.into());
//             }
//             self.show_grid.checked = editor.render_options.show_grid;
//             self.tooltip.update(&self.show_grid.state, "C-~", context);

//             // let waypoints = bar.cut_top(button_height);
//             // bar.cut_top(spacing);
//             // update!(self.view_waypoints, waypoints);
//             // if self.view_waypoints.text.state.clicked {
//             //     editor.view_waypoints();
//             // }

//             let zoom = bar.cut_top(value_height);
//             bar.cut_top(spacing);
//             {
//                 let mut view_zoom = editor.view_zoom;
//                 update!(self.view_zoom, zoom, &mut view_zoom);
//                 actions.push(EditorAction::SetViewZoom(view_zoom).into());
//             }
//             context.update_focus(self.view_zoom.state.hovered);

//             bar.cut_top(layout_size * 1.5);
//             left_bar = bar;
//         }

//         {
//             // Spacing
//             let mut bar = left_bar;

//             let placement = bar.cut_top(title_size);
//             update!(self.placement, placement);
//             self.placement.options.size = title_size;

//             let grid_snap = bar.cut_top(button_height);
//             bar.cut_top(spacing);
//             update!(self.snap_grid, grid_snap);
//             if self.snap_grid.state.clicked {
//                 actions.push(EditorAction::ToggleGridSnap.into());
//             }
//             self.snap_grid.checked = editor.snap_to_grid;
//             self.tooltip.update(&self.snap_grid.state, "~", context);

//             let grid_size = bar.cut_top(value_height);
//             bar.cut_top(spacing);
//             {
//                 let mut value = 10.0 / editor.grid_size.as_f32();
//                 update!(self.grid_size, grid_size, &mut value);
//                 actions.push(EditorAction::SetGridSize(r32(10.0 / value)).into());
//             }
//             context.update_focus(self.grid_size.state.hovered);

//             bar.cut_top(layout_size * 1.5);
//             // left_bar = bar;
//         }

//         {
//             // Light
//             let selected = level_editor
//                 .selected_light
//                 .and_then(|i| level_editor.level.events.get(i.event))
//                 .filter(|event| matches!(event.event, Event::Light(_)));

//             match selected {
//                 None => {
//                     self.light.hide();
//                     self.light_delete.hide();
//                     self.light_danger.hide();
//                     self.light_fade_in.hide();
//                     self.light_fade_out.hide();
//                     self.waypoint.hide();
//                 }
//                 Some(event) => {
//                     let light_id = level_editor
//                         .selected_light
//                         .expect("light selected without id 0_0");
//                     if let Event::Light(light) = &event.event {
//                         self.light.show();
//                         self.light_delete.show();
//                         self.light_danger.show();
//                         self.light_fade_in.show();
//                         self.light_fade_out.show();
//                         self.waypoint.show();

//                         let mut bar = right_bar;

//                         let light_pos = bar.cut_top(title_size);
//                         update!(self.light, light_pos);
//                         self.light.options.size = title_size;

//                         let delete = bar.cut_top(button_height);
//                         self.light_delete.update(delete, context);
//                         // NOTE: click action delayed because level_editor is borrowed
//                         self.tooltip
//                             .update(&self.light_delete.text.state, "X", context);

//                         let danger_pos = bar.cut_top(button_height);
//                         bar.cut_top(spacing);
//                         update!(self.light_danger, danger_pos);
//                         if self.light_danger.state.clicked {
//                             actions.push(LevelAction::ToggleDanger(light_id).into());
//                         }
//                         self.light_danger.checked = light.danger;
//                         self.tooltip.update(&self.light_danger.state, "D", context);

//                         {
//                             let fade_in = bar.cut_top(value_height);
//                             bar.cut_top(spacing);
//                             let mut fade = time_to_seconds(light.movement.fade_in);
//                             update!(self.light_fade_in, fade_in, &mut fade);
//                             context.update_focus(self.light_fade_in.state.hovered);
//                             actions.push(
//                                 LevelAction::ChangeFadeIn(
//                                     light_id,
//                                     Change::Set(seconds_to_time(fade)),
//                                 )
//                                 .into(),
//                             );
//                         }

//                         {
//                             let fade_out = bar.cut_top(value_height);
//                             bar.cut_top(spacing);
//                             let mut fade = time_to_seconds(light.movement.fade_out);
//                             update!(self.light_fade_out, fade_out, &mut fade);
//                             context.update_focus(self.light_fade_out.state.hovered);
//                             actions.push(
//                                 LevelAction::ChangeFadeOut(
//                                     light_id,
//                                     Change::Set(seconds_to_time(fade)),
//                                 )
//                                 .into(),
//                             );
//                         }

//                         bar.cut_top(layout_size * 1.5);

//                         let waypoints = bar.cut_top(button_height);
//                         update!(self.waypoint, waypoints);
//                         if self.waypoint.text.state.clicked {
//                             actions.push(LevelAction::ToggleWaypointsView.into());
//                         }

//                         bar.cut_top(spacing);
//                         right_bar = bar;

//                         // Delayed actions
//                         if self.light_delete.text.state.clicked {
//                             actions.push(LevelAction::DeleteLight(light_id).into());
//                         }
//                     }
//                 }
//             }
//         }

//         let mut waypoint = false;
//         if let Some(waypoints) = &level_editor.level_state.waypoints {
//             if let Some(selected) = waypoints.selected {
//                 if let Some(event) = level_editor.level.events.get(waypoints.light.event) {
//                     if let Event::Light(light) = &event.event {
//                         let frames = light.movement.key_frames.len();
//                         if let Some(frame) = light.movement.get_frame(selected) {
//                             // Waypoint
//                             waypoint = true;
//                             self.prev_waypoint.show();
//                             self.next_waypoint.show();
//                             self.current_waypoint.show();
//                             self.waypoint_delete.show();
//                             self.waypoint_scale.show();
//                             self.waypoint_angle.show();

//                             let mut bar = right_bar;

//                             let mut current = bar.cut_top(button_height);

//                             if let WaypointId::Initial = selected {
//                                 self.prev_waypoint.hide();
//                             } else {
//                                 self.prev_waypoint.show();
//                             }
//                             let prev = current.cut_left(current.height());
//                             self.prev_waypoint.update(prev, context);

//                             let i = match selected {
//                                 WaypointId::Initial => 0,
//                                 WaypointId::Frame(i) => i + 1,
//                             };

//                             if i >= frames {
//                                 self.next_waypoint.hide();
//                             } else {
//                                 self.next_waypoint.show();
//                             }
//                             let next = current.cut_right(current.height());
//                             self.next_waypoint.update(next, context);

//                             self.current_waypoint.update(current, context);
//                             self.current_waypoint.text = (i + 1).to_string().into();

//                             let delete = bar.cut_top(button_height);
//                             self.waypoint_delete.update(delete, context);
//                             if self.waypoint_delete.text.state.clicked {
//                                 actions.push(
//                                     LevelAction::DeleteWaypoint(waypoints.light, selected).into(),
//                                 );
//                             }
//                             self.tooltip
//                                 .update(&self.waypoint_delete.text.state, "X", context);

//                             let mut new_frame = frame;

//                             let scale = bar.cut_top(value_height);
//                             bar.cut_top(spacing);
//                             let mut value = frame.scale.as_f32();
//                             update!(self.waypoint_scale, scale, &mut value);
//                             new_frame.scale = r32(value);
//                             context.update_focus(self.waypoint_scale.state.hovered);

//                             let angle = bar.cut_top(value_height);
//                             bar.cut_top(spacing);
//                             let mut value = frame.rotation.as_degrees().as_f32();
//                             update!(self.waypoint_angle, angle, &mut value);
//                             new_frame.rotation = Angle::from_degrees(r32(value.round()));
//                             context.update_focus(self.waypoint_angle.state.hovered);
//                             self.tooltip
//                                 .update(&self.waypoint_angle.state, "Q/E", context);

//                             actions.push(
//                                 LevelAction::SetWaypointFrame(waypoints.light, selected, new_frame)
//                                     .into(),
//                             );

//                             let curve = bar.cut_top(button_height);
//                             bar.cut_top(spacing);
//                             let interpolation = bar.cut_top(button_height);
//                             bar.cut_top(spacing);
//                             if let Some((mut move_interpolation, mut curve_interpolation)) =
//                                 light.movement.get_interpolation(selected)
//                             {
//                                 self.waypoint_curve.show();
//                                 self.waypoint_curve.update(
//                                     curve,
//                                     context,
//                                     &mut curve_interpolation,
//                                 );
//                                 actions.push(
//                                     LevelAction::SetWaypointCurve(
//                                         waypoints.light,
//                                         selected,
//                                         curve_interpolation,
//                                     )
//                                     .into(),
//                                 );

//                                 self.waypoint_interpolation.show();
//                                 self.waypoint_interpolation.update(
//                                     interpolation,
//                                     context,
//                                     &mut move_interpolation,
//                                 );
//                                 actions.push(
//                                     LevelAction::SetWaypointInterpolation(
//                                         waypoints.light,
//                                         selected,
//                                         move_interpolation,
//                                     )
//                                     .into(),
//                                 );
//                             } else {
//                                 self.waypoint_curve.hide();
//                                 self.waypoint_interpolation.hide();
//                             }

//                             if self.prev_waypoint.state.clicked {
//                                 if let Some(id) = selected.prev() {
//                                     actions.push(LevelAction::SelectWaypoint(id).into());
//                                 }
//                             } else if self.next_waypoint.state.clicked {
//                                 actions.push(LevelAction::SelectWaypoint(selected.next()).into());
//                             }
//                         }
//                     }
//                 }
//             }
//         }
//         if !waypoint {
//             self.prev_waypoint.hide();
//             self.next_waypoint.hide();
//             self.current_waypoint.hide();
//             self.waypoint_delete.hide();
//             self.waypoint_scale.hide();
//             self.waypoint_angle.hide();
//             self.waypoint_curve.hide();
//             self.waypoint_interpolation.hide();
//         }

//         {
//             let timeline = bottom_bar.cut_top(font_size * 1.0);
//             let was_pressed = self.timeline.state.pressed;

//             {
//                 let mut state = (level_editor, vec![]);
//                 update!(self.timeline, timeline, &mut state);
//                 actions.extend(state.1.into_iter().map(Into::into));
//             }

//             if self.timeline.mainline.state.pressed {
//                 let time = self.timeline.get_cursor_time();
//                 actions.push(EditorStateAction::ScrollTime(
//                     time - level_editor.current_time,
//                 ));
//             }
//             self.timeline.update_time(level_editor.current_time);

//             // self.timeline.auto_scale(level_editor.level.last_beat());
//         }
//     }
// }

// TODO: move to layout (we have scroll in context)
// if shift && self.ui.edit.timeline.state.hovered {
//     actions.push(EditorStateAction::TimelineScroll(scroll));
// } else if ctrl {
//     if self.ui.edit.timeline.state.hovered {
//         // Zoom on the timeline
//         actions.push(EditorStateAction::TimelineZoom(scroll));
//     } else if let State::Place { .. }
//     | State::Waypoints {
//         state: WaypointsState::New,
//         ..
//     } = level_editor.state
//     {
//         // Scale light or waypoint placement
//         let delta = scroll * r32(0.1);
//         actions.push(LevelAction::ScalePlacement(delta).into());
//     } else if let Some(waypoints) = &level_editor.level_state.waypoints {
//         if let Some(selected) = waypoints.selected {
//             let delta = scroll * r32(0.1);
//             actions.push(
//                 LevelAction::ScaleWaypoint(waypoints.light, selected, delta)
//                     .into(),
//             );
//         }
//     } else if let Some(id) = level_editor.selected_light {
//         // Control fade time
//         let scroll = scroll.as_f32() as Time;
//         let change = scroll * self.editor.config.scroll_slow.as_time(beat_time);
//         let action = if shift {
//             LevelAction::ChangeFadeOut(id, Change::Add(change))
//         } else {
//             LevelAction::ChangeFadeIn(id, Change::Add(change))
//         };
//         actions.push(action.into());
//     }
