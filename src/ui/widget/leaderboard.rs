use super::*;

use crate::{prelude::LeaderboardState, ui::layout};

use nertboard_client::ScoreEntry;

pub struct LeaderboardWidget {
    pub state: WidgetState,
    pub title: TextWidget,
    pub subtitle: TextWidget,
    pub status: TextWidget,
    pub rows: Vec<LeaderboardEntryWidget>,
}

pub struct LeaderboardEntryWidget {
    pub state: WidgetState,
    pub rank: TextWidget,
    pub player: TextWidget,
    pub score: TextWidget,
}

impl LeaderboardWidget {
    pub fn new() -> Self {
        Self {
            state: WidgetState::new(),
            title: TextWidget::new("LEADERBOARD"),
            subtitle: TextWidget::new("TOP WORLD"),
            status: TextWidget::new(""),
            rows: Vec::new(),
        }
    }

    pub fn update_state(&mut self, state: &LeaderboardState) {
        self.rows.clear();
        self.status.text = "".to_string();
        match state {
            LeaderboardState::None => self.status.text = "what should be here?".to_string(),
            LeaderboardState::Pending => self.status.text = "LOADING...".to_string(),
            LeaderboardState::Failed => self.status.text = "FETCH FAILED :(".to_string(),
            LeaderboardState::Ready(board) => {
                if board.top10.is_empty() {
                    self.status.text = "EMPTY :(".to_string();
                } else {
                    self.load_scores(&board.top10);
                }
            }
        }
    }

    pub fn load_scores(&mut self, scores: &[ScoreEntry]) {
        self.rows = scores
            .iter()
            .enumerate()
            .map(|(rank, entry)| LeaderboardEntryWidget::new(rank + 1, &entry.player, entry.score))
            .collect();
    }
}

impl Widget for LeaderboardWidget {
    fn update(&mut self, position: Aabb2<f32>, context: &UiContext) {
        self.state.update(position, context);
        let main = position.extend_symmetric(-vec2(1.0, 0.0) * context.font_size);

        let (title, main) = layout::cut_top_down(main, context.font_size * 2.0);
        self.title.update(title, context);
        self.title.options.size = context.font_size * 1.5;

        let (subtitle, main) = layout::cut_top_down(main, context.font_size * 1.0);
        self.subtitle.update(subtitle, context);
        self.subtitle.options.size = context.font_size * 1.0;

        let (status, _) = layout::cut_top_down(main, context.font_size * 1.0);
        self.status.update(status, context);
        self.status.options.size = context.font_size * 1.0;

        let row = Aabb2::point(main.top_left())
            .extend_right(main.width())
            .extend_down(context.font_size * 1.0);
        let rows = layout::stack(row, vec2(0.0, -context.font_size * 1.0), self.rows.len());
        for (row, position) in self.rows.iter_mut().zip(rows) {
            row.update(position, context);
        }
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
    pub fn new(rank: usize, player: impl Into<String>, score: i32) -> Self {
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
        }
    }
}

impl Widget for LeaderboardEntryWidget {
    fn update(&mut self, position: Aabb2<f32>, context: &UiContext) {
        let main = position;

        let (rank, main) = layout::cut_left_right(main, context.font_size * 2.0);
        self.rank.update(rank, context);
        let main = main.extend_left(-context.font_size * 0.5);

        let (main, score) = layout::cut_left_right(main, main.width() - context.font_size * 7.0);
        self.score.update(score, context);

        let player = main;
        self.player.update(player, context);
    }

    fn walk_states_mut(&mut self, f: &dyn Fn(&mut WidgetState)) {
        self.state.walk_states_mut(f);
        self.rank.walk_states_mut(f);
        self.player.walk_states_mut(f);
        self.score.walk_states_mut(f);
    }
}
