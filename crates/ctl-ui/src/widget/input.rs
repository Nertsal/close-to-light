use super::*;

use crate::layout::AreaOps;

use ctl_core::{prelude::Color, types::Name};

pub struct InputWidget {
    pub state: WidgetState,
    pub name: TextWidget,
    pub text: TextWidget,
    pub edit_id: Option<usize>,
    pub raw: String,
    pub editing: bool,

    pub hide_input: bool,
    pub format: InputFormat,
    pub layout_vertical: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum InputFormat {
    Any,
    Integer { signed: bool },
    Float { precision: usize },
    Ratio,
}

impl InputFormat {
    pub fn fix(&self, s: &str) -> String {
        match *self {
            InputFormat::Any => s.to_owned(),
            InputFormat::Integer { signed } => {
                let (s, negative) = if signed {
                    match s.strip_prefix("-") {
                        Some(s) => (s, true),
                        None => (s, false),
                    }
                } else {
                    (s, false)
                };
                let mut s = s.replace(|c: char| !c.is_ascii_digit(), "");
                if negative {
                    s = "-".to_string() + &s;
                }
                s
            }
            InputFormat::Float { precision } => {
                if precision == 0 {
                    return InputFormat::Integer { signed: true }.fix(s);
                }
                let (s, negative) = match s.strip_prefix("-") {
                    Some(s) => (s, true),
                    None => (s, false),
                };
                let mut s = s.replace(|c: char| c != '.' && !c.is_ascii_digit(), "");
                if let Some((a, b)) = s.split_once('.') {
                    let fraction = b.replace('.', "");
                    let precision = precision.max(fraction.len());
                    s = a.to_owned() + "." + &fraction[..precision];
                }
                if negative {
                    s = "-".to_string() + &s;
                }
                s
            }
            InputFormat::Ratio => {
                let fix_num = |s| InputFormat::Integer { signed: false }.fix(s);

                if let Some((num, den)) = s.split_once('/') {
                    let mut s = fix_num(num);
                    s.push('/');
                    s += &fix_num(den);
                    return s;
                }
                fix_num(s)
            }
        }
    }
}

impl InputWidget {
    pub fn new(name: impl Into<Name>) -> Self {
        Self {
            state: WidgetState::new(),
            name: TextWidget::new(name),
            text: TextWidget::new(""),
            edit_id: None,
            raw: String::new(),
            editing: false,

            hide_input: false,
            format: InputFormat::Any,
            layout_vertical: false,
        }
    }

    pub fn format(self, format: InputFormat) -> Self {
        Self { format, ..self }
    }

    pub fn vertical(self) -> Self {
        Self {
            layout_vertical: true,
            ..self
        }
    }

    // pub fn hide_input(self) -> Self {
    //     Self {
    //         hide_input: true,
    //         ..self
    //     }
    // }

    pub fn sync(&mut self, text: &str, context: &UiContext) {
        if self.raw == text {
            return;
        }

        text.clone_into(&mut self.raw);
        self.text.text = self.raw.clone().into();

        self.editing = self
            .edit_id
            .is_some_and(|id| context.text_edit.is_active(id));
        if self.editing {
            self.edit_id = Some(context.text_edit.edit(&self.raw));
        }
    }

    pub fn update(&mut self, position: Aabb2<f32>, context: &UiContext) {
        self.state.update(position, context);

        if self.state.mouse_left.clicked {
            self.edit_id = Some(context.text_edit.edit(&self.text.text));
        }

        self.editing = if self
            .edit_id
            .is_some_and(|id| context.text_edit.is_active(id))
        {
            if self.raw != context.text_edit.get_text() {
                self.raw = context.text_edit.get_text();
                self.raw = self.format.fix(&self.raw);
                self.edit_id = Some(context.text_edit.edit(&self.raw));

                self.text.text = if self.hide_input {
                    "*".repeat(self.raw.len()).into()
                } else {
                    self.raw.clone().into()
                };
            }
            true
        } else {
            false
        };

        let mut main = position;

        if self.layout_vertical {
            if !self.name.text.is_empty() {
                let name = main.split_top(0.5);
                self.name.align(vec2(0.5, 0.5));
                self.name.update(name, context);
            }
            self.text.align(vec2(0.5, 0.5));
            self.text.update(main, context);
        } else {
            if !self.name.text.is_empty() {
                let name_width = (context.layout_size * 5.0).min(main.width() / 2.0);
                let name = main.cut_left(name_width);
                self.name.align(vec2(0.0, 0.5));
                self.name.update(name, context);
                self.text.align(vec2(1.0, 0.5));
            } else {
                self.text.align(vec2(0.5, 0.5));
            }
            self.text.update(main, context);
        }

        let theme = context.theme();
        self.name.options.color = theme.light;
        self.text.options.color = if self.editing {
            theme.highlight
        } else {
            theme.light
        };
    }

    pub fn draw_colored(&self, context: &UiContext, color: Color) -> Geometry {
        let theme = context.theme();
        let mut geometry = self.name.draw_colored(context, color);
        geometry.merge(self.text.draw_colored(context, color));
        if self.editing {
            let pos = self.text.state.position;
            let underline = Aabb2::point(pos.center() - vec2(0.0, context.font_size * 0.5))
                .extend_symmetric(vec2(pos.width(), context.font_size * 0.1) / 2.0);
            geometry.merge(context.geometry.quad(underline, theme.highlight));
        }
        geometry
    }
}

impl Widget for InputWidget {
    simple_widget_state!();
    fn draw(&self, context: &UiContext) -> Geometry {
        self.draw_colored(context, self.name.options.color)
    }
}
