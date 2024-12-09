mod game;
mod ui;

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
    geng: Geng,
    // assets: Rc<Assets>,
    dither: DitherRender,
    util: UtilRender,
    ui: UiRender,
    mask: MaskedRender,
    // unit_quad: ugli::VertexBuffer<draw2d::TexturedVertex>,
    game_texture: ugli::Texture,
    ui_texture: ugli::Texture,
    font_size: f32,
}

pub struct RenderOptions {
    pub hide_ui: bool,
    pub show_grid: bool,
}

impl EditorRender {
    pub fn new(context: Context) -> Self {
        let mut game_texture = geng_utils::texture::new_texture(context.geng.ugli(), vec2(1, 1));
        game_texture.set_filter(ugli::Filter::Nearest);
        let mut ui_texture = geng_utils::texture::new_texture(context.geng.ugli(), vec2(1, 1));
        ui_texture.set_filter(ugli::Filter::Nearest);

        Self {
            geng: context.geng.clone(),
            // assets: assets.clone(),
            dither: DitherRender::new(&context.geng, &context.assets),
            util: UtilRender::new(context.clone()),
            ui: UiRender::new(context.clone()),
            mask: MaskedRender::new(&context.geng, &context.assets, vec2(1, 1)),
            // unit_quad: geng_utils::geometry::unit_quad_geometry(geng.ugli()),
            game_texture,
            ui_texture,
            font_size: 1.0,
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
        geng_utils::texture::update_texture_size(
            &mut self.game_texture,
            context.screen.size().map(|x| x.round() as usize),
            self.geng.ugli(),
        );
        geng_utils::texture::update_texture_size(
            &mut self.ui_texture,
            framebuffer.size(),
            self.geng.ugli(),
        );

        let edit_tab = matches!(editor.tab, EditorTab::Edit);
        self.draw_game(editor, edit_tab);
        if !editor.render_options.hide_ui {
            self.draw_ui(editor, context);
        }

        let camera = &geng::PixelPerfectCamera;

        if edit_tab {
            let mut masked = self.mask.start();
            masked.mask_quad(if editor.render_options.hide_ui {
                context.screen
            } else {
                ui.game.position
            });
            self.geng.draw2d().textured_quad(
                &mut masked.color,
                camera,
                context.screen,
                &self.game_texture,
                Color::WHITE,
            );
            self.mask.draw(draw_parameters(), framebuffer);
        }

        if !editor.render_options.hide_ui {
            self.geng.draw2d().textured_quad(
                framebuffer,
                camera,
                Aabb2::ZERO.extend_positive(framebuffer.size().as_f32()),
                &self.ui_texture,
                Color::WHITE,
            );
        }
    }
}
