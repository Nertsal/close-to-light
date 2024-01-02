use super::*;

use crate::{
    leaderboard::{LeaderboardStatus, LoadedBoard},
    prelude::Assets,
    ui::layout,
};

pub struct LeaderboardWidget {
    pub state: WidgetState,
    pub close: ButtonWidget,
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
    pub fn new(assets: &Rc<Assets>) -> Self {
        Self {
            state: WidgetState::new(),
            close: ButtonWidget::new_textured("", &assets.sprites.button_close),
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
        player_name: &str,
    ) {
        // let player_name = board.local_high.as_ref().map_or("", |entry| &entry.player);
        self.rows.clear();
        self.status.text = "".to_string();
        match state {
            LeaderboardStatus::None => self.status.text = "NOT AVAILABLE".to_string(),
            LeaderboardStatus::Pending => self.status.text = "LOADING...".to_string(),
            LeaderboardStatus::Failed => self.status.text = "FETCH FAILED :(".to_string(),
            LeaderboardStatus::Done => {
                if board.filtered.is_empty() {
                    self.status.text = "EMPTY :(".to_string();
                }
            }
        }
        self.load_scores(board, player_name);
    }

    pub fn load_scores(&mut self, board: &LoadedBoard, player_name: &str) {
        self.rows = board
            .filtered
            .iter()
            .enumerate()
            .map(|(rank, entry)| {
                LeaderboardEntryWidget::new(
                    (rank + 1).to_string(),
                    &entry.player,
                    entry.score,
                    entry.player == player_name,
                )
            })
            .collect();
        match &board.local_high {
            None => self.highscore.hide(),
            Some(score) => {
                self.highscore.show();
                self.highscore.rank.text = board
                    .my_position
                    .map_or("???".to_string(), |rank| format!("{}.", rank + 1));
                self.highscore.player.text = player_name.to_string(); // score.player.clone();
                self.highscore.score.text = format!("{}", score.score);
            }
        }
    }
}

impl Widget for LeaderboardWidget {
    fn update(&mut self, position: Aabb2<f32>, context: &UiContext) {
        self.state.update(position, context);
        let main = position;

        let close = layout::align_aabb(
            vec2::splat(1.0) * context.font_size,
            main.extend_uniform(-0.5 * context.layout_size),
            vec2(0.0, 1.0),
        );
        self.close.update(close, context);

        let main = main
            .extend_symmetric(-vec2(1.0, 0.0) * context.layout_size)
            .extend_up(-context.layout_size);

        let (title, main) = layout::cut_top_down(main, context.font_size * 1.2);
        self.title.update(title, &context.scale_font(1.1));

        let (subtitle, main) = layout::cut_top_down(main, context.font_size * 1.0);
        self.subtitle.update(subtitle, context);

        let (status, _) = layout::cut_top_down(main, context.font_size * 1.0);
        self.status.update(status, context);

        let main = main.extend_right(-0.5 * context.font_size);

        let (main, highscore) = layout::cut_top_down(main, main.height() - context.font_size * 1.5);
        self.highscore.update(highscore, context);

        let (main, separator) = layout::cut_top_down(main, main.height() - context.font_size * 0.1);
        let separator = separator.extend_right(0.5 * context.font_size);
        self.separator.update(separator, context);

        let main = main.extend_down(-0.2 * context.font_size);

        self.rows_state.update(main, context);
        let main = main.translate(vec2(0.0, -self.scroll));
        let row = Aabb2::point(main.top_left())
            .extend_right(main.width())
            .extend_down(context.font_size * 1.0);
        let rows = layout::stack(row, vec2(0.0, -context.font_size * 1.0), self.rows.len());
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

    fn walk_states_mut(&mut self, f: &dyn Fn(&mut WidgetState)) {
        self.state.walk_states_mut(f);
        self.title.walk_states_mut(f);
        self.subtitle.walk_states_mut(f);
        for row in &mut self.rows {
            row.walk_states_mut(f);
        }
    }
}

impl LeaderboardEntryWidget {
    pub fn new(
        rank: impl Into<String>,
        player: impl Into<String>,
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
    fn update(&mut self, position: Aabb2<f32>, context: &UiContext) {
        let main = position;

        let (rank, main) = layout::cut_left_right(main, context.font_size * 1.0);
        self.rank.update(rank, context);
        let main = main.extend_left(-context.font_size * 0.2);

        let (main, score) = layout::cut_left_right(main, main.width() - context.font_size * 5.0);
        self.score.update(score, context);

        let player = main;
        self.player.update(player, context);
        self.player.options.color = if self.highlight {
            context.theme.danger
        } else {
            context.theme.light
        }
    }

    fn walk_states_mut(&mut self, f: &dyn Fn(&mut WidgetState)) {
        self.state.walk_states_mut(f);
        self.rank.walk_states_mut(f);
        self.player.walk_states_mut(f);
        self.score.walk_states_mut(f);
    }
}
