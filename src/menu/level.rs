use super::*;

pub struct LevelMenu {
    geng: Geng,
    assets: Rc<Assets>,
    transition: Option<geng::state::Transition>,
    dither: DitherRender,
    util_render: UtilRender,

    theme: Theme,

    groups: Vec<GroupEntry>,
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
            dither: DitherRender::new(geng, assets),
            util_render: UtilRender::new(geng, assets),

            theme: Theme::default(),
            groups,
        }
    }
}

impl geng::State for LevelMenu {
    fn transition(&mut self) -> Option<geng::state::Transition> {
        self.transition.take()
    }

    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        ugli::clear(framebuffer, Some(self.theme.dark), None, None);

        println!("{:#?}", self.groups);
    }
}
