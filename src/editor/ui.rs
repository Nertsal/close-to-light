use super::*;

use crate::ui::{layout, widget::*};

/// Layout and state of the UI.
pub struct EditorUI {
    pub screen: WidgetState,
    pub game: WidgetState,
    pub level_info: WidgetState,
    pub general: WidgetState,

    pub new_palette: ButtonWidget,
    pub new_light: ButtonWidget,
    pub new_selector: WidgetState,
    // TODO: dynamic
    pub new_circle: LightWidget,
    pub new_line: LightWidget,

    pub selected_text: TextWidget,
    pub selected_light: LightStateWidget,

    /// The size for the light texture to render pixel-perfectly.
    pub light_size: vec2<usize>,
    pub visualize_beat: CheckboxWidget,
    pub show_grid: CheckboxWidget,
    pub snap_grid: CheckboxWidget,

    pub current_beat: TextWidget,
    pub timeline: TimelineWidget,
}

impl EditorUI {
    pub fn new() -> Self {
        Self {
            screen: default(),
            game: default(),
            level_info: default(),
            general: default(),
            new_palette: ButtonWidget::new("Palette Swap"),
            new_light: ButtonWidget::new("New Light"),
            new_selector: {
                let mut state = WidgetState::default();
                state.hide();
                state
            },
            new_circle: LightWidget::new_shape(Shape::Circle { radius: r32(1.3) }),
            new_line: LightWidget::new_shape(Shape::Line { width: r32(1.7) }),
            selected_text: default(),
            selected_light: LightStateWidget::new(),
            light_size: vec2(1, 1),
            visualize_beat: CheckboxWidget::new("Show movement"),
            show_grid: CheckboxWidget::new("Show grid"),
            snap_grid: CheckboxWidget::new("Snap to grid"),
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
    ) {
        let screen = layout::fit_aabb(vec2(16.0, 9.0), screen, vec2::splat(0.5));

        let font_size = screen.height() * 0.03;

        let context = UiContext {
            theme: editor.model.options.theme,
            layout_size: screen.height() * 0.03,
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
            let max_size = screen.size() * 0.8;

            let ratio = 16.0 / 9.0;
            let max_height = max_size.y.min(max_size.x / ratio);

            let game_height = max_height;
            let game_size = vec2(game_height * ratio, game_height);

            update!(
                self.game,
                layout::align_aabb(game_size, screen, vec2(0.0, 1.0))
            );
        }

        let margin = screen.width().min(screen.height()) * 0.02;

        let side_bar = Aabb2 {
            min: vec2(self.game.position.max.x, screen.min.y),
            max: self.screen.position.max,
        }
        .extend_uniform(-margin);
        let bottom_bar = Aabb2 {
            min: self.screen.position.min,
            max: vec2(self.game.position.max.x, self.game.position.min.y),
        }
        .extend_uniform(-margin);

        let (side_bar, level_info) = layout::split_top_down(side_bar, 0.8);
        update!(self.level_info, level_info);

        let (side_bar, general) = layout::split_top_down(side_bar, 0.6);
        let (_, general) = layout::cut_top_down(general, font_size);

        {
            update!(self.general, general);

            let pos = layout::cut_top_down(general, font_size)
                .0
                .extend_symmetric(vec2(-font_size, 0.0));
            let targets = [
                (&mut self.visualize_beat, &mut editor.visualize_beat),
                (&mut self.show_grid, &mut render_options.show_grid),
                (&mut self.snap_grid, &mut editor.snap_to_grid),
            ];
            for (pos, (target, value)) in layout::stack(pos, vec2(0.0, -font_size), targets.len())
                .into_iter()
                .zip(targets)
            {
                update!(target, pos);
                if target.check.clicked {
                    *value = !*value;
                }
                target.checked = *value;
            }
        }

        let (buttons_new, side_bar) = layout::cut_top_down(side_bar, font_size * 1.5);
        {
            let targets = [&mut self.new_palette, &mut self.new_light];
            for (pos, target) in layout::split_columns(buttons_new, 2)
                .into_iter()
                .zip(targets)
            {
                update!(target, pos);
            }

            if let State::Idle
            | State::Waypoints {
                state: WaypointsState::Idle,
                ..
            } = &editor.state
            {
                self.new_light.show();
            } else {
                self.new_light.hide();
            }

            if let State::Idle = &editor.state {
                self.new_palette.show();
            } else {
                self.new_palette.hide();
            }

            if self.new_light.text.state.clicked {
                match &mut editor.state {
                    State::Idle => {
                        if self.new_selector.visible {
                            self.new_selector.hide();
                            self.new_circle.hide();
                            self.new_line.hide();
                        } else {
                            self.new_selector.show();
                            self.new_circle.show();
                            self.new_line.show();
                        }
                    }
                    State::Waypoints {
                        state: state @ WaypointsState::Idle,
                        ..
                    } => {
                        *state = WaypointsState::New;
                    }
                    _ => {}
                }
            } else if self.new_palette.text.state.clicked {
                editor.palette_swap();
            }
        }

        {
            // Selector
            let (selector, _) = layout::cut_top_down(side_bar, side_bar.width());
            let targets = [&mut self.new_circle, &mut self.new_line];
            for (pos, target) in layout::split_rows(selector, 2).into_iter().zip(targets) {
                update!(target, pos);
                if target.state.clicked {
                    editor.state = State::Place {
                        shape: target.light.shape,
                        danger: false,
                    };
                }
            }
        }

        {
            update!(self.selected_light, side_bar);

            let light_size = self.selected_light.light.state.position.size();
            self.light_size = light_size.map(|x| x.round() as usize);

            let target = side_bar;
            update!(
                self.selected_text,
                layout::fit_aabb_width(vec2(target.width(), font_size), target, 1.0)
            );
        }

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
                .and_then(|i| editor.level.events.get_mut(i.event))
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

            self.timeline.auto_scale(editor.level.last_beat());
        }
    }
}
