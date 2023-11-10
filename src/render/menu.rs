use super::{
    util::{TextRenderOptions, UtilRender},
    *,
};

use crate::menu::{GroupEntry, MenuUI};

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

    pub fn draw_ui(
        &mut self,
        ui: &MenuUI,
        theme: &Theme,
        groups: &[GroupEntry],
        framebuffer: &mut ugli::Framebuffer,
    ) {
        let font_size = framebuffer.size().y as f32 * 0.04;
        let options = TextRenderOptions::new(font_size)
            .color(theme.light)
            .align(vec2(0.5, 1.0));

        let camera = &geng::PixelPerfectCamera;

        geng_utils::texture::draw_texture_fit_height(
            &self.assets.sprites.title,
            ui.ctl_logo.position,
            0.5,
            camera,
            &self.geng,
            framebuffer,
        );

        for (group, entry) in ui.groups.iter().zip(groups) {
            if let Some(logo) = &entry.logo {
                self.geng.draw2d().textured_quad(
                    framebuffer,
                    camera,
                    group.logo.position,
                    logo,
                    Color::WHITE,
                );
            }
            self.util
                .draw_text_widget(&group.name, options.align(vec2(0.0, 0.5)), framebuffer);

            let state = &group.state;
            let color = if state.pressed {
                theme.light.map_rgb(|x| x * 0.5)
            } else if state.hovered {
                theme.light.map_rgb(|x| x * 0.7)
            } else {
                theme.light
            };
            self.util.draw_outline(
                &Collider::aabb(state.position.map(r32)),
                font_size * 0.2,
                color,
                camera,
                framebuffer,
            );
        }
    }
}
