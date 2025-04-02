use super::{geometry::GeometryContext, state::UiState};

use crate::{
    assets::Font,
    prelude::{Context, Theme},
};

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
    /// Position set outside of the update, synchronized in the update.
    next_position: vec2<f32>,
    pub position: vec2<f32>,
    /// Cursor position last frame.
    pub last_position: vec2<f32>,
    pub down: bool,
    pub right_down: bool,
    /// Was the cursor down last frame.
    pub was_down: bool,
    /// Was the right cursor down last frame.
    pub was_right_down: bool,
    pub scroll: f32,
}

impl CursorContext {
    pub fn new() -> Self {
        Self {
            next_position: vec2::ZERO,
            position: vec2::ZERO,
            last_position: vec2::ZERO,
            down: false,
            right_down: false,
            was_down: false,
            was_right_down: false,
            scroll: 0.0,
        }
    }

    pub fn scroll_dir(&self) -> i64 {
        if self.scroll == 0.0 {
            0
        } else {
            self.scroll.signum() as i64
        }
    }

    pub fn cursor_move(&mut self, pos: vec2<f32>) {
        self.next_position = pos;
    }

    pub fn reset(&mut self) {
        *self = Self::new();
    }

    /// Returns the delta the cursor has travelled since last frame.
    pub fn delta(&self) -> vec2<f32> {
        self.position - self.last_position
    }

    pub fn update(&mut self, is_down: bool, right_down: bool) {
        self.last_position = self.position;
        self.position = self.next_position;
        self.was_down = self.down;
        self.was_right_down = self.right_down;
        self.down = is_down;
        self.right_down = right_down;
    }
}

#[derive(Clone)]
pub struct UiContext {
    pub context: Context,
    pub geometry: GeometryContext,
    pub font: Rc<Font>,

    pub state: UiState,
    pub cursor: CursorContext,

    pub text_edit: TextEdit,
    /// Active key modifiers.
    pub mods: KeyModifiers,
    /// Whether the widget can use the cursor position to get focus.
    pub can_focus: RefCell<bool>,

    pub total_focus: RefCell<bool>,
    pub cancel_focus: RefCell<bool>,

    pub real_time: f32,
    pub delta_time: f32,
    pub screen: Aabb2<f32>,
    pub layout_size: f32,
    pub font_size: f32,
}

#[derive(Clone)]
pub struct TextEdit(Rc<RefCell<TextEditImpl>>);

struct TextEditImpl {
    geng: Option<Geng>,
    /// Counter for the number of text edits.
    counter: usize,
    text: String,
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
        Self(Rc::new(RefCell::new(TextEditImpl {
            geng: Some(geng.clone()),
            counter: 0,
            text: String::new(),
        })))
    }

    pub fn get_text(&self) -> String {
        self.0.borrow().text.clone()
    }

    pub fn set_text(&self, text: String) {
        self.0.borrow_mut().text = text;
    }

    /// Starts the text edit and returns the id.
    pub fn edit(&self, text: &str) -> usize {
        let mut inner = self.0.borrow_mut();
        if let Some(geng) = inner.geng.clone() {
            if geng.window().is_editing_text() {
                geng.window().stop_text_edit();
                inner.counter += 1;
            }
            geng.window().start_text_edit(text);
            text.clone_into(&mut inner.text);
            inner.counter
        } else {
            0
        }
    }

    /// Stop editing text.
    pub fn stop(&self) {
        let mut inner = self.0.borrow_mut();
        if let Some(geng) = &inner.geng {
            if !geng.window().is_editing_text() {
                return;
            }
            geng.window().stop_text_edit();
            inner.counter += 1;
        }
    }

    /// Check if the id is an active edit.
    pub fn is_active(&self, id: usize) -> bool {
        let inner = self.0.borrow();
        inner.geng.is_some() && id == inner.counter
    }

    /// Check if there is an active edit.
    pub fn any_active(&self) -> bool {
        self.0
            .borrow()
            .geng
            .as_ref()
            .is_some_and(|geng| geng.window().is_editing_text())
    }
}

impl UiContext {
    pub fn new(context: Context) -> Self {
        Self {
            geometry: GeometryContext::new(context.assets.clone()),
            font: context.assets.fonts.default.clone(),

            state: UiState::new(),
            cursor: CursorContext::new(),

            text_edit: TextEdit::new(&context.geng),
            mods: KeyModifiers::default(),
            can_focus: true.into(),

            total_focus: false.into(),
            cancel_focus: false.into(),

            screen: Aabb2::ZERO.extend_positive(vec2(1.0, 1.0)),
            layout_size: 1.0,
            font_size: 1.0,
            real_time: 0.0,
            delta_time: 0.1,
            context,
        }
    }

    pub fn theme(&self) -> Theme {
        self.context.get_options().theme
    }

    // TODO: better
    pub fn scale_font(&self, scale: f32) -> Self {
        // TODO: cloning doesnt work for text edit
        Self {
            font_size: self.font_size * scale,
            ..self.clone()
        }
    }

    pub fn can_focus(&self) -> bool {
        *self.can_focus.borrow()
    }

    pub fn reset_focus(&self) {
        *self.can_focus.borrow_mut() = true;
        *self.total_focus.borrow_mut() = false;
    }

    /// Update `can_focus` property given another widget's focus.
    pub fn update_focus(&self, take_focus: bool) {
        let mut focus = self.can_focus.borrow_mut();
        *focus = *focus && !take_focus;
    }

    /// Whether any of the widgets requested total focus.
    pub fn is_totally_focused(&self) -> bool {
        *self.total_focus.borrow()
    }

    /// Cancel total widget focus.
    pub fn cancel_total_focus(&self) {
        *self.cancel_focus.borrow_mut() = true;
    }

    /// Request total focus on the widget.
    /// If a cancel was called since last frame, `true` is returned.
    pub fn total_focus(&self) -> bool {
        let mut cancel = self.cancel_focus.borrow_mut();
        let cancel = std::mem::take(&mut *cancel);
        if !cancel {
            *self.total_focus.borrow_mut() = true;
        }
        cancel
    }

    /// Should be called before layout.
    /// Updates input values.
    // TODO: use window from context
    pub fn update(&mut self, delta_time: f32) {
        self.reset_focus();

        self.real_time += delta_time;
        self.delta_time = delta_time;
        let window = self.context.geng.window();
        self.cursor.update(
            geng_utils::key::is_key_pressed(window, [geng::MouseButton::Left]),
            geng_utils::key::is_key_pressed(window, [geng::MouseButton::Right]),
        );
        self.mods = KeyModifiers::from_window(window);
    }

    /// Should be called after the layout.
    /// Reset accumulators to prepare for the next frame.
    pub fn frame_end(&mut self) {
        *self.cancel_focus.borrow_mut() = false;
        self.cursor.scroll = 0.0
    }
}
