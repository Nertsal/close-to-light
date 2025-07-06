mod game;
mod ui;

use mask::MaskedStack;

use super::{
    dither::DitherRender,
    mask::MaskedRender,
    ui::UiRender,
    util::{TextRenderOptions, UtilRender},
    *,
};

use crate::{
    editor::{State, *},
    ui::UiContext,
};

pub struct EditorRender {
    context: Context,
    dither: DitherRender,
    util: UtilRender,
    ui: UiRender,
    mask: MaskedRender,
    mask_stack: MaskedStack,
    // unit_quad: ugli::VertexBuffer<draw2d::TexturedVertex>,
    game_texture: ugli::Texture,
    ui_texture: ugli::Texture,
    ui_depth: ugli::Renderbuffer<ugli::DepthComponent>,
    font_size: f32,
}

impl EditorRender {
    pub fn new(context: Context) -> Self {
        let mut game_texture = geng_utils::texture::new_texture(context.geng.ugli(), vec2(1, 1));
        game_texture.set_filter(ugli::Filter::Nearest);
        let mut ui_texture = geng_utils::texture::new_texture(context.geng.ugli(), vec2(1, 1));
        let ui_depth = ugli::Renderbuffer::new(context.geng.ugli(), vec2(1, 1));
        ui_texture.set_filter(ugli::Filter::Nearest);

        Self {
            dither: DitherRender::new(&context.geng, &context.assets),
            util: UtilRender::new(context.clone()),
            ui: UiRender::new(context.clone()),
            mask: MaskedRender::new(&context.geng, &context.assets, vec2(1, 1)),
            mask_stack: MaskedStack::new(&context.geng, &context.assets),
            // unit_quad: geng_utils::geometry::unit_quad_geometry(geng.ugli()),
            game_texture,
            ui_texture,
            ui_depth,
            font_size: 1.0,
            context,
        }
    }

    pub fn draw_editor(
        &mut self,
        editor: &Editor,
        ui: &EditorUi,
        context: &UiContext,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        ugli::clear(
            framebuffer,
            Some(editor.context.get_options().theme.dark),
            None,
            None,
        );

        self.mask.update_size(framebuffer.size());
        self.mask_stack.update_size(framebuffer.size());
        geng_utils::texture::update_texture_size(
            &mut self.game_texture,
            context.screen.size().map(|x| x.round() as usize),
            self.context.geng.ugli(),
        );
        if self.ui_texture.size() != framebuffer.size() {
            self.ui_texture =
                ugli::Texture::new_with(self.context.geng.ugli(), framebuffer.size(), |_| {
                    Rgba::BLACK
                });
            self.ui_depth = ugli::Renderbuffer::new(self.context.geng.ugli(), framebuffer.size());
            self.ui_texture.set_filter(ugli::Filter::Nearest);
        }

        let edit_tab = matches!(editor.tab, EditorTab::Edit);
        self.draw_game(editor, edit_tab);
        if !editor.render_options.hide_ui {
            self.draw_ui(ui, context);
        }

        let camera = &geng::PixelPerfectCamera;
        let theme = context.theme();

        if edit_tab {
            let mut masked = self.mask.start();
            masked.mask_quad(if editor.render_options.hide_ui {
                context.screen
            } else {
                ui.game.position
            });
            self.context.geng.draw2d().textured_quad(
                &mut masked.color,
                camera,
                context.screen,
                &self.game_texture,
                Color::WHITE,
            );
            self.mask.draw(draw_parameters(), framebuffer);

            if !editor.render_options.hide_ui {
                // Game border
                let width = 10.0;
                let mut border = context.geometry.quad_outline(
                    ui.game.position.extend_uniform(5.0),
                    width,
                    theme.light,
                );
                border.change_z_index(9999);
                self.util
                    .draw_geometry(&mut self.mask_stack, border, camera, framebuffer);
            }
        }

        if !editor.render_options.hide_ui {
            self.context.geng.draw2d().textured_quad(
                framebuffer,
                camera,
                Aabb2::ZERO.extend_positive(framebuffer.size().as_f32()),
                &self.ui_texture,
                Color::WHITE,
            );
        }
    }
}
