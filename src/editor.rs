use crate::{assets::*, model::*};

use geng::prelude::*;

pub struct Editor {
    geng: Geng,
    assets: Rc<Assets>,
    level: Level,
}

impl Editor {
    pub fn new(geng: Geng, assets: Rc<Assets>, level: Level) -> Self {
        Self {
            geng,
            assets,
            level,
        }
    }
}

impl geng::State for Editor {
    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        ugli::clear(framebuffer, Some(crate::render::COLOR_DARK), None, None);
    }
}
