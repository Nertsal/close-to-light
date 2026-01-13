use super::{mask::MaskedRender, util::UtilRender, *};

use crate::ui::{layout::AreaOps, widget::*};
use ctl_render_core::{SubTexture, get_pixel_scale};

pub struct UiRender {
    context: Context,
    pub util: UtilRender,
}

impl UiRender {
    pub fn new(context: Context) -> Self {
        Self {
            util: UtilRender::new(context.clone()),
            context,
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

        // Check orientation of the head
        // TODO: more precise
        let head_delta = main.center() - head.as_ref().map_or(main.center(), |head| head.center());
        let head_dir = if head.is_none() {
            vec2::ZERO
        } else if head_delta.x.abs() > head_delta.y.abs() {
            vec2(1.0, 0.0)
        } else {
            vec2(0.0, 1.0)
        };

        // Fill
        if let Some(head) = head {
            self.draw_quad(head, theme.dark, framebuffer);
            mask.mask_quad(head);
        }
        mask.mask_quad(main.extend_uniform(-outline_width / 2.0));
        self.draw_quad(main.extend_uniform(-outline_width), theme.dark, framebuffer);

        inner(&mut mask.color);
        masked.draw(draw_parameters(), framebuffer);

        // Outline
        if let Some(head) = head {
            let mut low = 0.0;
            let mut high = outline_width;
            if vec2::dot(head_dir, head_delta) < 0.0 {
                std::mem::swap(&mut low, &mut high);
            }
            self.draw_outline(
                head.extend_left(head_dir.x * low)
                    .extend_right(head_dir.x * high)
                    .extend_down(head_dir.y * low)
                    .extend_up(head_dir.y * high),
                outline_width,
                theme.light,
                framebuffer,
            );
        }
        self.draw_outline(main, outline_width, theme.light, framebuffer);
        if let Some(head) = head {
            let mut low = -outline_width;
            let mut high = 2.0 * outline_width;
            if vec2::dot(head_dir, head_delta) < 0.0 {
                std::mem::swap(&mut low, &mut high);
            }
            self.draw_quad(
                head.extend_uniform(-outline_width)
                    .extend_left(head_dir.x * low)
                    .extend_right(head_dir.x * high)
                    .extend_down(head_dir.y * low)
                    .extend_up(head_dir.y * high),
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
        pixel_scale: f32,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        let pos = geng_utils::pixel::pixel_perfect_aabb(
            quad.center(),
            vec2(0.5, 0.5),
            (texture.size().as_f32() * pixel_scale * get_pixel_scale(framebuffer.size()))
                .map(|x| x as usize),
            &geng::PixelPerfectCamera,
            framebuffer.size().as_f32(),
        );
        self.context.geng.draw2d().textured_quad(
            framebuffer,
            &geng::PixelPerfectCamera,
            pos,
            texture,
            color,
        );
    }

    pub fn draw_subtexture(
        &self,
        quad: Aabb2<f32>,
        texture: &SubTexture,
        color: Color,
        pixel_scale: f32,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        let size = texture.size().as_f32() * pixel_scale * get_pixel_scale(framebuffer.size());
        let pos = crate::ui::layout::align_aabb(size, quad, vec2(0.5, 0.5));
        self.context.geng.draw2d().draw2d(
            framebuffer,
            &geng::PixelPerfectCamera,
            &draw2d::TexturedQuad::colored(pos, &*texture.texture, color).sub_texture(texture.uv),
        );
    }

    pub fn draw_outline(
        &self,
        quad: Aabb2<f32>,
        width: f32,
        color: Color,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        let scale = get_pixel_scale(framebuffer.size());
        let (texture, real_width) = if width < 2.0 * scale {
            (&self.context.assets.sprites.border_thinner, 1.0 * scale)
        } else if width < 16.0 * scale {
            (&self.context.assets.sprites.border_thin, 2.0 * scale)
        } else {
            (&self.context.assets.sprites.border, 4.0 * scale)
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

    pub fn fill_quad_width(
        &self,
        position: Aabb2<f32>,
        width: f32,
        color: Color,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        let scale = get_pixel_scale(framebuffer.size());
        let (texture, real_width) = if width < 2.0 * scale {
            (&self.context.assets.sprites.fill_thinner, 1.0 * scale)
        } else if width < 16.0 * scale {
            (&self.context.assets.sprites.fill_thin, 2.0 * scale)
        } else {
            (&self.context.assets.sprites.fill, 4.0 * scale)
        };
        self.util.draw_nine_slice(
            position.extend_uniform(real_width - width),
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
        self.context.geng.draw2d().draw2d(
            framebuffer,
            &geng::PixelPerfectCamera,
            &draw2d::Quad::new(quad, color),
        );
    }

    pub fn draw_button(
        &self,
        button: &ButtonWidget,
        theme: Theme,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        let state = &button.text.state;
        if !state.visible {
            return;
        }

        let width = button.text.options.size * 0.2;

        let position = state.position;
        let bg_color = theme.get_color(button.bg_color);

        if state.mouse_left.pressed.is_some() {
            self.fill_quad(position.extend_uniform(-width), bg_color, framebuffer);
        } else if state.hovered {
            self.fill_quad(position.extend_uniform(-width * 0.5), bg_color, framebuffer)
        } else {
            self.fill_quad(position, bg_color, framebuffer)
        }
        self.draw_text(&button.text, framebuffer);
    }

    pub fn draw_icon_button(
        &self,
        icon: &IconButtonWidget,
        theme: Theme,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        if !icon.icon.state.visible {
            return;
        }
        self.draw_icon(&icon.icon, theme, framebuffer);
    }

    pub fn draw_icon(&self, icon: &IconWidget, theme: Theme, framebuffer: &mut ugli::Framebuffer) {
        if !icon.state.visible {
            return;
        }

        if let Some(bg) = &icon.background {
            match bg.kind {
                IconBackgroundKind::NineSlice => {
                    let texture = //if width < 5.0 {
                        &self.context.assets.sprites.fill_thin;
                    // } else {
                    //     &self.assets.sprites.fill
                    // };
                    self.util.draw_nine_slice(
                        icon.state.position,
                        theme.get_color(bg.color),
                        texture,
                        icon.pixel_scale * get_pixel_scale(framebuffer.size()),
                        &geng::PixelPerfectCamera,
                        framebuffer,
                    );
                }
                IconBackgroundKind::Circle => {
                    self.draw_texture(
                        icon.state.position,
                        &self.context.assets.sprites.circle,
                        theme.get_color(bg.color),
                        icon.pixel_scale,
                        framebuffer,
                    );
                }
            }
        }
        self.draw_subtexture(
            icon.state.position,
            &icon.texture,
            theme.get_color(icon.color),
            icon.pixel_scale,
            framebuffer,
        );
    }

    // TODO: as text render option
    pub fn draw_text_wrapped(&self, widget: &TextWidget, framebuffer: &mut ugli::Framebuffer) {
        if !widget.state.visible {
            return;
        }

        let main = widget.state.position;
        let lines = crate::util::wrap_text(
            &self.context.assets.fonts.pixel,
            &widget.text,
            main.width() / widget.options.size,
        );
        let row = main.align_aabb(vec2(main.width(), widget.options.size), vec2(0.5, 1.0));
        let rows = row.stack(vec2(0.0, -row.height()), lines.len());

        for (line, position) in lines.into_iter().zip(rows) {
            self.util.draw_text(
                line,
                position.align_pos(widget.options.align),
                widget.options,
                &geng::PixelPerfectCamera,
                framebuffer,
            );
        }
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

        // Fit to area
        let mut widget = widget.clone();

        let font = &self.context.assets.fonts.pixel;
        let measure = font.measure(&widget.text, 1.0);

        let size = widget.state.position.size();
        let right = vec2(size.x, 0.0).rotate(widget.options.rotation).x;
        let left = vec2(0.0, size.y).rotate(widget.options.rotation).x;
        let width = if left.signum() != right.signum() {
            left.abs() + right.abs()
        } else {
            left.abs().max(right.abs())
        };

        let max_height = size.y * 0.9;
        let max_width = width * 0.85; // Leave some space TODO: move into a parameter or smth
        let max_size = (max_width / measure.width()).min(max_height / measure.height());
        let size = widget.options.size.min(max_size).min(max_height);

        widget.options.size = size;

        self.util.draw_text(
            &widget.text,
            geng_utils::layout::aabb_pos(widget.state.position, widget.options.align),
            widget.options.color(color),
            &geng::PixelPerfectCamera,
            framebuffer,
        );
        // self.draw_quad(
        //     widget
        //         .state
        //         .position
        //         .align_aabb(measure.size() * size, widget.options.align),
        //     Rgba::new(0.7, 0.5, 0.5, 0.75),
        //     framebuffer,
        // );
    }

    pub fn draw_slider(
        &self,
        slider: &SliderWidget,
        mut theme: Theme,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        if !slider.state.visible {
            return;
        }

        if slider.state.hovered {
            std::mem::swap(&mut theme.dark, &mut theme.light);
            self.fill_quad(slider.state.position, theme.dark, framebuffer);
        }

        self.draw_text_colored(&slider.text, theme.light, framebuffer);
        self.draw_text_colored(&slider.value, theme.light, framebuffer);

        if slider.bar.visible {
            self.context.geng.draw2d().quad(
                framebuffer,
                &geng::PixelPerfectCamera,
                slider.bar.position,
                theme.light,
            );
        }

        if slider.head.visible {
            let color = theme.light;
            self.context.geng.draw2d().quad(
                framebuffer,
                &geng::PixelPerfectCamera,
                slider.head.position,
                color,
            );
        }
    }

    pub fn draw_notification(
        &self,
        notification: &NotificationWidget,
        outline_width: f32,
        theme: Theme,
        masked: &mut MaskedRender,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        let window = notification.state.position;
        self.draw_window(
            masked,
            window,
            None,
            outline_width,
            theme,
            framebuffer,
            |framebuffer| {
                self.draw_text_wrapped(&notification.text, framebuffer);
                self.draw_icon(&notification.confirm.icon, theme, framebuffer);
            },
        );
    }

    pub fn draw_confirm(
        &self,
        confirm: &ConfirmWidget,
        outline_width: f32,
        theme: Theme,
        masked: &mut MaskedRender,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        let t = crate::util::smoothstep(confirm.window.show.time.get_ratio());

        let window = confirm.state.position;
        let min_height = outline_width * 10.0;
        let height = (t * window.height()).max(min_height);

        let window = window.with_height(height, 1.0);
        self.draw_window(
            masked,
            window,
            None,
            outline_width,
            theme,
            framebuffer,
            |framebuffer| {
                let title = confirm.title.state.position;
                self.fill_quad(title, theme.light, framebuffer);
                self.draw_text_colored(&confirm.title, theme.dark, framebuffer);
                self.draw_text(&confirm.message, framebuffer);
                self.draw_icon(&confirm.confirm_icon.icon, theme, framebuffer);
                self.draw_icon(&confirm.discard_icon.icon, theme, framebuffer);
                self.draw_button(&confirm.confirm_text, theme, framebuffer);
                self.draw_button(&confirm.discard_text, theme, framebuffer);
            },
        );
    }

    pub fn draw_score(
        &self,
        score: &ScoreWidget,
        theme: Theme,
        outline_width: f32,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        let camera = &geng::PixelPerfectCamera;

        self.context.geng.draw2d().draw2d(
            framebuffer,
            camera,
            &draw2d::Quad::new(score.state.position, theme.dark),
        );

        self.draw_text(&score.music_name, framebuffer);
        self.draw_text(&score.difficulty_name, framebuffer);
        for icon in &score.modifiers {
            self.draw_icon(icon, theme, framebuffer);
        }

        self.draw_text(&score.score_text, framebuffer);
        self.draw_subtexture(
            score.grade_back.position,
            &self.context.assets.atlas.grade_back_score(),
            theme.highlight,
            score.grade.pixel_scale,
            framebuffer,
        );
        self.draw_text(&score.score_value, framebuffer);
        self.draw_text(&score.grade_text, framebuffer);
        self.draw_subtexture(
            score.grade_back.position,
            &self.context.assets.atlas.grade_back(),
            theme.light,
            score.grade.pixel_scale,
            framebuffer,
        );
        self.draw_icon(&score.grade, theme, framebuffer);

        let mut draw_bar = |position: Aabb2<f32>,
                            light_color: Color,
                            total: usize,
                            highlight: usize,
                            light: usize,
                            danger: usize,
                            dark: usize| {
            let outline_pos = position.extend_uniform(outline_width);

            let total = total.max(1) as f32;
            let highlight = highlight as f32 / total * position.height();
            let light = light as f32 / total * position.height();
            let danger = danger as f32 / total * position.height();
            let dark = dark as f32 / total * position.height();
            self.draw_quad(position.with_height(light, 0.0), light_color, framebuffer);
            self.draw_quad(
                position.with_height(highlight, 0.0),
                theme.highlight,
                framebuffer,
            );
            self.draw_quad(
                position.extend_down(-light).extend_up(-dark),
                theme.danger,
                framebuffer,
            );
            self.draw_quad(
                position.extend_down(-light - danger),
                theme.dark,
                framebuffer,
            );
            self.draw_outline(outline_pos, outline_width, theme.light, framebuffer);
        };

        let metrics = &score.saved_score.score.metrics;
        draw_bar(
            score.accuracy_bar.position,
            theme.highlight,
            metrics.discrete.total,
            0,
            metrics.discrete.perfect,
            metrics.discrete.total - metrics.discrete.perfect,
            0,
        );
        draw_bar(
            score.precision_bar.position,
            theme.light,
            metrics.dynamic.frames,
            metrics.dynamic.frames_perfect,
            metrics.dynamic.frames_light,
            metrics.dynamic.frames_red,
            metrics.dynamic.frames_black,
        );

        self.draw_text(&score.accuracy_value, framebuffer);
        self.draw_text(&score.accuracy_text, framebuffer);
        self.draw_text(&score.precision_value, framebuffer);
        self.draw_text(&score.precision_text, framebuffer);

        self.draw_outline(
            score.state.position.extend_uniform(outline_width),
            outline_width,
            theme.light,
            framebuffer,
        );
    }

    pub fn draw_leaderboard(
        &self,
        leaderboard: &LeaderboardWidget,
        theme: Theme,
        outline_width: f32,
        masked: &mut MaskedRender,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        let camera = &geng::PixelPerfectCamera;

        self.context.geng.draw2d().draw2d(
            framebuffer,
            camera,
            &draw2d::Quad::new(leaderboard.state.position, theme.dark),
        );
        // self.draw_icon(&leaderboard.close.icon, framebuffer);
        if leaderboard.reload.icon.state.visible {
            self.draw_icon(&leaderboard.reload.icon, theme, framebuffer);
        }
        self.draw_text(&leaderboard.title, framebuffer);
        self.draw_text(&leaderboard.subtitle, framebuffer);
        self.draw_text(&leaderboard.status, framebuffer);

        // self.draw_quad(
        //     leaderboard.separator_title.position,
        //     theme.light,
        //     framebuffer,
        // );

        let mut buffer = masked.start();

        buffer.mask_quad(leaderboard.rows_state.position);

        for row in &leaderboard.rows {
            self.draw_text(&row.rank, &mut buffer.color);
            self.draw_text(&row.player, &mut buffer.color);
            self.draw_text(&row.score, &mut buffer.color);
            self.draw_text(&row.accuracy, &mut buffer.color);
            self.draw_icon(&row.grade, theme, &mut buffer.color);
            self.draw_outline(
                row.state.position,
                outline_width,
                theme.light,
                &mut buffer.color,
            );
            for icon in &row.modifiers {
                self.draw_icon(icon, theme, &mut buffer.color);
            }
        }

        masked.draw(draw_parameters(), framebuffer);

        self.draw_quad(
            leaderboard.separator_highscore.position,
            theme.light,
            framebuffer,
        );

        if leaderboard.highscore.state.visible {
            self.draw_text(&leaderboard.highscore.rank, framebuffer);
            self.draw_text(&leaderboard.highscore.player, framebuffer);
            self.draw_text(&leaderboard.highscore.score, framebuffer);
            self.draw_icon(&leaderboard.highscore.grade, theme, framebuffer);
            self.draw_text(&leaderboard.highscore.accuracy, framebuffer);
            for icon in &leaderboard.highscore.modifiers {
                self.draw_icon(icon, theme, framebuffer);
            }
        }
    }

    pub fn draw_toggle_widget(
        &self,
        toggle: &ToggleWidget,
        theme: Theme,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        let width = toggle.text.options.size * 0.1;

        let mut fg_color = theme.light;
        if toggle.state.hovered {
            fg_color = theme.get_color(toggle.checked_color);
        }
        if toggle.checked {
            self.fill_quad_width(
                toggle.tick.position,
                width,
                theme.get_color(toggle.checked_color),
                framebuffer,
            );
        }
        self.draw_outline(toggle.tick.position, width, fg_color, framebuffer);
        self.draw_text_colored(&toggle.text, fg_color, framebuffer);
    }

    pub fn draw_toggle_button(
        &self,
        toggle: &ToggleButtonWidget,
        theme: Theme,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        let width = toggle.text.options.size * 0.2;
        let (bg_color, fg_color) = if toggle.selected {
            (theme.light, theme.dark)
        } else {
            (theme.dark, theme.light)
        };

        let outline = toggle.text.state.position;
        self.fill_quad_width(outline, width, bg_color, framebuffer);
        self.draw_text_colored(&toggle.text, fg_color, framebuffer);
        if let Some(icon) = &toggle.icon {
            self.draw_icon(icon, theme, framebuffer);
        }
        self.draw_outline(outline, width, theme.light, framebuffer);
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
        }

        self.draw_outline(state.position, width, theme.light, framebuffer);
    }

    pub fn fill_quad(
        &self,
        position: Aabb2<f32>,
        color: Color,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        let size = position.size();
        let size = size.x.min(size.y);

        let scale = ui::get_pixel_scale(framebuffer.size());

        let texture = if size < 48.0 * scale {
            &self.context.assets.sprites.fill_thin
        } else {
            &self.context.assets.sprites.fill
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

    pub fn draw_profile(&self, ui: &ProfileWidget, framebuffer: &mut ugli::Framebuffer) {
        let theme = self.context.get_options().theme;
        self.draw_text(&ui.offline, framebuffer);

        let register = &ui.register;
        if register.state.visible {
            // self.draw_input(&register.username, framebuffer);
            // self.draw_input(&register.password, framebuffer);
            // self.draw_button(&register.login, framebuffer);
            // self.draw_button(&register.register, framebuffer);
            self.draw_text(&register.login_with, framebuffer);
            self.draw_icon(&register.discord.icon, theme, framebuffer);
            #[cfg(feature = "steam")]
            self.draw_icon(&register.steam.icon, theme, framebuffer);
        }

        let logged = &ui.logged;
        if logged.state.visible {
            self.draw_text(&logged.username, framebuffer);
            self.draw_button(&logged.logout, theme, framebuffer);
        }

        let connecting = &ui.connecting;
        if connecting.state.visible {
            self.draw_text(connecting, framebuffer);
        }
    }
}
