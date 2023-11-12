use super::{
    util::{TextRenderOptions, UtilRender},
    *,
};

use crate::menu::{MenuState, MenuUI};

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

    pub fn draw_ui(&mut self, ui: &MenuUI, state: &MenuState, framebuffer: &mut ugli::Framebuffer) {
        let font_size = framebuffer.size().y as f32 * 0.04;
        let options = TextRenderOptions::new(font_size)
            .color(state.theme.light)
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

        for (group, entry) in ui.groups.iter().zip(&state.groups) {
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
                .draw_text_widget(&group.name, options.align(vec2(0.0, 0.0)), framebuffer);
            self.util
                .draw_text_widget(&group.author, options.align(vec2(0.0, 1.0)), framebuffer);

            let group = &group.state;
            let color = if group.pressed {
                state.theme.light.map_rgb(|x| x * 0.5)
            } else if group.hovered {
                state.theme.light.map_rgb(|x| x * 0.7)
            } else {
                state.theme.light
            };
            self.util.draw_outline(
                &Collider::aabb(group.position.map(r32)),
                font_size * 0.2,
                color,
                camera,
                framebuffer,
            );
        }

        if ui.level.state.visible {
            self.util.draw_outline(
                &Collider::aabb(ui.level.state.position.map(r32)),
                font_size * 0.2,
                state.theme.light,
                camera,
                framebuffer,
            );

            // self.util
            //     .draw_text_widget(&ui.level.name, options.align(vec2(0.5, 0.5)), framebuffer);

            self.util
                .draw_button_widget(&ui.level.level_normal, options, framebuffer);
            self.util.draw_text_widget(
                &ui.level.credits_normal,
                options.align(vec2(0.5, 0.5)),
                framebuffer,
            );

            self.util
                .draw_button_widget(&ui.level.level_hard, options, framebuffer);
            self.util.draw_text_widget(
                &ui.level.credits_hard,
                options.align(vec2(0.5, 0.5)),
                framebuffer,
            );

            // self.util.draw_text_widget(
            //     &ui.level.music_credits,
            //     options.align(vec2(1.0, 0.5)),
            //     framebuffer,
            // );
        }
    }
}
