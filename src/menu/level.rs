mod ui;

pub use self::ui::*;

use super::*;

use crate::render::menu::MenuRender;

use geng::MouseButton;

pub struct LevelMenu {
    geng: Geng,
    assets: Rc<Assets>,
    transition: Option<geng::state::Transition>,
    render: MenuRender,

    theme: Theme,
    groups: Vec<GroupEntry>,

    ui: MenuUI,
    cursor_pos: vec2<f64>,
}

pub struct GroupEntry {
    pub meta: GroupMeta,
    pub logo: Option<ugli::Texture>,
    /// Path to the group directory.
    pub path: std::path::PathBuf,
}

impl Debug for GroupEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GroupEntry")
            .field("meta", &self.meta)
            .field("logo", &self.logo.as_ref().map(|_| "<logo>"))
            .field("path", &self.path)
            .finish()
    }
}

impl LevelMenu {
    pub fn new(geng: &Geng, assets: &Rc<Assets>, groups: Vec<GroupEntry>) -> Self {
        Self {
            geng: geng.clone(),
            assets: assets.clone(),
            transition: None,
            render: MenuRender::new(geng, assets),

            theme: Theme::default(),
            groups,

            ui: MenuUI::new(),
            cursor_pos: vec2::ZERO,
        }
    }
}

impl geng::State for LevelMenu {
    fn transition(&mut self) -> Option<geng::state::Transition> {
        self.transition.take()
    }

    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        ugli::clear(framebuffer, Some(self.theme.dark), None, None);

        self.ui.layout(
            &self.groups,
            Aabb2::ZERO.extend_positive(framebuffer.size().as_f32()),
            self.cursor_pos.as_f32(),
            geng_utils::key::is_key_pressed(self.geng.window(), [MouseButton::Left]),
            &self.geng,
        );
        self.render
            .draw_ui(&self.ui, &self.theme, &self.groups, framebuffer);
    }

    fn handle_event(&mut self, event: geng::Event) {
        if let geng::Event::CursorMove { position } = event {
            self.cursor_pos = position;
        }
    }
}
