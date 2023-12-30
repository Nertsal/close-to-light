use crate::ui::layout;

use super::*;

pub struct LevelConfigWidget {
    pub state: WidgetState,
    pub tab_difficulty: TextWidget,
    pub tab_mods: TextWidget,
    pub difficulty: LevelDifficultyWidget,
    pub mods: LevelModsWidget,
}

impl LevelConfigWidget {
    pub fn new() -> Self {
        let mut w = Self {
            state: WidgetState::new(),
            tab_difficulty: TextWidget::new("Difficulty"),
            tab_mods: TextWidget::new("Modifiers"),
            difficulty: LevelDifficultyWidget::new(),
            mods: LevelModsWidget::new(),
        };
        w.difficulty.hide();
        w.mods.hide();
        w
    }
}

impl Widget for LevelConfigWidget {
    fn update(&mut self, position: Aabb2<f32>, context: &UiContext) {
        self.state.update(position, context);
        let main = position;

        let (bar, main) = layout::cut_top_down(main, context.font_size * 1.2);
        let tab =
            Aabb2::point(bar.bottom_left()).extend_positive(vec2(5.0, 1.0) * context.font_size);
        let tabs = layout::stack(tab, vec2(tab.width(), 0.0), 2);
        for (tab, pos) in [&mut self.tab_difficulty, &mut self.tab_mods]
            .into_iter()
            .zip(tabs)
        {
            tab.update(pos, context);
        }

        if self.tab_difficulty.state.clicked {
            self.difficulty.show();
            self.mods.hide();
        } else if self.tab_mods.state.clicked {
            self.difficulty.hide();
            self.mods.show();
        }

        self.difficulty.update(main, context);
        self.mods.update(main, context);
    }

    fn walk_states_mut(&mut self, f: &dyn Fn(&mut WidgetState)) {
        self.state.walk_states_mut(f);
        self.tab_difficulty.walk_states_mut(f);
        self.tab_mods.walk_states_mut(f);
        self.difficulty.walk_states_mut(f);
        self.mods.walk_states_mut(f);
    }
}

pub struct LevelDifficultyWidget {
    pub state: WidgetState,
}

impl LevelDifficultyWidget {
    pub fn new() -> Self {
        Self {
            state: WidgetState::new(),
        }
    }
}

impl Widget for LevelDifficultyWidget {
    fn update(&mut self, position: Aabb2<f32>, context: &UiContext) {
        self.state.update(position, context);
    }

    fn walk_states_mut(&mut self, f: &dyn Fn(&mut WidgetState)) {
        self.state.walk_states_mut(f);
    }
}

pub struct LevelModsWidget {
    pub state: WidgetState,
}

impl LevelModsWidget {
    pub fn new() -> Self {
        Self {
            state: WidgetState::new(),
        }
    }
}

impl Widget for LevelModsWidget {
    fn update(&mut self, position: Aabb2<f32>, context: &UiContext) {
        self.state.update(position, context);
    }

    fn walk_states_mut(&mut self, f: &dyn Fn(&mut WidgetState)) {
        self.state.walk_states_mut(f);
    }
}
