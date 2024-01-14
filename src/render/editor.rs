mod game;
mod ui;

use super::{
    dither::DitherRender,
    mask::MaskedRender,
    ui::UiRender,
    util::{TextRenderOptions, UtilRender},
    *,
};

use crate::editor::{State, *};

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
}

pub struct RenderOptions {
    pub hide_ui: bool,
    pub show_grid: bool,
}

impl EditorRender {
    pub fn new(geng: &Geng, assets: &Rc<Assets>) -> Self {
        let mut game_texture = geng_utils::texture::new_texture(geng.ugli(), vec2(1, 1));
        game_texture.set_filter(ugli::Filter::Nearest);
        let mut ui_texture = geng_utils::texture::new_texture(geng.ugli(), vec2(1, 1));
        ui_texture.set_filter(ugli::Filter::Nearest);

        Self {
            geng: geng.clone(),
            // assets: assets.clone(),
            dither: DitherRender::new(geng, assets),
            util: UtilRender::new(geng, assets),
            ui: UiRender::new(geng, assets),
            mask: MaskedRender::new(geng, assets, vec2(1, 1)),
            // unit_quad: geng_utils::geometry::unit_quad_geometry(geng.ugli()),
            game_texture,
            ui_texture,
        }
    }

    pub fn draw_editor(
        &mut self,
        editor: &Editor,
        ui: &EditorUI,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        let size = ui.screen.position.size().map(|x| x.round() as usize);
        self.mask.update_size(size);
        geng_utils::texture::update_texture_size(&mut self.game_texture, size, self.geng.ugli());
        geng_utils::texture::update_texture_size(
            &mut self.ui_texture,
            framebuffer.size(),
            self.geng.ugli(),
        );

        self.draw_game(editor);
        self.draw_ui(editor, ui);

        let camera = &geng::PixelPerfectCamera;

        let mut masked = self.mask.start();
        masked.mask_quad(ui.game.position);
        self.geng.draw2d().textured_quad(
            &mut masked.color,
            camera,
            ui.screen.position,
            &self.game_texture,
            Color::WHITE,
        );
        self.mask.draw(draw_parameters(), framebuffer);

        self.geng.draw2d().textured_quad(
            framebuffer,
            camera,
            Aabb2::ZERO.extend_positive(framebuffer.size().as_f32()),
            &self.ui_texture,
            Color::WHITE,
        );
    }
}
