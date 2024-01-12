use super::*;

use crate::ui::{layout, widget::*};

/// Layout and state of the UI.
pub struct EditorUI {
    pub screen: WidgetState,
    pub game: WidgetState,

    pub new_event: TextWidget,
    pub new_palette: ButtonWidget,
    pub new_circle: ButtonWidget,
    pub new_line: ButtonWidget,

    pub selected_text: TextWidget,
    pub selected_light: LightStateWidget,

    pub view: TextWidget,
    pub visualize_beat: CheckboxWidget,
    pub show_grid: CheckboxWidget,
    pub view_lights: ButtonWidget,
    pub view_waypoints: ButtonWidget,
    pub view_zoom: ValueWidget<f32>,

    pub snap_grid: CheckboxWidget,

    pub current_beat: TextWidget,
    pub timeline: TimelineWidget,
}

impl EditorUI {
    pub fn new() -> Self {
        Self {
            screen: default(),
            game: default(),

            new_event: TextWidget::new("Event"),
            new_palette: ButtonWidget::new("Palette Swap"),
            new_circle: ButtonWidget::new("Circle"),
            new_line: ButtonWidget::new("Line"),

            selected_text: default(),
            selected_light: LightStateWidget::new(),

            view: TextWidget::new("View"),
            visualize_beat: CheckboxWidget::new("Dynamic"),
            show_grid: CheckboxWidget::new("Grid"),
            view_lights: ButtonWidget::new("Lights"),
            view_waypoints: ButtonWidget::new("Waypoints"),
            view_zoom: ValueWidget::new("Zoom: ", 1.0, 0.5..=2.0, 0.25),

            snap_grid: CheckboxWidget::new("Grid snap"),

            current_beat: default(),
            timeline: TimelineWidget::new(),
        }
    }

    pub fn layout(
        &mut self,
        editor: &mut Editor,
        render_options: &mut RenderOptions,
        screen: Aabb2<f32>,
        cursor: CursorContext,
        delta_time: Time,
        geng: &Geng,
    ) -> bool {
        let screen = layout::fit_aabb(vec2(16.0, 9.0), screen, vec2::splat(0.5));

        let font_size = screen.height() * 0.03;
        let layout_size = screen.height() * 0.03;

        let mut context = UiContext {
            theme: editor.model.options.theme,
            layout_size,
            font_size,
            can_focus: true,
            cursor,
            delta_time: delta_time.as_f32(),
        };
        macro_rules! update {
            ($widget:expr, $position:expr) => {{
                $widget.update($position, &context);
            }};
        }

        update!(self.screen, screen);

        {
            let max_size = screen.size() * 0.7;

            let ratio = 16.0 / 9.0;
            let max_height = max_size.y.min(max_size.x / ratio);

            let game_height = max_height;
            let game_size = vec2(game_height * ratio, game_height);

            let game = layout::align_aabb(game_size, screen, vec2(0.5, 0.5));
            update!(self.game, game);
        }

        let main = screen;
        let (top_bar, main) = layout::cut_top_down(main, font_size * 1.5);
        let main = main.extend_down(-layout_size);
        let (main, bottom_bar) = layout::cut_top_down(main, main.height() - font_size * 1.5);

        let main = main.extend_symmetric(-vec2(1.0, 2.0) * layout_size);
        let (left_bar, main) = layout::cut_left_right(main, font_size * 5.0);
        let (right_bar, main) = layout::cut_left_right(main, main.width() - font_size * 5.0);

        {
            let bar = left_bar;
            let spacing = layout_size * 0.25;
            let button_height = font_size * 1.2;

            let (event, bar) = layout::cut_top_down(bar, font_size);
            update!(self.new_event, event);

            let (palette, bar) = layout::cut_top_down(bar, button_height);
            let bar = bar.extend_up(-spacing);
            update!(self.new_palette, palette);
            if self.new_palette.text.state.clicked {
                editor.palette_swap();
            }

            let (circle, bar) = layout::cut_top_down(bar, button_height);
            let bar = bar.extend_up(-spacing);
            update!(self.new_circle, circle);
            if self.new_circle.text.state.clicked {
                editor.new_light_circle();
            }

            let (line, bar) = layout::cut_top_down(bar, button_height);
            let bar = bar.extend_up(-spacing);
            update!(self.new_line, line);
            if self.new_line.text.state.clicked {
                editor.new_light_line();
            }

            let bar = bar.extend_up(-layout_size * 1.5);

            let (view, bar) = layout::cut_top_down(bar, font_size);
            let bar = bar.extend_up(-spacing);
            update!(self.view, view);

            let (dynamic, bar) = layout::cut_top_down(bar, font_size);
            let bar = bar.extend_up(-spacing);
            update!(self.visualize_beat, dynamic);
            if self.visualize_beat.state.clicked {
                editor.visualize_beat = !editor.visualize_beat;
            }
            self.visualize_beat.checked = editor.visualize_beat;

            let (grid, bar) = layout::cut_top_down(bar, font_size);
            let bar = bar.extend_up(-spacing);
            update!(self.show_grid, grid);
            if self.show_grid.state.clicked {
                render_options.show_grid = !render_options.show_grid;
            }
            self.show_grid.checked = editor.visualize_beat;

            let (lights, bar) = layout::cut_top_down(bar, button_height);
            let bar = bar.extend_up(-spacing);
            update!(self.view_lights, lights);
            if self.view_lights.text.state.clicked {
                editor.view_lights();
            }

            let (waypoints, bar) = layout::cut_top_down(bar, button_height);
            let bar = bar.extend_up(-spacing);
            update!(self.view_waypoints, waypoints);
            if self.view_waypoints.text.state.clicked {
                editor.view_waypoints();
            }

            let (zoom, bar) = layout::cut_top_down(bar, font_size);
            let bar = bar.extend_up(-spacing);
            self.view_zoom.value.set(editor.view_zoom);
            update!(self.view_zoom, zoom);
            context.update_focus(self.view_zoom.state.hovered);
            editor.view_zoom = self.view_zoom.value.value();

            let _ = bar;
        }

        // let (buttons_new, side_bar) = layout::cut_top_down(side_bar, font_size * 1.5);
        // {
        //     let targets = [&mut self.new_palette, &mut self.new_light];
        //     for (pos, target) in layout::split_columns(buttons_new, 2)
        //         .into_iter()
        //         .zip(targets)
        //     {
        //         update!(target, pos);
        //     }

        //     if let State::Idle
        //     | State::Waypoints {
        //         state: WaypointsState::Idle,
        //         ..
        //     } = &editor.state
        //     {
        //         self.new_light.show();
        //     } else {
        //         self.new_light.hide();
        //     }

        //     if let State::Idle = &editor.state {
        //         self.new_palette.show();
        //     } else {
        //         self.new_palette.hide();
        //     }

        //     if self.new_light.text.state.clicked {
        //         match &mut editor.state {
        //             State::Idle => {
        //                 if self.new_selector.visible {
        //                     self.new_selector.hide();
        //                     self.new_circle.hide();
        //                     self.new_line.hide();
        //                 } else {
        //                     self.new_selector.show();
        //                     self.new_circle.show();
        //                     self.new_line.show();
        //                 }
        //             }
        //             State::Waypoints {
        //                 state: state @ WaypointsState::Idle,
        //                 ..
        //             } => {
        //                 *state = WaypointsState::New;
        //             }
        //             _ => {}
        //         }
        //     } else if self.new_palette.text.state.clicked {
        //         editor.palette_swap();
        //     }
        // }

        // {
        //     // Selector
        //     let (selector, _) = layout::cut_top_down(side_bar, side_bar.width());
        //     let targets = [&mut self.new_circle, &mut self.new_line];
        //     for (pos, target) in layout::split_rows(selector, 2).into_iter().zip(targets) {
        //         update!(target, pos);
        //         if target.state.clicked {
        //             editor.state = State::Place {
        //                 shape: target.light.shape,
        //                 danger: false,
        //             };
        //         }
        //     }
        // }

        // {
        //     update!(self.selected_light, side_bar);

        //     let light_size = self.selected_light.light.state.position.size();
        //     self.light_size = light_size.map(|x| x.round() as usize);

        //     let target = side_bar;
        //     update!(
        //         self.selected_text,
        //         layout::fit_aabb_width(vec2(target.width(), font_size), target, 1.0)
        //     );
        // }

        {
            let selected = if let State::Place { shape, danger } = &mut editor.state {
                // Place new
                let light = LightSerde {
                    danger: *danger,
                    shape: shape.scaled(editor.place_scale),
                    movement: Movement {
                        initial: Transform {
                            rotation: editor.place_rotation,
                            ..default()
                        },
                        ..default()
                    },
                };
                Some(("Left click to place a new light", danger, light))
            } else if let Some(selected_event) = editor
                .selected_light
                .and_then(|i| editor.level.level.events.get_mut(i.event))
            {
                if let Event::Light(event) = &mut selected_event.event {
                    let light = event.light.clone();
                    Some(("Selected light", &mut event.light.danger, light))
                } else {
                    None
                }
            } else {
                None
            };

            match selected {
                None => {
                    self.selected_text.hide();
                    self.selected_light.hide();
                }
                Some((text, danger, light)) => {
                    // Selected light
                    self.selected_text.show();
                    self.selected_text.text = text.to_owned();
                    self.selected_light.show();

                    if self.selected_light.danger.check.clicked {
                        *danger = !*danger;
                    }
                    self.selected_light.danger.checked = *danger;

                    let scale = match light.shape {
                        Shape::Circle { radius } => format!("{:.1}", radius),
                        Shape::Line { width } => format!("{:.1}", width),
                        Shape::Rectangle { width, height } => format!("{:.1}x{:.1}", width, height),
                    };
                    self.selected_light.scale.text = format!("{} Scale", scale);
                    let fade_out = light.movement.fade_out;
                    let fade_in = light.movement.fade_in;
                    self.selected_light.fade_in.text = format!("{:.1} Fade in time", fade_in);
                    self.selected_light.fade_out.text = format!("{:.1} Fade out time", fade_out);
                    self.selected_light.light.light = light;
                }
            }
        }

        {
            let (current_beat, bottom_bar) = layout::cut_top_down(bottom_bar, font_size * 1.5);
            update!(self.current_beat, current_beat);
            self.current_beat.text = format!("Beat: {:.2}", editor.current_beat);

            let (timeline, _bottom_bar) = layout::cut_top_down(bottom_bar, font_size * 1.0);
            let was_pressed = self.timeline.state.pressed;
            update!(self.timeline, timeline);

            if self.timeline.state.pressed {
                let time = self.timeline.get_cursor_time();
                editor.scroll_time(time - editor.current_beat);
            }
            let replay = editor
                .dynamic_segment
                .as_ref()
                .map(|replay| replay.current_beat);
            self.timeline.update_time(editor.current_beat, replay);

            let select = geng_utils::key::is_key_pressed(geng.window(), [Key::ControlLeft]);
            if select {
                if !was_pressed && self.timeline.state.pressed {
                    self.timeline.start_selection();
                } else if was_pressed && !self.timeline.state.pressed {
                    let (start_beat, end_beat) = self.timeline.end_selection();
                    if start_beat != end_beat {
                        editor.dynamic_segment = Some(Replay {
                            start_beat,
                            end_beat,
                            current_beat: start_beat,
                            speed: Time::ONE,
                        });
                    }
                }
            }

            self.timeline.auto_scale(editor.level.level.last_beat());
        }

        context.can_focus
    }
}
