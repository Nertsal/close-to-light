use super::*;

use crate::{
    leaderboard::{LeaderboardStatus, LoadedBoard},
    prelude::Assets,
    ui::layout::AreaOps,
};

use ctl_client::core::types::{Name, UserInfo};

pub struct LeaderboardWidget {
    pub state: WidgetState,
    pub window: UiWindow<()>,
    // pub close: IconButtonWidget,
    pub show_title: bool,
    pub title: TextWidget,
    pub subtitle: TextWidget,
    pub status: TextWidget,
    pub scroll: f32,
    pub target_scroll: f32,
    pub rows_state: WidgetState,
    pub rows: Vec<LeaderboardEntryWidget>,
    pub separator: WidgetState,
    pub highscore: LeaderboardEntryWidget,
}

pub struct LeaderboardEntryWidget {
    pub state: WidgetState,
    pub rank: TextWidget,
    pub player: TextWidget,
    pub score: TextWidget,
    pub highlight: bool,
}

impl LeaderboardWidget {
    pub fn new(_assets: &Rc<Assets>, show_title: bool) -> Self {
        Self {
            state: WidgetState::new(),
            window: UiWindow::new((), 0.3).reload_skip(),
            // close: IconButtonWidget::new_close_button(&assets.sprites.button_close),
            show_title,
            title: TextWidget::new("LEADERBOARD"),
            subtitle: TextWidget::new("TOP WORLD"),
            status: TextWidget::new(""),
            scroll: 0.0,
            target_scroll: 0.0,
            rows_state: WidgetState::new(),
            rows: Vec::new(),
            separator: WidgetState::new(),
            highscore: LeaderboardEntryWidget::new("", "", 0, false),
        }
    }

    pub fn update_state(
        &mut self,
        state: &LeaderboardStatus,
        board: &LoadedBoard,
        user: &UserInfo,
    ) {
        // let player_name = board.local_high.as_ref().map_or("", |entry| &entry.player);
        self.rows.clear();
        self.status.text = "".into();
        match state {
            LeaderboardStatus::None => self.status.text = "NOT AVAILABLE".into(),
            LeaderboardStatus::Pending => self.status.text = "LOADING...".into(),
            LeaderboardStatus::Failed => self.status.text = "FETCH FAILED :(".into(),
            LeaderboardStatus::Done => {
                if board.filtered.is_empty() {
                    self.status.text = "EMPTY :(".into();
                }
            }
        }
        self.load_scores(board, user);
    }

    pub fn load_scores(&mut self, board: &LoadedBoard, user: &UserInfo) {
        self.rows = board
            .filtered
            .iter()
            .enumerate()
            .map(|(rank, entry)| {
                LeaderboardEntryWidget::new(
                    (rank + 1).to_string(),
                    entry.user.name.clone(),
                    entry.score,
                    entry.user.id == user.id,
                )
            })
            .collect();
        match &board.local_high {
            None => self.highscore.hide(),
            Some(score) => {
                self.highscore.show();
                self.highscore.rank.text = board
                    .my_position
                    .map_or("???".into(), |rank| format!("{}.", rank + 1).into());
                self.highscore.player.text = user.name.clone();
                self.highscore.score.text = format!("{}", score.score).into();
            }
        }
    }
}

impl Widget for LeaderboardWidget {
    fn state_mut(&mut self) -> &mut WidgetState {
        &mut self.state
    }

    fn update(&mut self, position: Aabb2<f32>, context: &mut UiContext) {
        self.state.update(position, context);
        self.window.update(context.delta_time);

        let main = position;

        // let close = layout::align_aabb(
        //     vec2::splat(1.0) * context.font_size,
        //     main.extend_uniform(-0.5 * context.layout_size),
        //     vec2(0.0, 1.0),
        // );
        // self.close.update(close, context);

        let mut main = main
            .extend_symmetric(-vec2(1.0, 0.0) * context.layout_size)
            .extend_up(-context.layout_size);

        let title = main.cut_top(context.font_size * 1.2);
        if self.show_title {
            self.title.update(title, &mut context.scale_font(1.1)); // TODO: better
        }

        let subtitle = main.cut_top(context.font_size * 1.0);
        self.subtitle.update(subtitle, context);

        let status = main.clone().cut_top(context.font_size * 1.0);
        self.status.update(status, context);

        main.cut_right(0.5 * context.font_size);

        let highscore = main.cut_bottom(context.font_size * 1.5);
        self.highscore.update(highscore, context);

        let separator = main.cut_bottom(context.font_size * 0.1);
        let separator = separator.extend_right(0.5 * context.font_size);
        self.separator.update(separator, context);

        main.cut_bottom(0.2 * context.font_size);

        self.rows_state.update(main, context);
        let main = main.translate(vec2(0.0, -self.scroll));
        let row = Aabb2::point(main.top_left())
            .extend_right(main.width())
            .extend_down(context.font_size * 1.0);
        let rows = row.stack(vec2(0.0, -context.font_size * 1.0), self.rows.len());
        let height = rows.last().map_or(0.0, |row| main.max.y - row.min.y);
        for (row, position) in self.rows.iter_mut().zip(rows) {
            row.update(position, context);
        }

        self.target_scroll += context.cursor.scroll;
        let overflow_up = self.target_scroll;
        let max_scroll = (height - main.height()).max(0.0);
        let overflow_down = -max_scroll - self.target_scroll;
        let overflow = if overflow_up > 0.0 {
            overflow_up
        } else if overflow_down > 0.0 {
            -overflow_down
        } else {
            0.0
        };
        self.target_scroll -= overflow * (context.delta_time / 0.2).min(1.0);

        self.scroll += (self.target_scroll - self.scroll) * (context.delta_time / 0.1).min(1.0);
    }
}

impl LeaderboardEntryWidget {
    pub fn new(
        rank: impl Into<Name>,
        player: impl Into<Name>,
        score: i32,
        highlight: bool,
    ) -> Self {
        let rank = rank.into();
        let mut rank = TextWidget::new(format!("{}.", rank));
        rank.align(vec2(1.0, 0.5));

        let mut player = TextWidget::new(player);
        player.align(vec2(0.0, 0.5));

        let mut score = TextWidget::new(format!("{}", score));
        score.align(vec2(1.0, 0.5));

        Self {
            state: WidgetState::new(),
            rank,
            player,
            score,
            highlight,
        }
    }
}

impl Widget for LeaderboardEntryWidget {
    fn state_mut(&mut self) -> &mut WidgetState {
        &mut self.state
    }

    fn update(&mut self, position: Aabb2<f32>, context: &mut UiContext) {
        let mut main = position;

        let rank = main.cut_left(context.font_size * 1.0);
        self.rank.update(rank, context);
        main.cut_left(context.font_size * 0.2);

        let score = main.cut_right(context.font_size * 5.0);
        self.score.update(score, context);

        let player = main;
        self.player.update(player, context);
        self.player.options.color = if self.highlight {
            context.theme.danger
        } else {
            context.theme.light
        }
    }
}
