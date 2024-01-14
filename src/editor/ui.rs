use super::*;

use crate::ui::{layout, widget::*};

/// Layout and state of the UI.
pub struct EditorUI {
    pub screen: WidgetState,
    pub game: WidgetState,

    pub help: TextWidget,
    pub tab_edit: ButtonWidget,
    pub tab_config: ButtonWidget,

    pub edit: EditorEditWidget,
    pub config: EditorConfigWidget,
}

pub struct EditorConfigWidget {
    pub state: WidgetState,

    pub timing: TextWidget,
    pub bpm: ValueWidget<Time>,
    // pub tempo:
    pub offset: ValueWidget<Time>,

    pub music: TextWidget,
    pub level: TextWidget, // TODO: input

    pub timeline: TextWidget,
    /// Normal time scroll.
    pub scroll_by: ValueWidget<Time>, // TODO: 1/4 instead of 0.25
    /// Slow time scroll.
    pub shift_scroll: ValueWidget<Time>,
    /// Fast time scroll.
    pub alt_scroll: ValueWidget<Time>,
    // pub snap_to: CheckboxWidget,
}

pub struct EditorEditWidget {
    pub state: WidgetState,

    pub new_event: TextWidget,
    pub new_palette: ButtonWidget,
    pub new_circle: ButtonWidget,
    pub new_line: ButtonWidget,

    pub view: TextWidget,
    pub visualize_beat: CheckboxWidget,
    pub show_grid: CheckboxWidget,
    pub view_zoom: ValueWidget<f32>,

    pub placement: TextWidget,
    pub snap_grid: CheckboxWidget,
    pub grid_size: ValueWidget<f32>,

    pub light: TextWidget,
    pub light_danger: CheckboxWidget,
    pub light_fade_in: ValueWidget<Time>,
    pub light_fade_out: ValueWidget<Time>,

    pub waypoint: ButtonWidget,
    pub waypoint_scale: ValueWidget<f32>,
    /// Angle in degrees.
    pub waypoint_angle: ValueWidget<f32>,

    pub current_beat: TextWidget,
    pub timeline: TimelineWidget,
}

impl EditorUI {
    pub fn new() -> Self {
        Self {
            screen: default(),
            game: default(),

            help: TextWidget::new("?"),
            tab_edit: ButtonWidget::new("Edit"),
            tab_config: ButtonWidget::new("Config"),

            edit: EditorEditWidget::new(),
            config: {
                let mut w = EditorConfigWidget::new();
                w.hide();
                w
            },
        }
    }

    pub fn layout(
        &mut self,
        editor: &mut Editor,
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
            mods: KeyModifiers::from_window(geng.window()),
        };

        self.screen.update(screen, &context);

        {
            let max_size = screen.size() * 0.75;

            let ratio = 16.0 / 9.0;
            let max_height = max_size.y.min(max_size.x / ratio);

            let game_height = max_height;
            let game_size = vec2(game_height * ratio, game_height);

            let game = layout::align_aabb(game_size, screen, vec2(0.5, 0.5));
            self.game.update(game, &context);
        }

        let main = screen;

        let (top_bar, main) = layout::cut_top_down(main, font_size * 1.5);

        let (help, top_bar) = layout::cut_left_right(top_bar, layout_size * 3.0);
        let tabs = [&mut self.tab_edit, &mut self.tab_config];
        let tab = Aabb2::point(top_bar.bottom_left())
            .extend_positive(vec2(layout_size * 5.0, top_bar.height()));
        let tabs_pos = layout::stack(tab, vec2(tab.width() + layout_size, 0.0), tabs.len());
        for (tab, pos) in tabs.into_iter().zip(tabs_pos) {
            tab.update(pos, &mut context);
        }

        if self.tab_edit.text.state.clicked {
            self.edit.show();
            self.config.hide();
        } else if self.tab_config.text.state.clicked {
            self.edit.hide();
            self.config.show();
        }

        let main = main.extend_down(-layout_size).extend_up(-layout_size * 3.0);

        self.edit.update(main, &mut context, editor);
        self.config.update(main, &mut context, editor);

        context.can_focus
    }
}

impl EditorEditWidget {
    pub fn new() -> Self {
        Self {
            state: WidgetState::new(),

            new_event: TextWidget::new("Event"),
            new_palette: ButtonWidget::new("Palette Swap"),
            new_circle: ButtonWidget::new("Circle"),
            new_line: ButtonWidget::new("Line"),

            view: TextWidget::new("View"),
            visualize_beat: CheckboxWidget::new("Dynamic"),
            show_grid: CheckboxWidget::new("Grid"),
            view_zoom: ValueWidget::new("Zoom: ", 1.0, 0.5..=2.0, 0.25),

            placement: TextWidget::new("Placement"),
            snap_grid: CheckboxWidget::new("Grid snap"),
            grid_size: ValueWidget::new("Grid size", 16.0, 2.0..=32.0, 1.0),

            light: TextWidget::new("Light"),
            light_danger: CheckboxWidget::new("Danger"),
            light_fade_in: ValueWidget::new("Fade in", r32(1.0), r32(0.25)..=r32(10.0), r32(0.25)),
            light_fade_out: ValueWidget::new(
                "Fade out",
                r32(1.0),
                r32(0.25)..=r32(10.0),
                r32(0.25),
            ),

            waypoint: ButtonWidget::new("Waypoints"),
            waypoint_scale: ValueWidget::new("Scale", 1.0, 0.25..=2.0, 0.25),
            waypoint_angle: ValueWidget::new("Angle", 0.0, 0.0..=360.0, 15.0).wrapping(),

            current_beat: default(),
            timeline: TimelineWidget::new(),
        }
    }
}

impl StatefulWidget for EditorEditWidget {
    type State = Editor;

    fn update(&mut self, position: Aabb2<f32>, context: &mut UiContext, state: &mut Self::State) {
        let editor = state;
        let main = position;
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

        let (main, bottom_bar) = layout::cut_top_down(main, main.height() - font_size * 3.0);
        let bottom_bar = bottom_bar.extend_symmetric(-vec2(5.0, 0.0) * layout_size);

        let main = main
            .extend_symmetric(-vec2(1.0, 2.0) * layout_size)
            .extend_up(-layout_size * 5.0);
        let (left_bar, main) = layout::cut_left_right(main, font_size * 5.0);
        let (main, mut right_bar) = layout::cut_left_right(main, main.width() - font_size * 5.0);
        let _ = main;

        let spacing = layout_size * 0.25;
        let title_size = font_size * 1.3;
        let button_height = font_size * 1.2;

        {
            let bar = left_bar;

            let (event, bar) = layout::cut_top_down(bar, title_size);
            update!(self.new_event, event);
            self.new_event.options.size = title_size;

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

            let (view, bar) = layout::cut_top_down(bar, title_size);
            let bar = bar.extend_up(-spacing);
            update!(self.view, view);
            self.view.options.size = title_size;

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
                editor.render_options.show_grid = !editor.render_options.show_grid;
            }
            self.show_grid.checked = editor.render_options.show_grid;

            // let (waypoints, bar) = layout::cut_top_down(bar, button_height);
            // let bar = bar.extend_up(-spacing);
            // update!(self.view_waypoints, waypoints);
            // if self.view_waypoints.text.state.clicked {
            //     editor.view_waypoints();
            // }

            let (zoom, bar) = layout::cut_top_down(bar, font_size);
            let bar = bar.extend_up(-spacing);
            update!(self.view_zoom, zoom, &mut editor.view_zoom);
            context.update_focus(self.view_zoom.state.hovered);

            let _ = bar;
        }

        {
            // Spacing
            let bar = right_bar;

            let (placement, bar) = layout::cut_top_down(bar, title_size);
            update!(self.placement, placement);
            self.placement.options.size = title_size;

            let (grid_snap, bar) = layout::cut_top_down(bar, button_height);
            let bar = bar.extend_up(-spacing);
            update!(self.snap_grid, grid_snap);
            if self.snap_grid.state.clicked {
                editor.snap_to_grid = !editor.snap_to_grid;
            }
            self.snap_grid.checked = editor.snap_to_grid;

            let (grid_size, bar) = layout::cut_top_down(bar, button_height);
            let bar = bar.extend_up(-spacing);
            let mut value = 10.0 / editor.grid_size.as_f32();
            update!(self.grid_size, grid_size, &mut value);
            editor.grid_size = r32(10.0 / value);
            context.update_focus(self.grid_size.state.hovered);

            right_bar = bar.extend_up(-font_size * 1.5);
        }

        {
            // Light
            let selected = if let Some(selected_event) = editor
                .selected_light
                .and_then(|i| editor.level.level.events.get_mut(i.event))
            {
                if let Event::Light(event) = &mut selected_event.event {
                    Some(&mut event.light)
                } else {
                    None
                }
            } else {
                None
            };

            match selected {
                None => {
                    self.light.hide();
                    self.light_danger.hide();
                    self.light_fade_in.hide();
                    self.light_fade_out.hide();
                    self.waypoint.hide();
                }
                Some(light) => {
                    self.light.show();
                    self.light_danger.show();
                    self.light_fade_in.show();
                    self.light_fade_out.show();
                    self.waypoint.show();

                    let bar = right_bar;

                    let (light_pos, bar) = layout::cut_top_down(bar, title_size);
                    update!(self.light, light_pos);
                    self.light.options.size = title_size;

                    let (danger_pos, bar) = layout::cut_top_down(bar, button_height);
                    let bar = bar.extend_up(-spacing);
                    update!(self.light_danger, danger_pos);
                    if self.light_danger.state.clicked {
                        light.danger = !light.danger;
                    }
                    self.light_danger.checked = light.danger;

                    let (fade_in, bar) = layout::cut_top_down(bar, button_height);
                    let bar = bar.extend_up(-spacing);
                    update!(self.light_fade_in, fade_in, &mut light.movement.fade_in);
                    context.update_focus(self.light_fade_in.state.hovered);

                    let (fade_out, bar) = layout::cut_top_down(bar, button_height);
                    let bar = bar.extend_up(-spacing);
                    update!(self.light_fade_out, fade_out, &mut light.movement.fade_out);
                    context.update_focus(self.light_fade_out.state.hovered);

                    let bar = bar.extend_up(-font_size * 0.5);

                    let (waypoints, bar) = layout::cut_top_down(bar, button_height);
                    update!(self.waypoint, waypoints);
                    if self.waypoint.text.state.clicked {
                        editor.view_waypoints();
                    }

                    right_bar = bar.extend_up(-spacing);
                }
            }
        }

        let mut waypoint = false;
        if let Some(waypoints) = &editor.level_state.waypoints {
            if let Some(selected) = waypoints.selected {
                if let Some(event) = editor.level.level.events.get_mut(waypoints.event) {
                    if let Event::Light(light) = &mut event.event {
                        if let Some(frame) = light.light.movement.get_frame_mut(selected) {
                            // Waypoint
                            waypoint = true;
                            self.waypoint_scale.show();
                            self.waypoint_angle.show();

                            let bar = right_bar;

                            let (scale, bar) = layout::cut_top_down(bar, button_height);
                            let bar = bar.extend_up(-spacing);
                            let mut value = frame.scale.as_f32();
                            update!(self.waypoint_scale, scale, &mut value);
                            frame.scale = r32(value);
                            context.update_focus(self.waypoint_scale.state.hovered);

                            let (angle, bar) = layout::cut_top_down(bar, button_height);
                            let bar = bar.extend_up(-spacing);
                            let mut value = frame.rotation.as_degrees().as_f32();
                            update!(self.waypoint_angle, angle, &mut value);
                            frame.rotation = Angle::from_degrees(r32(value));
                            context.update_focus(self.waypoint_angle.state.hovered);

                            let _ = bar;
                        }
                    }
                }
            }
        }
        if !waypoint {
            self.waypoint_scale.hide();
            self.waypoint_angle.hide();
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

            let select = context.mods.ctrl;
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
    }

    fn walk_states_mut(&mut self, f: &dyn Fn(&mut WidgetState)) {
        self.state.walk_states_mut(f);
    }
}

impl EditorConfigWidget {
    pub fn new() -> Self {
        Self {
            state: WidgetState::new(),

            timing: TextWidget::new("Timing"),
            bpm: ValueWidget::new("BPM", r32(150.0), r32(60.0)..=r32(240.0), r32(1.0)), // TODO: different
            offset: ValueWidget::new("Offset", r32(0.0), r32(-10.0)..=r32(10.0), r32(0.1)),

            music: TextWidget::new("Music"),
            level: TextWidget::new("Level"),

            timeline: TextWidget::new("Timeline"),
            scroll_by: ValueWidget::new("Scroll by", r32(1.0), r32(0.25)..=r32(4.0), r32(0.25)),
            shift_scroll: ValueWidget::new(
                "Shift scroll",
                r32(0.25),
                r32(0.125)..=r32(1.0),
                r32(0.125),
            ),
            alt_scroll: ValueWidget::new("Alt scroll", r32(10.0), r32(1.0)..=r32(20.0), r32(0.5)),
        }
    }
}

impl StatefulWidget for EditorConfigWidget {
    type State = Editor;

    fn update(&mut self, position: Aabb2<f32>, context: &mut UiContext, state: &mut Self::State) {
        self.state.update(position, context);

        let main = position;

        let width = context.layout_size * 7.0;
        let spacing = context.layout_size * 5.0;

        let columns = 3;
        let total_width = columns as f32 * width + (columns - 1) as f32 * spacing;
        let column = Aabb2::point(vec2(main.center().x - total_width / 2.0, main.max.y))
            .extend_right(width)
            .extend_down(main.height());

        let columns = layout::stack(column, vec2(width + spacing, 0.0), columns);

        let bar = columns[0];
        let (timing, bar) = layout::cut_top_down(bar, context.font_size);
        self.timing.update(timing, context);

        let (bpm, bar) = layout::cut_top_down(bar, context.font_size);
        self.bpm
            .update(bpm, context, &mut state.level.group_meta.music.bpm);

        // let (offset, bar) = layout::cut_top_down(bar, context.font_size);
        // self.offset.update(offset, context);

        let _ = bar;

        let bar = columns[1];
        let (music, bar) = layout::cut_top_down(bar, context.font_size);
        self.music.text = format!("Music: {}", state.level.group_meta.name);
        self.music.update(music, context);

        let (level, bar) = layout::cut_top_down(bar, context.font_size);
        self.level.text = format!("Level: {}", state.level.level_meta.name);
        self.level.update(level, context);

        let _ = bar;

        let bar = columns[2];
        let (timeline, bar) = layout::cut_top_down(bar, context.font_size);
        self.timeline.update(timeline, context);

        let (scroll_by, bar) = layout::cut_top_down(bar, context.font_size);
        self.scroll_by
            .update(scroll_by, context, &mut state.config.scroll_normal);

        let (shift_scroll, bar) = layout::cut_top_down(bar, context.font_size);
        self.shift_scroll
            .update(shift_scroll, context, &mut state.config.scroll_slow);

        let (alt_scroll, bar) = layout::cut_top_down(bar, context.font_size);
        self.alt_scroll
            .update(alt_scroll, context, &mut state.config.scroll_fast);

        let _ = bar;
    }

    fn walk_states_mut(&mut self, f: &dyn Fn(&mut WidgetState)) {
        self.state.walk_states_mut(f);
    }
}
