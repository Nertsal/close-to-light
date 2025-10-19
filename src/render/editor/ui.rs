use crate::ui::{geometry::Geometry, widget::Widget};

use super::*;

impl EditorRender {
    pub(super) fn draw_ui(&mut self, editor_ui: &EditorUi, ui: &UiContext) {
        let framebuffer = &mut ugli::Framebuffer::new(
            self.context.geng.ugli(),
            ugli::ColorAttachment::Texture(&mut self.ui_texture),
            ugli::DepthAttachment::Renderbuffer(&mut self.ui_depth),
        );
        // let theme = editor.context.get_options().theme;
        self.font_size = framebuffer.size().y as f32 * 0.04;

        let camera = &geng::PixelPerfectCamera;
        ugli::clear(framebuffer, Some(Color::TRANSPARENT_BLACK), Some(1.0), None);

        let mut geometry = Geometry::new();
        if let Some(widget) = &editor_ui.confirm {
            geometry.merge(widget.draw(ui));
        }
        geometry.merge(editor_ui.context_menu.draw(ui));

        let geometry = RefCell::new(geometry);
        ui.state.iter_widgets(
            |w| {
                geometry.borrow_mut().merge(w.draw_top(ui));
            },
            |w| {
                geometry.borrow_mut().merge(w.draw(ui));
            },
        );
        let geometry = geometry.into_inner();

        self.util
            .draw_geometry(&mut self.mask_stack, geometry, camera, framebuffer);
    }
}
