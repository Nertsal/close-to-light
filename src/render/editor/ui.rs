use crate::ui::{geometry::Geometry, widget::Widget};

use super::*;

impl EditorRender {
    pub(super) fn draw_ui(&mut self, editor_ui: &EditorUi, ui: &UiContext) {
        let framebuffer = &mut ugli::Framebuffer::new(
            self.geng.ugli(),
            ugli::ColorAttachment::Texture(&mut self.ui_texture),
            ugli::DepthAttachment::Renderbuffer(&mut self.ui_depth),
        );
        // let theme = editor.context.get_options().theme;
        self.font_size = framebuffer.size().y as f32 * 0.04;

        let camera = &geng::PixelPerfectCamera;
        ugli::clear(framebuffer, Some(Color::TRANSPARENT_BLACK), Some(1.0), None);

        let mut geometry = Geometry::new();
        geometry.merge(editor_ui.context_menu.draw(ui));

        ui.state.iter_widgets(|w| {
            geometry.merge(w.draw(ui));
        });

        self.util
            .draw_geometry(&mut self.mask_stack, geometry, camera, framebuffer);
    }
}
