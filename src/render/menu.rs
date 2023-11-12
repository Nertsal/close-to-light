use super::{util::UtilRender, *};

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
            self.util.draw_text_widget(&group.name, framebuffer);
            self.util.draw_text_widget(&group.author, framebuffer);

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

            self.util
                .draw_button_widget(&ui.level.level_normal, framebuffer);
            self.util
                .draw_text_widget(&ui.level.credits_normal, framebuffer);

            self.util
                .draw_button_widget(&ui.level.level_hard, framebuffer);
            self.util
                .draw_text_widget(&ui.level.credits_hard, framebuffer);

            self.util
                .draw_text_widget(&ui.level.config_title, framebuffer);
            for preset in &ui.level.presets {
                let mut button = preset.button.clone();
                button.text.state.pressed = preset.selected;
                self.util.draw_button_widget(&button, framebuffer);
            }
        }
    }
}
