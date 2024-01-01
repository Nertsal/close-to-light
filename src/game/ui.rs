use super::*;

pub use crate::ui::{layout, widget::*};

pub struct GameUI {
    pub leaderboard: LeaderboardWidget,
}

impl GameUI {
    pub fn new(assets: &Rc<Assets>) -> Self {
        Self {
            leaderboard: LeaderboardWidget::new(assets),
        }
    }

    pub fn layout(
        &mut self,
        model: &mut Model,
        screen: Aabb2<f32>,
        cursor: CursorContext,
        delta_time: f32,
        _geng: &Geng,
    ) -> bool {
        // Fix aspect
        let screen = layout::fit_aabb(vec2(16.0, 9.0), screen, vec2::splat(0.5));

        let layout_size = screen.height() * 0.03;

        let mut context = UiContext {
            theme: model.options.theme,
            layout_size,
            font_size: screen.height() * 0.04,
            can_focus: true,
            cursor,
            delta_time,
        };
        macro_rules! update {
            ($widget:expr, $position:expr) => {{
                $widget.update($position, &context);
            }};
        }

        // Margin
        let main = screen.extend_uniform(-layout_size * 2.0);

        // Logo
        let (_ctl_logo, main) = layout::cut_top_down(main, layout_size * 4.0);
        // update!(self.ctl_logo, ctl_logo);
        let main = main.extend_up(-layout_size * 3.0);

        if let State::Lost { .. } | State::Finished = model.state {
            // Leaderboard
            self.leaderboard.show();
            self.leaderboard.close.hide();

            let width = layout_size * 20.0;
            let height = main.height() + layout_size * 2.0;

            let leaderboard = Aabb2::point(main.bottom_right() + vec2(0.0, 2.0) * layout_size)
                .extend_left(width)
                .extend_down(height);

            let offset = main.height();

            let leaderboard = leaderboard.translate(vec2(0.0, offset));

            self.leaderboard
                .update_state(&model.leaderboard.status, &model.leaderboard.loaded);
            update!(self.leaderboard, leaderboard);
            context.update_focus(self.leaderboard.state.hovered);
        } else {
            self.leaderboard.hide();
        }

        !context.can_focus
    }
}
