use super::*;

use crate::{prelude::Assets, ui::layout::AreaOps};

use ctl_core::types::{Name, UserInfo};
use ctl_local::{Leaderboard, LeaderboardStatus, LoadedBoard, SavedScore};
use ctl_ui::util::ScrollState;

pub struct LeaderboardWidget {
    pub state: WidgetState,
    pub assets: Rc<Assets>,
    pub window: UiWindow<()>,
    pub pin: ToggleButtonWidget,
    pub reload: IconButtonWidget,
    pub show_title: bool,
    pub title: TextWidget,
    pub subtitle: TextWidget,
    pub level_name: TextWidget,
    pub separator_title: WidgetState,
    pub status: TextWidget,
    pub scroll: ScrollState,
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
    pub accuracy: TextWidget,
    pub highlight: bool,
    pub score_grade: ScoreGrade,
    pub grade: IconWidget,
    pub modifiers: Vec<IconWidget>,
}

impl LeaderboardWidget {
    pub fn new(assets: &Rc<Assets>, show_title: bool) -> Self {
        Self {
            state: WidgetState::new().with_sfx(WidgetSfxConfig::hover()),
            assets: assets.clone(),
            window: UiWindow::new((), 0.3).reload_skip(),
            pin: ToggleButtonWidget::new_deselectable("").with_icon(assets.atlas.pin()),
            reload: IconButtonWidget::new_normal(assets.atlas.reset()),
            show_title,
            title: TextWidget::new("LEADERBOARD"),
            subtitle: TextWidget::new("login to submit scores"),
            level_name: TextWidget::new("Level - Difficulty"),
            separator_title: WidgetState::new(),
            status: TextWidget::new(""),
            scroll: ScrollState::new(),
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
        if leaderboard.get_user().is_some() {
            self.subtitle.hide();
        } else {
            self.subtitle.show();
        }

        let user = &leaderboard.get_user().as_ref().map_or(
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
        match leaderboard.get().status {
            LeaderboardStatus::None => self.status.text = "NOT AVAILABLE".into(),
            LeaderboardStatus::Pending => self.status.text = "LOADING...".into(),
            LeaderboardStatus::Failed => self.status.text = "FETCH FAILED :(".into(),
            LeaderboardStatus::Done => {
                if leaderboard.get_loaded().filtered.is_empty() {
                    self.status.text = "EMPTY :(".into();
                }
            }
        }
        self.load_scores(&leaderboard.get_loaded(), user);
    }

    pub fn load_scores(&mut self, board: &LoadedBoard, user: &UserInfo) {
        self.level_name.text = format!("{} - {}", board.music.name, board.level.name).into();
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
                        .map_or("??".into(), |rank| format!("{}", rank + 1)),
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
        if self.pin.selected {
            // Nullifying a request will prevent the window from getting closed.
            self.window.request = None;
        }
        self.window.update(context.delta_time);

        let main = position;

        self.scroll.drag(context, &self.state);

        let pin = main
            .extend_uniform(-0.5 * context.layout_size)
            .align_aabb(vec2::splat(1.0) * context.font_size, vec2(0.0, 1.0));
        self.pin.update(pin, context);

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
            self.title.update(title, &context.scale_font(1.1));
        }

        if self.subtitle.state.visible {
            let subtitle = main.cut_top(context.font_size * 0.7);
            self.subtitle.update(subtitle, context);

            let level_name = main.cut_top(context.font_size * 1.0);
            self.level_name.update(level_name, context);

            main.cut_top(context.font_size * 0.3);
        } else {
            main.cut_top(context.font_size * 0.5);
            let level_name = main.cut_top(context.font_size * 1.0);
            main.cut_top(context.font_size * 0.5);
            self.level_name.update(level_name, context);
        }

        let separator = main.cut_top(context.font_size * 0.1);
        self.separator_title.update(separator, context);

        let mut status = main.clone().cut_top(context.font_size * 3.0);
        status.cut_top(context.font_size);
        self.status.update(status, context);

        let highscore = main.cut_bottom(context.font_size * 2.0);
        self.highscore.update(highscore, context);

        let separator = main
            .cut_bottom(context.font_size * 0.5)
            .with_height(context.font_size * 0.1, 0.0)
            .with_width(main.width() * 0.8, 0.5);
        self.separator_highscore.update(separator, context);

        main.cut_bottom(0.2 * context.font_size);

        self.rows_state.update(main, context);
        let main = main.translate(vec2(0.0, -self.scroll.state.current));
        let row = Aabb2::point(main.top_left())
            .extend_right(main.width())
            .extend_down(context.font_size * 2.0);
        let rows = row.stack(vec2(0.0, -row.height()), self.rows.len());
        let height =
            rows.first().map_or(0.0, |row| row.max.y) - rows.last().map_or(0.0, |row| row.min.y);
        for (row, position) in self.rows.iter_mut().zip(rows) {
            row.update(position, context);
        }

        self.scroll
            .overflow(context.delta_time, height, main.height());
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
        rank.align(vec2(1.0, 0.0));

        let mut player = TextWidget::new(score.user.name.clone());
        player.align(vec2(0.0, 0.0));

        let modifiers = score
            .meta
            .category
            .mods
            .iter()
            .map(|modifier| IconWidget::new(assets.get_modifier(modifier)))
            .collect();

        let score_grade = score.meta.score.calculate_grade(score.meta.completion);
        let grade = IconWidget::new(assets.get_grade(score_grade));

        let accuracy = TextWidget::new(format!(
            "rhythm: {}%",
            (score.meta.score.calculated.accuracy.as_f32() * 100.0).floor() as i32,
        ))
        .aligned(vec2(1.0, 1.0));
        let score = TextWidget::new(format!("{}", score.score)).aligned(vec2(1.0, 0.0));

        Self {
            state: WidgetState::new(),
            rank,
            player,
            score,
            accuracy,
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

        main.cut_top(context.layout_size * 0.5);
        main.cut_bottom(context.layout_size * 0.5);

        let mut top_row = main;
        let bottom_row = top_row.split_bottom(0.5);
        let mod_pos = bottom_row.align_aabb(
            vec2(bottom_row.height(), bottom_row.height()),
            vec2(0.5, 0.5),
        );
        let mods = mod_pos.stack_aligned(
            vec2(mod_pos.width(), 0.0),
            self.modifiers.len(),
            vec2(0.5, 0.5),
        );
        for (modifier, pos) in self.modifiers.iter_mut().zip(mods) {
            modifier.update(pos, context);
            modifier.color = ThemeColor::Danger;
        }

        let mut right = main;
        right.cut_right(context.layout_size);
        let right = right.cut_right(main.width() / 3.0);
        let mut score = right;
        score.min.y = right.center().y - context.font_size * 0.25;
        self.score.update(score, context);
        let mut acc = right;
        acc.max.y = score.min.y;
        self.accuracy.update(acc, &context.scale_font(0.5));

        let grade = if self.modifiers.is_empty() {
            main
        } else {
            top_row
        };
        self.grade.update(grade, &context.scale_font(1.0));
        self.grade.color = match self.score_grade {
            ScoreGrade::F => ThemeColor::Danger,
            _ => ThemeColor::Highlight,
        };

        let mut rank_player = main;
        rank_player.min.y = main.center().y - context.font_size * 0.25;
        let rank = rank_player.cut_left(context.font_size * 1.0);
        self.rank.update(rank, context);
        rank_player.cut_left(context.font_size * 0.2);

        rank_player.max.x = main.center().x;

        self.player.update(rank_player, context);
        self.player.options.color = if self.highlight {
            theme.highlight
        } else {
            theme.light
        }
    }
}
