use super::*;

pub struct LeaderboardWidget {
    pub state: WidgetState,
    pub title: TextWidget,
    pub subtitle: TextWidget,
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
            rows: Vec::new(),
        }
    }
}

impl Widget for LeaderboardWidget {
    fn update(&mut self, position: Aabb2<f32>, context: &UiContext) {
        self.state.update(position, context);
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

impl Widget for LeaderboardEntryWidget {
    fn update(&mut self, position: Aabb2<f32>, context: &UiContext) {}

    fn walk_states_mut(&mut self, f: &dyn Fn(&mut WidgetState)) {
        self.state.walk_states_mut(f);
        self.rank.walk_states_mut(f);
        self.player.walk_states_mut(f);
        self.score.walk_states_mut(f);
    }
}
