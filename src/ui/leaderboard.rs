use super::*;

use crate::{prelude::Assets, ui::layout::AreaOps};

use ctl_core::types::{Name, UserInfo};
use ctl_local::{Leaderboard, LeaderboardStatus, LoadedBoard, SavedScore};
use ctl_util::{SecondOrderDynamics, SecondOrderState};

pub struct LeaderboardWidget {
    pub state: WidgetState,
    pub assets: Rc<Assets>,
    pub window: UiWindow<()>,
    // pub close: IconButtonWidget,
    pub reload: IconButtonWidget,
    pub show_title: bool,
    pub title: TextWidget,
    pub subtitle: TextWidget,
    pub separator_title: WidgetState,
    pub status: TextWidget,
    pub scroll: SecondOrderState<f32>,
    scroll_drag_from: f32,
    pub rows_state: WidgetState,
    pub rows: Vec<LeaderboardEntryWidget>,
    pub separator_highscore: WidgetState,
    pub highscore: LeaderboardEntryWidget,
}

pub struct LeaderboardEntryWidget {
    pub state: WidgetState,
    pub rank: TextWidget,
    pub player: TextWidget,
    pub score: TextWidget,
    pub highlight: bool,
    pub score_grade: ScoreGrade,
    pub grade: TextWidget,
    pub modifiers: Vec<IconWidget>,
}

impl LeaderboardWidget {
    pub fn new(assets: &Rc<Assets>, show_title: bool) -> Self {
        Self {
            state: WidgetState::new().with_sfx(WidgetSfxConfig::hover()),
            assets: assets.clone(),
            window: UiWindow::new((), 0.3).reload_skip(),
            // close: IconButtonWidget::new_close_button(&assets.sprites.button_close),
            reload: IconButtonWidget::new_normal(assets.atlas.reset()),
            show_title,
            title: TextWidget::new("LEADERBOARD"),
            subtitle: TextWidget::new("login to submit scores"),
            separator_title: WidgetState::new(),
            status: TextWidget::new(""),
            scroll: SecondOrderState::new(SecondOrderDynamics::new(5.0, 2.0, 0.0, 0.0)),
            scroll_drag_from: 0.0,
            rows_state: WidgetState::new(),
            rows: Vec::new(),
            separator_highscore: WidgetState::new(),
            highscore: LeaderboardEntryWidget::new(
                assets,
                "",
                SavedScore {
                    user: UserInfo {
                        id: 0,
                        name: "player".into(),
                    },
                    score: 0,
                    meta: ctl_local::ScoreMeta::default(),
                },
                false,
            ),
        }
    }

    pub fn update_state(&mut self, leaderboard: &Leaderboard) {
        if leaderboard.user.is_some() {
            self.subtitle.hide();
        } else {
            self.subtitle.show();
        }

        let user = &leaderboard.user.as_ref().map_or(
            UserInfo {
                id: 0,
                name: "local highscore".into(),
            },
            |user| UserInfo {
                id: user.id,
                name: user.name.clone(),
            },
        );
        // let player_name = board.local_high.as_ref().map_or("", |entry| &entry.player);

        self.rows.clear();
        self.status.text = "".into();
        match leaderboard.status {
            LeaderboardStatus::None => self.status.text = "NOT AVAILABLE".into(),
            LeaderboardStatus::Pending => self.status.text = "LOADING...".into(),
            LeaderboardStatus::Failed => self.status.text = "FETCH FAILED :(".into(),
            LeaderboardStatus::Done => {
                if leaderboard.loaded.filtered.is_empty() {
                    self.status.text = "EMPTY :(".into();
                }
            }
        }
        self.load_scores(&leaderboard.loaded, user);
    }

    pub fn load_scores(&mut self, board: &LoadedBoard, user: &UserInfo) {
        self.rows = board
            .filtered
            .iter()
            .enumerate()
            .filter_map(|(rank, entry)| {
                let meta = entry
                    .extra_info
                    .as_ref()
                    .and_then(|meta| serde_json::from_str(meta).ok())?;
                let score = SavedScore {
                    user: entry.user.clone(),
                    score: entry.score,
                    meta,
                };
                Some(LeaderboardEntryWidget::new(
                    &self.assets,
                    (rank + 1).to_string(),
                    score,
                    entry.user.id == user.id,
                ))
            })
            .collect();
        match &board.local_high {
            None => self.highscore.hide(),
            Some(score) => {
                self.highscore = LeaderboardEntryWidget::new(
                    &self.assets,
                    board
                        .my_position
                        .map_or("???".into(), |rank| format!("{}", rank + 1)),
                    score.clone(),
                    false,
                );
            }
        }
    }
}

impl WidgetOld for LeaderboardWidget {
    fn state_mut(&mut self) -> &mut WidgetState {
        &mut self.state
    }

    fn update(&mut self, position: Aabb2<f32>, context: &mut UiContext) {
        self.state.update(position, context);
        self.window.update(context.delta_time);

        let main = position;

        ctl_ui::util::scroll_drag(
            context,
            &self.state,
            &mut self.scroll,
            &mut self.scroll_drag_from,
        );

        // let close = layout::align_aabb(
        //     vec2::splat(1.0) * context.font_size,
        //     main.extend_uniform(-0.5 * context.layout_size),
        //     vec2(0.0, 1.0),
        // );
        // self.close.update(close, context);

        let reload = main
            .extend_uniform(-0.5 * context.layout_size)
            .align_aabb(vec2::splat(1.0) * context.font_size, vec2(1.0, 1.0));
        self.reload.update(reload, context);
        if self.reload.icon.state.mouse_left.clicked {
            self.window.request = Some(WidgetRequest::Reload);
        }

        let mut main = main
            .extend_symmetric(-vec2(1.0, 0.0) * context.layout_size)
            .extend_up(-context.layout_size);

        let title = main.cut_top(context.font_size * 1.2);
        if self.show_title {
            self.title.update(title, &context.scale_font(1.1)); // TODO: better
        }

        let subtitle = main.cut_top(context.font_size * 1.0);
        self.subtitle.update(subtitle, context);

        let separator = main.cut_top(context.font_size * 0.1);
        self.separator_title.update(separator, context);

        let status = main.clone().cut_top(context.font_size * 1.0);
        self.status.update(status, context);

        let highscore = main.cut_bottom(context.font_size * 2.0);
        self.highscore.update(highscore, context);

        let separator = main.cut_bottom(context.font_size * 0.1);
        self.separator_highscore.update(separator, context);

        main.cut_bottom(0.2 * context.font_size);

        self.rows_state.update(main, context);
        let main = main.translate(vec2(0.0, -self.scroll.current));
        let row = Aabb2::point(main.top_left())
            .extend_right(main.width())
            .extend_down(context.font_size * 2.0);
        let rows = row.stack(vec2(0.0, -row.height()), self.rows.len());
        let height = rows.last().map_or(0.0, |row| main.max.y - row.min.y);
        for (row, position) in self.rows.iter_mut().zip(rows) {
            row.update(position, context);
        }

        ctl_ui::util::overflow_scroll(
            context.delta_time,
            self.scroll.current,
            &mut self.scroll.target,
            height,
            main.height(),
        );
    }
}

impl LeaderboardEntryWidget {
    pub fn new(
        assets: &Rc<Assets>,
        rank: impl Into<Name>,
        score: SavedScore,
        highlight: bool,
    ) -> Self {
        let rank = rank.into();
        let mut rank = TextWidget::new(format!("{rank}."));
        rank.align(vec2(1.0, 0.5));

        let mut player = TextWidget::new(score.user.name.clone());
        player.align(vec2(0.0, 0.5));

        let modifiers = score
            .meta
            .category
            .mods
            .iter()
            .map(|modifier| IconWidget::new(assets.get_modifier(modifier)))
            .collect();

        let score_grade = score.meta.score.calculate_grade(score.meta.completion);
        let grade = TextWidget::new(format!("{score_grade}"));

        let mut score = TextWidget::new(format!(
            "{} ({}/{})",
            score.score,
            (score.meta.score.calculated.accuracy.as_f32() * 100.0).floor() as i32,
            (score.meta.score.calculated.precision.as_f32() * 100.0).floor()
        ));
        score.align(vec2(1.0, 0.5));

        Self {
            state: WidgetState::new(),
            rank,
            player,
            score,
            highlight,
            score_grade,
            grade,
            modifiers,
        }
    }
}

impl WidgetOld for LeaderboardEntryWidget {
    fn state_mut(&mut self) -> &mut WidgetState {
        &mut self.state
    }

    fn update(&mut self, position: Aabb2<f32>, context: &mut UiContext) {
        self.state.update(position, context);
        let mut main = position;
        let theme = context.theme();

        let mut bottom_row = main.cut_bottom(context.font_size * 1.0);
        bottom_row.cut_left(context.layout_size);
        bottom_row.cut_right(context.layout_size);
        let mod_pos = bottom_row.align_aabb(
            vec2(bottom_row.height(), bottom_row.height()),
            vec2(1.0, 0.5),
        );
        let mods = mod_pos.stack_aligned(
            vec2(mod_pos.width(), 0.0),
            self.modifiers.len(),
            vec2(1.0, 0.5),
        );
        for (modifier, pos) in self.modifiers.iter_mut().zip(mods) {
            modifier.update(pos, context);
        }

        self.grade.update(bottom_row, &context.scale_font(1.0));
        self.grade.options.color = match self.score_grade {
            ScoreGrade::F => theme.danger,
            _ => theme.highlight,
        };

        let rank = main.cut_left(context.font_size * 1.0);
        self.rank.update(rank, context);
        main.cut_left(context.font_size * 0.2);

        main.cut_right(context.layout_size);

        let score = main.cut_right(main.width() / 2.0);
        self.score.update(score, context);

        let player = main;
        self.player.update(player, context);
        self.player.options.color = if self.highlight {
            theme.highlight
        } else {
            theme.light
        }
    }
}
