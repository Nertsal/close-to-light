use super::{util::UtilRender, *};

use crate::menu::MenuState;

pub struct MenuRender {
    geng: Geng,
    assets: Rc<Assets>,
    util: UtilRender,
}

impl MenuRender {
    pub fn new(geng: &Geng, assets: &Rc<Assets>) -> Self {
        Self {
            geng: geng.clone(),
            assets: assets.clone(),
            util: UtilRender::new(geng, assets),
        }
    }

    pub fn draw_menu(&mut self, state: &MenuState, framebuffer: &mut ugli::Framebuffer) {}
}
