use super::{
    mask::MaskedRender,
    util::{TextRenderOptions, UtilRender},
    *,
};

use crate::ui::widget::*;

pub struct UiRender {
    geng: Geng,
    // assets: Rc<Assets>,
    util: UtilRender,
}

impl UiRender {
    pub fn new(geng: &Geng, assets: &Rc<Assets>) -> Self {
        Self {
            geng: geng.clone(),
            // assets: assets.clone(),
            util: UtilRender::new(geng, assets),
        }
    }

    pub fn draw_outline(
        &self,
        quad: Aabb2<f32>,
        width: f32,
        color: Color,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        self.util.draw_outline(
            &Collider::aabb(quad.map(r32)),
            width,
            color,
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

    pub fn draw_circle(
        &self,
        position: vec2<f32>,
        radius: f32,
        color: Rgba<f32>,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        self.geng.draw2d().draw2d(
            framebuffer,
            &geng::PixelPerfectCamera,
            &draw2d::Ellipse::circle(position, radius, color),
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
        self.draw_text_widget(&widget.text, framebuffer);
    }

    pub fn draw_text_widget(&self, widget: &TextWidget, framebuffer: &mut ugli::Framebuffer) {
        self.draw_text_widget_colored(widget, widget.options.color, framebuffer)
    }

    pub fn draw_text_widget_colored(
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

    pub fn draw_slider_widget(&self, slider: &SliderWidget, framebuffer: &mut ugli::Framebuffer) {
        self.draw_text_widget(&slider.text, framebuffer);
        self.draw_text_widget(&slider.value, framebuffer);

        if slider.bar.visible {
            self.geng.draw2d().quad(
                framebuffer,
                &geng::PixelPerfectCamera,
                slider.bar.position,
                slider.options.color,
            );
        }

        if slider.head.visible {
            let options = &slider.options;
            let color = if slider.bar_box.pressed {
                options.press_color
            } else if slider.bar_box.hovered {
                options.hover_color
            } else {
                options.color
            };
            self.geng.draw2d().quad(
                framebuffer,
                &geng::PixelPerfectCamera,
                slider.head.position,
                color,
            );
        }
    }

    pub fn draw_button_widget(&self, widget: &ButtonWidget, framebuffer: &mut ugli::Framebuffer) {
        let state = &widget.text.state;
        if !state.visible {
            return;
        }

        let options = &widget.text.options;
        let color = if state.pressed {
            options.press_color
        } else if state.hovered {
            options.hover_color
        } else {
            options.color
        };
        if let Some(texture) = &widget.texture {
            let target = geng_utils::layout::fit_aabb(
                texture.size().as_f32(),
                widget.text.state.position,
                vec2(0.5, 0.5),
            );
            self.geng.draw2d().textured_quad(
                framebuffer,
                &geng::PixelPerfectCamera,
                target,
                texture,
                color,
            );
        } else {
            self.util.draw_outline(
                &Collider::aabb(widget.text.state.position.map(r32)),
                options.size * 0.2,
                color,
                &geng::PixelPerfectCamera,
                framebuffer,
            );
        }
        self.draw_text_widget(&widget.text, framebuffer);
    }

    pub fn draw_leaderboard(
        &self,
        leaderboard: &LeaderboardWidget,
        theme: &Theme,
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
        self.draw_close_button(&leaderboard.close, theme, framebuffer);
        self.draw_text_widget(&leaderboard.title, framebuffer);
        self.draw_text_widget(&leaderboard.subtitle, framebuffer);
        self.draw_text_widget(&leaderboard.status, framebuffer);

        let mut buffer = masked.start();

        buffer.mask_quad(leaderboard.rows_state.position);

        for row in &leaderboard.rows {
            self.draw_text_widget(&row.rank, &mut buffer.color);
            self.draw_text_widget(&row.player, &mut buffer.color);
            self.draw_text_widget(&row.score, &mut buffer.color);
        }

        masked.draw(draw_parameters(), framebuffer);

        self.draw_quad(leaderboard.separator.position, theme.light, framebuffer);

        self.draw_text_widget(&leaderboard.highscore.rank, framebuffer);
        self.draw_text_widget(&leaderboard.highscore.player, framebuffer);
        self.draw_text_widget(&leaderboard.highscore.score, framebuffer);

        self.draw_outline(
            leaderboard.state.position,
            font_size * 0.2,
            theme.light,
            framebuffer,
        );
    }

    pub fn draw_close_button(
        &self,
        button: &ButtonWidget,
        theme: &Theme,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        let state = &button.text.state;
        if !state.visible {
            return;
        }

        let (bg_color, fg_color) = (theme.light, theme.danger);

        if state.hovered {
            self.draw_circle(
                button.text.state.position.center(),
                button.text.state.position.width() / 2.0 * 1.5,
                bg_color,
                framebuffer,
            );
        }

        if let Some(texture) = &button.texture {
            let target = geng_utils::layout::fit_aabb(
                texture.size().as_f32(),
                button.text.state.position,
                vec2(0.5, 0.5),
            );
            self.geng.draw2d().textured_quad(
                framebuffer,
                &geng::PixelPerfectCamera,
                target,
                texture,
                fg_color,
            );
        }
        self.draw_text_widget(&button.text, framebuffer);
    }

    pub fn draw_toggle_button(
        &self,
        text: &TextWidget,
        selected: bool,
        can_deselect: bool,
        theme: &Theme,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        let state = &text.state;
        if !state.visible {
            return;
        }

        let options = text.options;
        let (bg_color, fg_color) = if selected {
            (theme.light, theme.dark)
        } else {
            (theme.dark, theme.light)
        };

        let width = options.size * 0.2;
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
            geng_utils::layout::aabb_pos(state.position, options.align),
            options.color(fg_color),
            &geng::PixelPerfectCamera,
            framebuffer,
        );
    }
}