use super::*;

use crate::game::ui::layout::AreaOps;
pub use crate::ui::{layout, widget::*, *};

pub struct GameUI {
    pub leaderboard_head: TextWidget,
    pub leaderboard: LeaderboardWidget,
    pub score: ScoreWidget,
}

impl GameUI {
    pub fn new(assets: &Rc<Assets>) -> Self {
        let mut leaderboard = LeaderboardWidget::new(assets, false);
        leaderboard.reload.hide();
        Self {
            leaderboard_head: TextWidget::new("Leaderboard").rotated(Angle::from_degrees(90.0)),
            leaderboard,
            score: ScoreWidget::new(assets),
        }
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

        // Margin
        let mut main = screen.extend_uniform(-layout_size * 2.0);

        // Logo
        let _ctl_logo = main.cut_top(layout_size * 4.0);
        // update!(self.ctl_logo, ctl_logo);
        let main = main.extend_up(-layout_size * 3.0);

        if let State::Lost { .. } | State::Finished = model.state {
            self.leaderboard.show();
            self.score.show();

            // Score
            {
                let width = layout_size * 13.0;
                let height = layout_size * 20.0;

                let score = Aabb2::point(main.bottom_right() + vec2(0.0, 2.0) * layout_size)
                    .extend_left(width)
                    .extend_down(height);

                let offset = main.height();

                let score = score.translate(vec2(-layout_size * 7.0, offset));
                self.score.update_state(
                    &ctl_local::ScoreMeta {
                        category: ctl_local::ScoreCategory::new(
                            model.level.config.modifiers.clone(),
                            model.level.config.health.clone(),
                        ),
                        score: model.score.clone(),
                    },
                    &model
                        .level
                        .group
                        .music
                        .as_ref()
                        .map(|music| music.meta.clone())
                        .unwrap_or_default(),
                    &model.level.level.meta,
                );
                self.score.update(score, context);
                context.update_focus(self.score.state.hovered);
            }

            // Leaderboard
            {
                let main = screen;

                let size = vec2(layout_size * 22.0, main.height() - layout_size * 6.0);
                let head_size = vec2(context.font_size, layout_size * 8.0);
                let pos = main.align_pos(vec2(1.0, 0.5));

                let hover_t = self.leaderboard.window.show.time.get_ratio();
                let hover_t = crate::util::smoothstep(hover_t);

                let slide =
                    vec2(-1.0, 0.0) * (hover_t * (size.x + layout_size * 2.0) + head_size.x);

                let up = 0.4;
                let leaderboard = Aabb2::point(pos + vec2(head_size.x, 0.0) + slide)
                    .extend_right(size.x)
                    .extend_up(size.y * up)
                    .extend_down(size.y * (1.0 - up));
                let leaderboard_head = Aabb2::point(pos + slide)
                    .extend_right(head_size.x)
                    .extend_symmetric(vec2(0.0, head_size.y) / 2.0);

                self.leaderboard.update_state(&model.leaderboard);
                self.leaderboard.update(leaderboard, context);
                self.leaderboard_head.update(leaderboard_head, context);
                context.update_focus(self.leaderboard.state.hovered);

                let hover = self.leaderboard.state.hovered || self.leaderboard_head.state.hovered;
                self.leaderboard.window.layout(
                    hover,
                    context.cursor.position.x < leaderboard.min.x && !hover,
                );
            }
        } else {
            self.leaderboard.hide();
            self.score.hide();
        }

        !context.can_focus()
    }
}
