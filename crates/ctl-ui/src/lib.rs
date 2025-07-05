mod context;
pub mod geometry;
pub mod layout;
mod state;
pub mod widget;
mod window;

pub use context::*;
pub use state::*;
pub use window::*;

pub fn update_text_options(options: &mut ctl_render_core::TextRenderOptions, context: &UiContext) {
    options.size = context.font_size;
    options.color = context.theme().light;
    options.hover_color = options.color.map_rgb(|x| x * 0.7);
    options.press_color = options.color.map_rgb(|x| x * 0.5);
}
