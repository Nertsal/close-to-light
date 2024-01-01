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
}

impl LeaderboardWidget {
    pub fn new(assets: &Rc<Assets>) -> Self {
        Self {
            state: WidgetState::new(),
            close: ButtonWidget::new_textured("", &assets.sprites.button_close),
            title: TextWidget::new("LEADERBOARD"),
            subtitle: TextWidget::new("TOP WORLD"),
            status: TextWidget::new(""),
            rows_state: WidgetState::new(),
            rows: Vec::new(),
            separator: WidgetState::new(),
            highscore: LeaderboardEntryWidget::new("", "", 0),
        }
    }

    pub fn update_state(&mut self, state: &LeaderboardStatus, board: &LoadedBoard) {
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
        self.load_scores(board);
    }

    pub fn load_scores(&mut self, board: &LoadedBoard) {
        self.rows = board
            .filtered
            .iter()
            .enumerate()
            .map(|(rank, entry)| {
                LeaderboardEntryWidget::new((rank + 1).to_string(), &entry.player, entry.score)
            })
            .collect();
        match &board.local_high {
            None => self.highscore.hide(),
            Some(score) => {
                self.highscore.show();
                self.highscore.rank.text = board
                    .my_position
                    .map_or("".to_string(), |rank| format!("{}.", rank + 1));
                self.highscore.player.text = score.player.clone();
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
            vec2::splat(context.font_size * 0.75),
            main.extend_uniform(-context.font_size * 0.2),
            vec2(0.0, 1.0),
        );
        self.close.update(close, context);

        let main = main.extend_symmetric(-vec2(1.0, 0.0) * context.font_size);

        let (title, main) = layout::cut_top_down(main, context.font_size * 2.0);
        self.title.update(title, context);
        self.title.options.size = context.font_size * 1.5;

        let (subtitle, main) = layout::cut_top_down(main, context.font_size * 1.0);
        self.subtitle.update(subtitle, context);
        self.subtitle.options.size = context.font_size * 1.0;

        let (status, _) = layout::cut_top_down(main, context.font_size * 1.0);
        self.status.update(status, context);
        self.status.options.size = context.font_size * 1.0;

        let (main, highscore) = layout::cut_top_down(main, main.height() - context.font_size * 1.5);
        self.highscore.update(highscore, context);

        let (main, separator) = layout::cut_top_down(main, main.height() - context.font_size * 0.1);
        self.separator.update(separator, context);

        self.rows_state.update(main, context);
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
    pub fn new(rank: impl Into<String>, player: impl Into<String>, score: i32) -> Self {
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
