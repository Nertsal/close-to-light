use super::*;

use crate::ui::{layout::AreaOps, widget::*};

const HELP: &str = "
Scroll / Arrow keys - move through time
Hold Shift / Alt - scroll slower / faster
Space - play music
Q / E - rotate
Ctrl+Scroll - scale lights
F1 - Hide UI
";

/// Layout and state of the UI.
pub struct EditorUI {
    pub screen: WidgetState,
    pub game: WidgetState,

    pub exit: ButtonWidget,
    pub help: IconWidget,
    pub tab_edit: ButtonWidget,
    pub tab_config: ButtonWidget,

    pub unsaved: TextWidget,
    pub save: ButtonWidget,

    pub help_text: TextWidget,
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
    pub level: TextWidget,
    pub level_name: InputWidget,
    pub level_delete: ButtonWidget,
    pub level_create: ButtonWidget,
    pub all_levels: TextWidget,
    pub all_level_names: Vec<TextWidget>,

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

    pub warn_select_level: TextWidget,

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
    pub fn new(geng: &Geng, assets: &Assets) -> Self {
        Self {
            screen: default(),
            game: default(),

            exit: ButtonWidget::new("Exit"),
            help: IconWidget::new(&assets.sprites.help),
            tab_edit: ButtonWidget::new("Edit"),
            tab_config: ButtonWidget::new("Config"),

            unsaved: TextWidget::new("Save to apply changes").aligned(vec2(1.0, 0.5)),
            save: ButtonWidget::new("Save"),

            help_text: TextWidget::new(HELP).aligned(vec2(0.0, 1.0)),
            edit: EditorEditWidget::new(geng),
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
        context: &mut UiContext,
    ) -> bool {
        let screen = screen.fit_aabb(vec2(16.0, 9.0), vec2::splat(0.5));

        let font_size = screen.height() * 0.03;
        let layout_size = screen.height() * 0.03;

        context.font_size = font_size;
        context.layout_size = layout_size;

        self.screen.update(screen, context);

        {
            let max_size = screen.size() * 0.75;

            let ratio = 16.0 / 9.0;
            let max_height = max_size.y.min(max_size.x / ratio);

            let game_height = max_height;
            let game_size = vec2(game_height * ratio, game_height);

            let game = screen.align_aabb(game_size, vec2(0.5, 0.5));
            self.game.update(game, context);
        }

        let mut main = screen;

        let mut top_bar = main.cut_top(font_size * 1.5);

        let exit = top_bar.cut_left(layout_size * 5.0);
        self.exit.update(exit, context);
        if self.exit.text.state.clicked {
            editor.exit();
        }

        let help = top_bar.cut_left(layout_size * 3.0);
        self.help.update(help, context);

        let help_text = Aabb2::point(help.bottom_right())
            .extend_right(layout_size * 12.0)
            .extend_down(font_size * HELP.lines().count() as f32);
        self.help_text.update(help_text, context);
        context.update_focus(self.help_text.state.hovered);
        if self.help.state.hovered {
            self.help_text.show();
        } else if !self.help_text.state.hovered
            && !Aabb2::from_corners(
                self.help.state.position.top_left(),
                self.help_text.state.position.bottom_right(),
            )
            .contains(context.cursor.position)
        {
            self.help_text.hide();
        }

        let tabs = [&mut self.tab_edit, &mut self.tab_config];
        let tab = Aabb2::point(top_bar.bottom_left())
            .extend_positive(vec2(layout_size * 5.0, top_bar.height()));
        let tabs_pos = tab.stack(vec2(tab.width() + layout_size, 0.0), tabs.len());
        for (tab, pos) in tabs.into_iter().zip(tabs_pos) {
            tab.update(pos, context);
        }

        if self.tab_edit.text.state.clicked {
            self.edit.show();
            self.config.hide();
        } else if self.tab_config.text.state.clicked {
            self.edit.hide();
            self.config.show();
        }

        let save = top_bar.cut_right(layout_size * 5.0);
        self.save.update(save, context);
        if self.save.text.state.clicked {
            editor.save();
        }

        let unsaved = top_bar.cut_right(layout_size * 10.0);
        let changed = editor.level_edit.as_ref().map_or(false, |level_editor| {
            level_editor.model.level.level.data != level_editor.level
        });
        if changed {
            self.unsaved.show();
            self.unsaved.update(unsaved, context);
        } else {
            self.unsaved.hide();
        }

        let main = main.extend_down(-layout_size).extend_up(-layout_size * 3.0);

        if self.edit.state.visible {
            self.edit.update(main, context, editor);
        }
        if self.config.state.visible {
            self.config.update(main, context, editor);
        }

        context.can_focus
    }
}

impl EditorEditWidget {
    pub fn new(geng: &Geng) -> Self {
        Self {
            state: WidgetState::new(),

            warn_select_level: TextWidget::new("Select or create a difficulty in the Config tab"),

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
            timeline: TimelineWidget::new(geng),
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
            let size = vec2(10.0, 1.0) * context.font_size;
            let warn = position.align_aabb(size, vec2(0.5, 0.9));
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

        let bottom_bar = main.cut_bottom(font_size * 3.0);
        let mut bottom_bar = bottom_bar.extend_symmetric(-vec2(5.0, 0.0) * layout_size);

        let mut main = main
            .extend_symmetric(-vec2(1.0, 2.0) * layout_size)
            .extend_up(-layout_size * 5.0);
        let left_bar = main.cut_left(font_size * 5.0);
        let mut right_bar = main.cut_right(font_size * 5.0);

        let spacing = layout_size * 0.25;
        let title_size = font_size * 1.3;
        let button_height = font_size * 1.2;

        {
            let mut bar = left_bar;

            let event = bar.cut_top(title_size);
            update!(self.new_event, event);
            self.new_event.options.size = title_size;

            let palette = bar.cut_top(button_height);
            bar.cut_top(spacing);
            update!(self.new_palette, palette);
            if self.new_palette.text.state.clicked {
                level_editor.palette_swap();
            }

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
        }

        {
            // Spacing
            let mut bar = right_bar;

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

            right_bar = bar.cut_top(font_size * 1.5);
        }

        {
            // Light
            let selected = if let Some(selected_event) = level_editor
                .selected_light
                .and_then(|i| level_editor.level.events.get_mut(i.event))
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

                    let mut bar = right_bar;

                    let light_pos = bar.cut_top(title_size);
                    update!(self.light, light_pos);
                    self.light.options.size = title_size;

                    let danger_pos = bar.cut_top(button_height);
                    bar.cut_top(spacing);
                    update!(self.light_danger, danger_pos);
                    if self.light_danger.state.clicked {
                        light.danger = !light.danger;
                    }
                    self.light_danger.checked = light.danger;

                    let fade_in = bar.cut_top(button_height);
                    bar.cut_top(spacing);
                    update!(self.light_fade_in, fade_in, &mut light.movement.fade_in);
                    context.update_focus(self.light_fade_in.state.hovered);

                    let fade_out = bar.cut_top(button_height);
                    bar.cut_top(spacing);
                    update!(self.light_fade_out, fade_out, &mut light.movement.fade_out);
                    context.update_focus(self.light_fade_out.state.hovered);

                    bar.cut_top(-font_size * 0.5);

                    let waypoints = bar.cut_top(button_height);
                    update!(self.waypoint, waypoints);
                    if self.waypoint.text.state.clicked {
                        level_editor.view_waypoints();
                    }

                    right_bar = bar.cut_top(spacing);
                }
            }
        }

        let mut waypoint = false;
        if let Some(waypoints) = &level_editor.level_state.waypoints {
            if let Some(selected) = waypoints.selected {
                if let Some(event) = level_editor.level.events.get_mut(waypoints.event) {
                    if let Event::Light(light) = &mut event.event {
                        if let Some(frame) = light.light.movement.get_frame_mut(selected) {
                            // Waypoint
                            waypoint = true;
                            self.waypoint_scale.show();
                            self.waypoint_angle.show();

                            let mut bar = right_bar;

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
                            frame.rotation = Angle::from_degrees(r32(value));
                            context.update_focus(self.waypoint_angle.state.hovered);
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

            self.timeline.auto_scale(level_editor.level.last_beat());
        }
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
            level: TextWidget::new("Difficulty"),
            level_name: InputWidget::new("Name", false),
            level_delete: ButtonWidget::new("Delete"),
            level_create: ButtonWidget::new("Create"),
            all_levels: TextWidget::new("All Dificulties"),
            all_level_names: Vec::new(),

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

    fn state_mut(&mut self) -> &mut WidgetState {
        &mut self.state
    }

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

        if let Some(level_editor) = &mut state.level_edit {
            let level = bar.cut_top(context.font_size);
            self.level.show();
            self.level.update(level, context);

            let name = bar.cut_top(context.font_size);
            self.level_name.sync(&level_editor.name, context);
            self.level_name.show();
            self.level_name.update(name, context);
            level_editor.name.clone_from(&self.level_name.raw);
        } else {
            self.level.hide();
            self.level_name.hide();
        }

        let delete = bar.cut_top(context.font_size);
        self.level_delete.update(delete, context);
        // TODO: click action

        let create = bar.cut_top(context.font_size);
        self.level_create.update(create, context);
        // TODO: click action

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
                .map(|name| TextWidget::new(name.clone()))
                .collect();
        }
        for (i, (level, level_name)) in self.all_level_names.iter_mut().zip(names).enumerate() {
            let name = bar.cut_top(context.font_size);
            level.update(name, context);
            level.text = level_name;
            if level.state.clicked {
                state.change_level(i);
            }
        }

        let mut bar = columns[2];
        let timeline = bar.cut_top(context.font_size);
        self.timeline.update(timeline, context);

        let scroll_by = bar.cut_top(context.font_size);
        self.scroll_by
            .update(scroll_by, context, &mut state.config.scroll_normal);

        let shift_scroll = bar.cut_top(context.font_size);
        self.shift_scroll
            .update(shift_scroll, context, &mut state.config.scroll_slow);

        let alt_scroll = bar.cut_top(context.font_size);
        self.alt_scroll
            .update(alt_scroll, context, &mut state.config.scroll_fast);
    }
}
