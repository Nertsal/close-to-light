use super::{
    mask::MaskedRender,
    util::{TextRenderOptions, UtilRender},
    *,
};

use crate::ui::widget::*;

pub fn pixel_scale(framebuffer: &ugli::Framebuffer) -> f32 {
    framebuffer.size().y as f32 / 360.0
}

pub struct UiRender {
    geng: Geng,
    pub assets: Rc<Assets>,
    pub util: UtilRender,
}

impl UiRender {
    pub fn new(geng: &Geng, assets: &Rc<Assets>) -> Self {
        Self {
            geng: geng.clone(),
            assets: assets.clone(),
            util: UtilRender::new(geng, assets),
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn draw_window(
        &self,
        masked: &mut MaskedRender,
        main: Aabb2<f32>,
        head: Option<Aabb2<f32>>,
        outline_width: f32,
        theme: Theme,
        framebuffer: &mut ugli::Framebuffer,
        inner: impl FnOnce(&mut ugli::Framebuffer),
    ) {
        // let size = main.size().map(|x| x.abs().ceil() as usize);
        // let mut texture = ugli::Texture::new_with(self.geng.ugli(), size, |_| theme.dark);

        let mut mask = masked.start();

        // Fill
        if let Some(head) = head {
            self.draw_quad(head, theme.dark, framebuffer);
            mask.mask_quad(head);
        }
        mask.mask_quad(main);
        self.draw_quad(main.extend_uniform(-outline_width), theme.dark, framebuffer);

        inner(&mut mask.color);
        masked.draw(draw_parameters(), framebuffer);

        // Outline
        if let Some(head) = head {
            let delta = main.center() - head.center(); // TODO: more precise
            let dir = if delta.x.abs() > delta.y.abs() {
                vec2(1.0, 0.0)
            } else {
                vec2(0.0, 1.0)
            };
            let mut low = 0.0;
            let mut high = outline_width;
            if vec2::dot(dir, delta) < 0.0 {
                std::mem::swap(&mut low, &mut high);
            }
            self.draw_outline(
                head.extend_left(dir.x * low)
                    .extend_right(dir.x * high)
                    .extend_down(dir.y * low)
                    .extend_up(dir.y * high),
                outline_width,
                theme.light,
                framebuffer,
            );
        }
        self.draw_outline(main, outline_width, theme.light, framebuffer);
        if let Some(head) = head {
            let delta = main.center() - head.center(); // TODO: more precise
            let dir = if delta.x.abs() > delta.y.abs() {
                vec2(1.0, 0.0)
            } else {
                vec2(0.0, 1.0)
            };
            let mut low = -1.0 * outline_width;
            let mut high = 3.0 * outline_width;
            if vec2::dot(dir, delta) < 0.0 {
                std::mem::swap(&mut low, &mut high);
            }
            self.draw_quad(
                head.extend_uniform(-outline_width)
                    .extend_left(dir.x * low)
                    .extend_right(dir.x * high)
                    .extend_down(dir.y * low)
                    .extend_up(dir.y * high),
                theme.dark,
                framebuffer,
            );
        }
    }

    pub fn draw_texture(
        &self,
        quad: Aabb2<f32>,
        texture: &ugli::Texture,
        color: Color,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        let size = texture.size().as_f32() * pixel_scale(framebuffer);
        let pos = crate::ui::layout::align_aabb(size, quad, vec2(0.5, 0.5));
        self.geng.draw2d().textured_quad(
            framebuffer,
            &geng::PixelPerfectCamera,
            pos,
            texture,
            color,
        );
    }

    pub fn draw_outline(
        &self,
        quad: Aabb2<f32>,
        width: f32,
        color: Color,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        let scale = pixel_scale(framebuffer);
        let (texture, real_width) = if width < 2.0 * scale {
            (&self.assets.sprites.border_thinner, 1.0 * scale)
        } else if width < 4.0 * scale {
            (&self.assets.sprites.border_thin, 2.0 * scale)
        } else {
            (&self.assets.sprites.border, 4.0 * scale)
        };
        self.util.draw_nine_slice(
            quad.extend_uniform(real_width - width),
            color,
            texture,
            scale,
            &geng::PixelPerfectCamera,
            framebuffer,
        );
    }

    pub fn draw_quad(
        &self,
        quad: Aabb2<f32>,
        color: Rgba<f32>,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        self.geng.draw2d().draw2d(
            framebuffer,
            &geng::PixelPerfectCamera,
            &draw2d::Quad::new(quad, color),
        );
    }

    pub fn draw_icon(&self, icon: &IconWidget, framebuffer: &mut ugli::Framebuffer) {
        self.draw_icon_colored(icon, Color::WHITE, framebuffer);
    }

    pub fn draw_icon_colored(
        &self,
        icon: &IconWidget,
        color: Rgba<f32>,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        if !icon.state.visible {
            return;
        }

        if let Some(bg) = &icon.background {
            match bg.kind {
                IconBackgroundKind::NineSlice => {
                    let texture = //if width < 5.0 {
                        &self.assets.sprites.fill_thin;
                    // } else {
                    //     &self.assets.sprites.fill
                    // };
                    self.util.draw_nine_slice(
                        icon.state.position,
                        bg.color * color,
                        texture,
                        pixel_scale(framebuffer),
                        &geng::PixelPerfectCamera,
                        framebuffer,
                    );
                }
                IconBackgroundKind::Circle => {
                    self.draw_texture(
                        icon.state.position,
                        &self.assets.sprites.circle,
                        bg.color * color,
                        framebuffer,
                    );
                }
            }
        }
        self.draw_texture(
            icon.state.position,
            &icon.texture,
            icon.color * color,
            framebuffer,
        );
    }

    pub fn draw_checkbox(
        &self,
        widget: &CheckboxWidget,
        options: TextRenderOptions,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        if !widget.check.visible {
            return;
        }

        let camera = &geng::PixelPerfectCamera;
        let options = options.align(vec2(0.0, 0.5)); // TODO

        let checkbox = widget.check.position;
        if widget.checked {
            let checkbox = checkbox.extend_uniform(-options.size * 0.05);
            for (a, b) in [
                (checkbox.bottom_left(), checkbox.top_right()),
                (checkbox.top_left(), checkbox.bottom_right()),
            ] {
                self.geng.draw2d().draw2d(
                    framebuffer,
                    camera,
                    &draw2d::Segment::new(Segment(a, b), options.size * 0.07, options.color),
                );
            }
        }
        self.util.draw_outline(
            &Collider::aabb(checkbox.map(r32)),
            options.size * 0.1,
            options.color,
            camera,
            framebuffer,
        );
        self.draw_text(&widget.text, framebuffer);
    }

    pub fn draw_text(&self, widget: &TextWidget, framebuffer: &mut ugli::Framebuffer) {
        self.draw_text_colored(widget, widget.options.color, framebuffer)
    }

    pub fn draw_text_colored(
        &self,
        widget: &TextWidget,
        color: Color,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        if !widget.state.visible {
            return;
        }

        self.util.draw_text(
            &widget.text,
            geng_utils::layout::aabb_pos(widget.state.position, widget.options.align),
            widget.options.color(color),
            &geng::PixelPerfectCamera,
            framebuffer,
        );
    }

    pub fn draw_slider(
        &self,
        slider: &SliderWidget,
        mut theme: Theme,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        if slider.state.hovered {
            std::mem::swap(&mut theme.dark, &mut theme.light);
            self.fill_quad(slider.state.position, theme.dark, framebuffer);
        }

        self.draw_text_colored(&slider.text, theme.light, framebuffer);
        self.draw_text_colored(&slider.value, theme.light, framebuffer);

        if slider.bar.visible {
            self.geng.draw2d().quad(
                framebuffer,
                &geng::PixelPerfectCamera,
                slider.bar.position,
                theme.light,
            );
        }

        if slider.head.visible {
            let color = theme.light;
            self.geng.draw2d().quad(
                framebuffer,
                &geng::PixelPerfectCamera,
                slider.head.position,
                color,
            );
        }
    }

    pub fn draw_button(
        &self,
        widget: &ButtonWidget,
        theme: Theme,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        let state = &widget.text.state;
        if !state.visible {
            return;
        }

        let options = &widget.text.options;
        let position = widget.text.state.position;
        let width = options.size * 0.2;

        let mut text_color = theme.light;
        if state.pressed {
            self.fill_quad(position, theme.light, framebuffer);
            text_color = theme.dark;
        } else if state.hovered {
            self.fill_quad(position.extend_uniform(width), theme.light, framebuffer);
            text_color = theme.dark;
        } else {
            self.draw_outline(position, width, theme.light, framebuffer);
        };

        self.draw_text_colored(&widget.text, text_color, framebuffer);
    }

    pub fn draw_input(&self, widget: &InputWidget, framebuffer: &mut ugli::Framebuffer) {
        if !widget.state.visible {
            return;
        }

        self.draw_text(&widget.name, framebuffer);
        self.draw_text(&widget.text, framebuffer);
    }

    pub fn draw_leaderboard(
        &self,
        leaderboard: &LeaderboardWidget,
        theme: Theme,
        masked: &mut MaskedRender,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        let font_size = framebuffer.size().y as f32 * 0.04; // TODO: put in some context
        let camera = &geng::PixelPerfectCamera;

        self.geng.draw2d().draw2d(
            framebuffer,
            camera,
            &draw2d::Quad::new(leaderboard.state.position, theme.dark),
        );
        // self.draw_icon(&leaderboard.close.icon, framebuffer);
        self.draw_icon(&leaderboard.reload.icon, framebuffer);
        self.draw_text(&leaderboard.title, framebuffer);
        self.draw_text(&leaderboard.subtitle, framebuffer);
        self.draw_text(&leaderboard.status, framebuffer);

        let mut buffer = masked.start();

        buffer.mask_quad(leaderboard.rows_state.position);

        for row in &leaderboard.rows {
            self.draw_text(&row.rank, &mut buffer.color);
            self.draw_text(&row.player, &mut buffer.color);
            self.draw_text(&row.score, &mut buffer.color);
        }

        masked.draw(draw_parameters(), framebuffer);

        self.draw_quad(leaderboard.separator.position, theme.light, framebuffer);

        self.draw_text(&leaderboard.highscore.rank, framebuffer);
        self.draw_text(&leaderboard.highscore.player, framebuffer);
        self.draw_text(&leaderboard.highscore.score, framebuffer);

        self.draw_outline(
            leaderboard.state.position,
            font_size * 0.2,
            theme.light,
            framebuffer,
        );
    }

    pub fn draw_toggle_button(
        &self,
        text: &TextWidget,
        selected: bool,
        can_deselect: bool,
        theme: Theme,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        let state = &text.state;
        if !state.visible {
            return;
        }

        let (bg_color, fg_color) = if selected {
            (theme.light, theme.dark)
        } else {
            (theme.dark, theme.light)
        };

        let width = text.options.size * 0.2;
        let shrink = if can_deselect && state.hovered && selected {
            width
        } else {
            0.0
        };
        let pos = state.position.extend_uniform(-shrink);
        self.draw_quad(pos.extend_uniform(-width), bg_color, framebuffer);
        if state.hovered || selected {
            self.draw_outline(pos, width, theme.light, framebuffer);
        }
        self.util.draw_text(
            &text.text,
            geng_utils::layout::aabb_pos(state.position, text.options.align),
            text.options.color(fg_color),
            &geng::PixelPerfectCamera,
            framebuffer,
        );
    }

    pub fn draw_toggle_widget(
        &self,
        toggle: &ToggleWidget,
        theme: Theme,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        self.draw_toggle_button(
            &toggle.text,
            toggle.selected,
            toggle.can_deselect,
            theme,
            framebuffer,
        );
    }

    // TODO: more general name
    pub fn draw_toggle(
        &self,
        text: &TextWidget,
        width: f32,
        theme: Theme,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        self.draw_toggle_slide(
            &text.state,
            &[text],
            width,
            text.state.hovered,
            theme,
            framebuffer,
        )
    }

    // TODO: more general name
    pub fn draw_toggle_slide(
        &self,
        state: &WidgetState,
        texts: &[&TextWidget],
        width: f32,
        selected: bool,
        theme: Theme,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        if !state.visible {
            return;
        }

        let (bg_color, fg_color) = if selected {
            (theme.light, theme.dark)
        } else {
            (theme.dark, theme.light)
        };

        self.draw_quad(state.position.extend_uniform(-width), bg_color, framebuffer);

        for text in texts {
            self.draw_text_colored(text, fg_color, framebuffer);
            self.draw_text_colored(text, fg_color, framebuffer);
        }

        self.draw_outline(state.position, width, theme.light, framebuffer);
    }

    pub fn draw_value<T>(&self, value: &ValueWidget<T>, framebuffer: &mut ugli::Framebuffer) {
        self.draw_text(&value.text, framebuffer);
        self.draw_text(&value.value_text, framebuffer);
    }

    pub fn fill_quad(
        &self,
        position: Aabb2<f32>,
        color: Color,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        let size = position.size();
        let size = size.x.min(size.y);

        let scale = ui::pixel_scale(framebuffer);

        let texture = if size < 48.0 * scale {
            &self.assets.sprites.fill_thin
        } else {
            &self.assets.sprites.fill
        };
        self.util.draw_nine_slice(
            position,
            color,
            texture,
            scale,
            &geng::PixelPerfectCamera,
            framebuffer,
        );
    }
}
