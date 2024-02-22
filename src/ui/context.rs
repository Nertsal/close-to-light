use crate::prelude::Theme;

use geng::prelude::*;

#[derive(Default, Debug, Clone, Copy)]
pub struct KeyModifiers {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
}

impl KeyModifiers {
    pub fn from_window(window: &geng::Window) -> Self {
        Self {
            shift: geng_utils::key::is_key_pressed(window, [geng::Key::ShiftLeft]),
            ctrl: geng_utils::key::is_key_pressed(window, [geng::Key::ControlLeft]),
            alt: geng_utils::key::is_key_pressed(window, [geng::Key::AltLeft]),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CursorContext {
    pub position: vec2<f32>,
    pub down: bool,
    /// Was the cursor down last frame.
    pub was_down: bool,
    pub scroll: f32,
}

impl CursorContext {
    pub fn new() -> Self {
        Self {
            position: vec2::ZERO,
            down: false,
            was_down: false,
            scroll: 0.0,
        }
    }

    pub fn update(&mut self, is_down: bool) {
        self.was_down = self.down;
        self.down = is_down;
    }
}

#[derive(Debug, Clone)]
pub struct UiContext {
    pub theme: Theme,
    pub layout_size: f32,
    pub font_size: f32,
    /// Whether the widget can use the cursor position to get focus.
    pub can_focus: bool,
    pub cursor: CursorContext,
    pub text_edit: TextEdit,
    pub delta_time: f32,
    /// Active key modifiers.
    pub mods: KeyModifiers,
}

#[derive(Clone)]
pub struct TextEdit {
    geng: Option<Geng>,
    /// Counter for the number of text edits.
    counter: usize,
    pub text: String,
}

impl Debug for TextEdit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TextEdit")
            .field("geng", &"<reference>")
            .finish()
    }
}

impl TextEdit {
    pub fn new(geng: &Geng) -> Self {
        Self {
            geng: Some(geng.clone()),
            counter: 0,
            text: String::new(),
        }
    }

    pub fn empty() -> Self {
        Self {
            geng: None,
            counter: 0,
            text: String::new(),
        }
    }

    /// Starts the text edit and returns the id.
    pub fn edit(&mut self, text: &str) -> usize {
        if let Some(geng) = &self.geng {
            if geng.window().is_editing_text() {
                geng.window().stop_text_edit();
                self.counter += 1;
            }
            geng.window().start_text_edit(text);
            self.text = text.to_owned();
            self.counter
        } else {
            0
        }
    }

    /// Stop editing text.
    pub fn stop(&mut self) {
        if let Some(geng) = &self.geng {
            if !geng.window().is_editing_text() {
                return;
            }
            geng.window().stop_text_edit();
            self.counter += 1;
        }
    }

    /// Check if the id is an active edit.
    pub fn is_active(&self, id: usize) -> bool {
        self.geng.is_some() && id == self.counter
    }
}

impl UiContext {
    pub fn new(geng: &Geng, theme: Theme) -> Self {
        Self {
            theme,
            layout_size: 1.0,
            font_size: 1.0,
            can_focus: true,
            cursor: CursorContext::new(),
            text_edit: TextEdit::new(geng),
            delta_time: 0.1,
            mods: KeyModifiers::default(),
        }
    }

    pub fn scale_font(&self, scale: f32) -> Self {
        // TODO: cloning doesnt work for text edit
        Self {
            font_size: self.font_size * scale,
            ..self.clone()
        }
    }

    /// Update `can_focus` property given another widget's focus.
    pub fn update_focus(&mut self, focus: bool) {
        self.can_focus = self.can_focus && !focus;
    }

    /// Should be called before layout.
    /// Updates input values.
    pub fn update(&mut self, window: &geng::Window, delta_time: f32) {
        self.delta_time = delta_time;
        self.cursor.update(geng_utils::key::is_key_pressed(
            window,
            [geng::MouseButton::Left],
        ));
        self.mods = KeyModifiers::from_window(window);
    }

    /// Should be called after the layout.
    /// Reset accumulators to prepare for the next frame.
    pub fn frame_end(&mut self) {
        self.can_focus = true;
        self.cursor.scroll = 0.0
    }
}
