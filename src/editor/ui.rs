use super::*;

#[derive(Debug)]
pub struct Widget {
    pub position: Aabb2<f32>,
    /// Whether to show the widget.
    pub visible: bool,
    pub hovered: bool,
    /// Whether user has clicked on the widget since last frame.
    pub clicked: bool,
    /// Whether user is holding the mouse button down on the widget.
    pub pressed: bool,
}

impl Widget {
    pub fn update(&mut self, position: Aabb2<f32>, cursor_position: vec2<f32>, cursor_down: bool) {
        self.position = position;
        self.hovered = self.position.contains(cursor_position);
        let was_pressed = self.pressed;
        self.pressed = cursor_down && self.hovered;
        self.clicked = !was_pressed && self.pressed;
    }

    pub fn show(&mut self) {
        self.visible = true;
    }

    pub fn hide(&mut self) {
        self.visible = false;
        self.pressed = false;
        self.clicked = false;
    }
}

impl Default for Widget {
    fn default() -> Self {
        Self {
            position: Aabb2::ZERO.extend_uniform(1.0),
            visible: true,
            hovered: false,
            clicked: false,
            pressed: false,
        }
    }
}

#[derive(Debug, Default)]
pub struct TextWidget {
    pub widget: Widget,
    pub text: String,
}

impl TextWidget {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            ..default()
        }
    }

    pub fn update(&mut self, position: Aabb2<f32>, cursor_position: vec2<f32>, cursor_down: bool) {
        self.widget.update(position, cursor_position, cursor_down);
    }

    pub fn show(&mut self) {
        self.widget.show();
    }

    pub fn hide(&mut self) {
        self.widget.hide();
    }
}

#[derive(Debug, Default)]
pub struct CheckboxWidget {
    pub text: TextWidget,
    pub check: Widget,
    pub checked: bool,
}

impl CheckboxWidget {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: TextWidget::new(text),
            ..default()
        }
    }

    pub fn update(&mut self, position: Aabb2<f32>, cursor_position: vec2<f32>, cursor_down: bool) {
        let check_size = position.size() * 0.9;
        let check_size = check_size.x.max(check_size.y);
        let check_pos = Aabb2::point(vec2(position.min.x + check_size / 2.0, position.center().y))
            .extend_uniform(check_size / 2.0);
        self.check.update(check_pos, cursor_position, cursor_down);

        let text_pos = position.extend_left(-check_size);
        self.text.update(text_pos, cursor_position, cursor_down);
    }

    pub fn show(&mut self) {
        self.text.show();
        self.check.show();
    }

    pub fn hide(&mut self) {
        self.text.hide();
        self.check.hide();
    }
}

#[derive(Debug)]
pub struct LightWidget {
    pub widget: Widget,
    pub light: LightSerde,
}

impl LightWidget {
    pub fn new() -> Self {
        Self {
            widget: Widget::default(),
            light: LightSerde {
                position: vec2::ZERO,
                danger: false,
                rotation: Coord::ZERO,
                shape: Shape::Circle { radius: r32(1.0) },
                movement: Movement::default(),
            },
        }
    }

    pub fn update(&mut self, position: Aabb2<f32>, cursor_position: vec2<f32>, cursor_down: bool) {
        self.widget.update(position, cursor_position, cursor_down);
    }

    pub fn show(&mut self) {
        self.widget.show();
    }

    pub fn hide(&mut self) {
        self.widget.hide();
    }
}

/// Layout and state of the UI.
pub struct EditorUI {
    pub screen: Widget,
    pub game: Widget,
    pub level_info: Widget,
    pub general: Widget,
    pub selected: Widget,
    pub selected_text: TextWidget,
    pub selected_light: LightWidget,
    /// The size for the light texture to render pixel-perfectly.
    pub light_size: vec2<usize>,
    pub current_beat: TextWidget,
    pub visualize_beat: CheckboxWidget,
    pub show_grid: CheckboxWidget,
    pub snap_grid: CheckboxWidget,
    pub danger: CheckboxWidget,
}

impl EditorUI {
    pub fn new() -> Self {
        Self {
            screen: default(),
            game: default(),
            level_info: default(),
            general: default(),
            selected: default(),
            selected_text: default(),
            selected_light: LightWidget::new(),
            light_size: vec2(1, 1),
            current_beat: default(),
            visualize_beat: CheckboxWidget::new("Show movement"),
            show_grid: CheckboxWidget::new("Show grid"),
            snap_grid: CheckboxWidget::new("Snap to grid"),
            danger: CheckboxWidget::new("Danger"),
        }
    }

    pub fn layout(
        &mut self,
        editor: &mut Editor,
        render_options: &mut RenderOptions,
        screen: Aabb2<f32>,
        cursor_pos: vec2<f32>,
        cursor_down: bool,
    ) {
        macro_rules! update {
            ($widget:expr, $position:expr) => {{
                $widget.update($position, cursor_pos, cursor_down);
            }};
        }

        let screen = geng_utils::layout::fit_aabb(vec2(16.0, 9.0), screen, vec2::splat(0.5));
        update!(self.screen, screen);

        let font_size = screen.height() * 0.04;
        let checkbox_size = font_size * 0.6;

        {
            let max_size = screen.size() * 0.8;

            let ratio = 16.0 / 9.0;
            let max_height = max_size.y.min(max_size.x / ratio);

            let game_height = max_height;
            let game_size = vec2(game_height * ratio, game_height);

            self.game.position = geng_utils::layout::align_aabb(game_size, screen, vec2(0.0, 1.0));
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

        {
            let size = side_bar.size() * vec2(1.0, 0.2);
            update!(
                self.level_info,
                geng_utils::layout::align_aabb(size, side_bar, vec2(0.5, 0.0))
            );
        }

        {
            let size = side_bar.size() * vec2(1.0, 0.3);
            update!(
                self.general,
                geng_utils::layout::align_aabb(
                    size,
                    side_bar.extend_down(-self.level_info.position.height()),
                    vec2(0.5, 0.0),
                )
            );

            let mut pos = vec2(
                self.general.position.min.x + font_size,
                self.general.position.max.y - font_size,
            );
            for (target, value) in [
                (&mut self.visualize_beat, &mut editor.visualize_beat),
                (&mut self.show_grid, &mut render_options.show_grid),
                (&mut self.snap_grid, &mut editor.snap_to_grid),
            ] {
                update!(
                    target,
                    Aabb2::point(pos).extend_uniform(checkbox_size / 2.0)
                );
                if target.check.clicked {
                    *value = !*value;
                }
                target.checked = *value;
                pos -= vec2(0.0, font_size);
            }
        }

        {
            let size = side_bar.size() * vec2(1.0, 0.45);
            update!(
                self.selected,
                geng_utils::layout::align_aabb(size, side_bar, vec2(0.5, 1.0))
            );

            update!(self.selected_light, self.selected.position);

            let target = self.selected.position;
            update!(
                self.selected_text,
                geng_utils::layout::fit_aabb_width(vec2(target.width(), font_size), target, 1.0)
            );
        }

        {
            let pos = vec2(
                self.selected.position.min.x,
                self.selected.position.max.y - self.light_size.y as f32,
            ) + vec2(1.0, -1.0) * font_size;
            let danger = Aabb2::point(pos).extend_uniform(checkbox_size / 2.0);
            update!(self.danger, danger);
        }

        {
            let selected = if let State::Place { shape, danger } = &mut editor.state {
                // Place new
                let light = LightSerde {
                    position: vec2::ZERO,
                    danger: *danger,
                    rotation: editor.place_rotation.as_degrees(),
                    shape: *shape,
                    movement: Movement::default(),
                };
                Some(("Left click to place a new light", danger, light))
            } else if let Some(selected_event) = editor
                .selected_light
                .and_then(|i| editor.level_state.light_event(i))
                .and_then(|i| editor.level.events.get_mut(i))
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
                    self.selected.hide();
                    self.selected_text.hide();
                    self.selected_light.hide();
                    self.danger.hide();
                }
                Some((text, danger, light)) => {
                    // Selected light
                    self.selected.show();
                    self.selected_text.show();
                    self.selected_text.text = text.to_owned();
                    self.selected_light.show();
                    self.selected_light.light = light;
                    self.danger.show();

                    if self.danger.check.clicked {
                        *danger = !*danger;
                    }
                    self.danger.checked = *danger;
                }
            }
        }

        {
            let size = self.selected.position.width() * 0.5;
            let size = vec2::splat(size);
            self.light_size = size.map(|x| x.round() as usize);
        }

        {
            let size = bottom_bar.height() * 0.2;
            let size = vec2::splat(size);
            update!(
                self.current_beat,
                geng_utils::layout::align_aabb(size, bottom_bar, vec2(0.5, 1.0))
            );
            self.current_beat.text = format!("Beat: {:.2}", editor.current_beat);
        }
    }
}
