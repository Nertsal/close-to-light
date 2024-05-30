use super::*;

use crate::game::ui::layout::AreaOps;
pub use crate::ui::{layout, widget::*, *};

pub struct GameUI {
    pub leaderboard: LeaderboardWidget,
}

impl GameUI {
    pub fn new(assets: &Rc<Assets>) -> Self {
        let mut leaderboard = LeaderboardWidget::new(assets, true);
        leaderboard.reload.hide();
        Self { leaderboard }
    }

    pub fn layout(
        &mut self,
        model: &mut Model,
        screen: Aabb2<f32>,
        context: &mut UiContext,
    ) -> bool {
        // Fix aspect
        let screen = layout::fit_aabb(vec2(16.0, 9.0), screen, vec2::splat(0.5));

        let layout_size = screen.height() * 0.03;

        context.layout_size = layout_size;
        context.font_size = screen.height() * 0.05;

        macro_rules! update {
            ($widget:expr, $position:expr) => {{
                $widget.update($position, context);
            }};
        }

        // Margin
        let mut main = screen.extend_uniform(-layout_size * 2.0);

        // Logo
        let _ctl_logo = main.cut_top(layout_size * 4.0);
        // update!(self.ctl_logo, ctl_logo);
        let main = main.extend_up(-layout_size * 3.0);

        if let State::Lost { .. } | State::Finished = model.state {
            // Leaderboard
            self.leaderboard.show();
            // self.leaderboard.close.hide();

            let width = layout_size * 20.0;
            let height = main.height() + layout_size * 2.0;

            let leaderboard = Aabb2::point(main.bottom_right() + vec2(0.0, 2.0) * layout_size)
                .extend_left(width)
                .extend_down(height);

            let offset = main.height();

            let leaderboard = leaderboard.translate(vec2(0.0, offset));

            self.leaderboard.update_state(&model.leaderboard);
            update!(self.leaderboard, leaderboard);
            context.update_focus(self.leaderboard.state.hovered);
        } else {
            self.leaderboard.hide();
        }

        !context.can_focus
    }
}
