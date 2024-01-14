mod game;
mod ui;

use super::{
    dither::DitherRender,
    ui::UiRender,
    util::{TextRenderOptions, UtilRender},
    *,
};

use crate::editor::{State, *};

use geng::prelude::ugli::{BlendEquation, BlendFactor, BlendMode, ChannelBlendMode};

pub struct EditorRender {
    geng: Geng,
    assets: Rc<Assets>,
    dither: DitherRender,
    util: UtilRender,
    ui: UiRender,
    unit_quad: ugli::VertexBuffer<draw2d::TexturedVertex>,
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
            assets: assets.clone(),
            dither: DitherRender::new(geng, assets),
            util: UtilRender::new(geng, assets),
            ui: UiRender::new(geng, assets),
            unit_quad: geng_utils::geometry::unit_quad_geometry(geng.ugli()),
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
        geng_utils::texture::update_texture_size(
            &mut self.game_texture,
            ui.game.position.size().map(|x| x.round() as usize),
            self.geng.ugli(),
        );
        geng_utils::texture::update_texture_size(
            &mut self.ui_texture,
            framebuffer.size(),
            self.geng.ugli(),
        );

        self.draw_game(editor);
        self.draw_ui(editor, ui);

        let camera = &geng::PixelPerfectCamera;
        self.geng.draw2d().textured_quad(
            framebuffer,
            camera,
            ui.game.position,
            &self.game_texture,
            Color::WHITE,
        );
        self.geng.draw2d().textured_quad(
            framebuffer,
            camera,
            Aabb2::ZERO.extend_positive(framebuffer.size().as_f32()),
            &self.ui_texture,
            Color::WHITE,
        );
    }
}
